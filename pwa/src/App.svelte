<script>
  import { onMount } from 'svelte';
  import { getAllNotes, putNote, deleteNote } from './lib/db.js';
  import { syncNotes } from './lib/sync.js';
  import { getToken, setToken, apiFetch } from './lib/api.js';

  let notes = $state([]);
  let text = $state('');
  let remindAt = $state('');
  let showReminder = $state(false);
  let showCustomTime = $state(false);
  let syncing = $state(false);
  let token = $state(getToken());
  let showSettings = $state(!getToken());
  let syncError = $state('');
  let editingNote = $state(null);
  let showInstallPrompt = $state(false);
  let installPlatform = $state('');
  let deferredPrompt = $state(null);

  // Swipe state
  let swipeNoteId = $state(null);
  let swipeOffset = $state(0);
  let swipeStartX = 0;
  let swipeStartY = 0;
  const SWIPE_THRESHOLD = 80;

  function setReminder(minutes) {
    const d = new Date(Date.now() + minutes * 60000);
    remindAt = toLocalISO(d);
    showCustomTime = false;
  }

  function setReminderAt(hour, tomorrow = false) {
    const d = new Date();
    if (tomorrow) d.setDate(d.getDate() + 1);
    d.setHours(hour, 0, 0, 0);
    remindAt = toLocalISO(d);
    showCustomTime = false;
  }

  function toLocalISO(d) {
    return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 16);
  }

  function toUTCISO(localISO) {
    if (!localISO) return null;
    return new Date(localISO).toISOString();
  }

  function formatReminder(iso) {
    if (!iso) return '';
    const d = new Date(iso);
    const now = new Date();
    const diffMs = d - now;
    const diffMin = Math.round(diffMs / 60000);
    const diffHours = Math.round(diffMs / 3600000);

    const tomorrow = new Date(now);
    tomorrow.setDate(tomorrow.getDate() + 1);

    const timeStr = d.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });

    // Past reminders
    if (diffMs < 0) {
      if (d.toDateString() === now.toDateString()) return `Today ${timeStr}`;
      return d.toLocaleDateString([], { month: 'short', day: 'numeric' }) + ' ' + timeStr;
    }

    // Within next hour: "in X min"
    if (diffMin <= 60) return `in ${diffMin} min`;

    // Within next 3 hours: "in X hours"
    if (diffHours <= 3) return `in ${diffHours} hour${diffHours > 1 ? 's' : ''}`;

    // Later today: "at 4pm"
    if (d.toDateString() === now.toDateString()) return `at ${timeStr}`;

    // Tomorrow: "Tomorrow 6am"
    if (d.toDateString() === tomorrow.toDateString()) return `Tomorrow ${timeStr}`;

    // Further out: "Jan 15 2:00 PM"
    return d.toLocaleDateString([], { month: 'short', day: 'numeric' }) + ' ' + timeStr;
  }

  async function load() {
    const all = await getAllNotes();
    notes = all.filter(n => !n._deleted);
  }

  onMount(() => {
    load();
    checkInstallPrompt();
  });

  function checkInstallPrompt() {
    // Already installed as PWA
    if (window.matchMedia('(display-mode: standalone)').matches || window.navigator.standalone) {
      return;
    }

    // Already dismissed
    if (localStorage.getItem('install_dismissed')) {
      return;
    }

    const ua = navigator.userAgent;
    const isIOS = /iPad|iPhone|iPod/.test(ua) && !window.MSStream;
    const isAndroid = /Android/.test(ua);

    if (isIOS) {
      installPlatform = 'ios';
      showInstallPrompt = true;
    } else if (isAndroid) {
      installPlatform = 'android';
      // Listen for the beforeinstallprompt event (Chrome only)
      window.addEventListener('beforeinstallprompt', (e) => {
        e.preventDefault();
        deferredPrompt = e;
        showInstallPrompt = true;
      });
      // Fallback: show manual instructions after delay if no native prompt
      setTimeout(() => {
        if (!deferredPrompt && !showInstallPrompt) {
          showInstallPrompt = true;
        }
      }, 2000);
    }
  }

  function dismissInstall() {
    showInstallPrompt = false;
    localStorage.setItem('install_dismissed', 'true');
  }

  async function installApp() {
    if (deferredPrompt) {
      deferredPrompt.prompt();
      const { outcome } = await deferredPrompt.userChoice;
      if (outcome === 'accepted') {
        showInstallPrompt = false;
      }
      deferredPrompt = null;
    }
  }

  async function addNote() {
    if (!text.trim()) return;
    if (editingNote) {
      // Update existing note - reset done to false
      const updated = {
        ...editingNote,
        text: text.trim(),
        remind_at: showReminder && remindAt ? toUTCISO(remindAt) : null,
        done: false,
        updated_at: new Date().toISOString(),
      };
      await putNote(updated);

      // If synced, update on server directly
      if (editingNote._synced) {
        try {
          await apiFetch(`/api/notes/${editingNote.id}`, {
            method: 'PUT',
            body: JSON.stringify({ text: updated.text, remind_at: updated.remind_at, done: false }),
          });
        } catch (e) {
          // Will sync later
        }
      }
    } else {
      // Create new note
      const note = {
        id: crypto.randomUUID(),
        text: text.trim(),
        remind_at: showReminder && remindAt ? toUTCISO(remindAt) : null,
        synced: false,
        done: false,
        _synced: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      await putNote(note);
    }
    cancelEdit();
    await load();
    doSync();
  }

  function editNote(note) {
    editingNote = note;
    text = note.text;
    if (note.remind_at) {
      showReminder = true;
      remindAt = toLocalISO(new Date(note.remind_at));
    } else {
      showReminder = false;
      remindAt = '';
    }
    showCustomTime = false;
  }

  function cancelEdit() {
    editingNote = null;
    text = '';
    remindAt = '';
    showReminder = false;
    showCustomTime = false;
  }

  async function removeNote(note) {
    if (note._synced) {
      // Mark for deletion sync
      await putNote({ ...note, _deleted: true });
    } else {
      // Not synced yet, just delete locally
      await deleteNote(note.id);
    }
    if (editingNote?.id === note.id) {
      cancelEdit();
    }
    await load();
    doSync();
  }

  async function doSync() {
    syncing = true;
    syncError = '';
    try {
      await syncNotes();
      await load();
    } catch (e) {
      syncError = e.message;
    } finally {
      syncing = false;
    }
  }

  function saveToken() {
    setToken(token);
    showSettings = false;
    doSync();
  }

  function formatDate(iso) {
    return new Date(iso).toLocaleString();
  }

  function getReminderState(note) {
    if (!note.remind_at) return null;
    if (note.done) return 'done';
    const now = new Date();
    const remindAt = new Date(note.remind_at);
    return remindAt > now ? 'future' : 'overdue';
  }

  async function markDone(note) {
    // Update local - preserve _synced state
    const updated = { ...note, done: true, updated_at: new Date().toISOString() };
    await putNote(updated);

    // If synced, try to update server (fire and forget)
    if (note._synced) {
      apiFetch(`/api/notes/${note.id}/done`, { method: 'PATCH' }).catch(() => {});
    }

    await load();
  }

  function handleTouchStart(e, noteId) {
    swipeNoteId = noteId;
    swipeOffset = 0;
    swipeStartX = e.touches[0].clientX;
    swipeStartY = e.touches[0].clientY;
  }

  function handleTouchMove(e) {
    if (!swipeNoteId) return;
    const deltaX = e.touches[0].clientX - swipeStartX;
    const deltaY = e.touches[0].clientY - swipeStartY;
    // Only allow horizontal swipe if it's more horizontal than vertical
    if (Math.abs(deltaX) > Math.abs(deltaY)) {
      e.preventDefault();
      swipeOffset = deltaX;
    }
  }

  async function handleTouchEnd(note) {
    if (!swipeNoteId) return;
    if (swipeOffset < -SWIPE_THRESHOLD) {
      // Swipe left → delete
      await removeNote(note);
    } else if (swipeOffset > SWIPE_THRESHOLD && note.remind_at && !note.done) {
      // Swipe right → done (only for reminders that aren't already done)
      await markDone(note);
    }
    swipeNoteId = null;
    swipeOffset = 0;
  }
</script>

<main>
  <header>
    <h1>navinote</h1>
    <div class="header-actions">
      <button onclick={doSync} disabled={syncing}>
        {syncing ? 'syncing...' : 'sync'}
      </button>
      <button onclick={() => showSettings = !showSettings}>settings</button>
    </div>
  </header>

  {#if syncError}
    <div class="error">{syncError}</div>
  {/if}

  {#if showInstallPrompt}
    <div class="install-prompt">
      {#if installPlatform === 'ios'}
        <p>Install this app: tap <strong>Share</strong> then <strong>Add to Home Screen</strong></p>
      {:else if installPlatform === 'android' && deferredPrompt}
        <p>Install this app for quick access</p>
        <button onclick={installApp}>Install</button>
      {:else if installPlatform === 'android'}
        <p>Install this app: tap <strong>Menu</strong> then <strong>Add to Home Screen</strong></p>
      {/if}
      <button class="dismiss-btn" onclick={dismissInstall}>dismiss</button>
    </div>
  {/if}

  {#if showSettings}
    <div class="settings">
      <label>
        API Token
        <input type="password" bind:value={token} placeholder="Bearer token" />
      </label>
      <button onclick={saveToken}>Save</button>
    </div>
  {/if}

  <form onsubmit={(e) => { e.preventDefault(); addNote(); }}>
    <textarea
      bind:value={text}
      placeholder="Quick note..."
      rows="3"
      autofocus
    ></textarea>
    <div class="note-options">
      <label>
        <input type="checkbox" bind:checked={showReminder} onchange={() => { showCustomTime = false; remindAt = ''; }} />
        Remind me
      </label>
      {#if showReminder && remindAt}
        <span class="reminder-preview">{formatReminder(remindAt)}</span>
        <button type="button" class="clear-btn" onclick={() => { remindAt = ''; showCustomTime = false; }}>x</button>
      {/if}
    </div>
    {#if showReminder && !remindAt}
      <div class="quick-times">
        <button type="button" onclick={() => setReminder(20)}>20 min</button>
        <button type="button" onclick={() => setReminder(60)}>1 hour</button>
        <button type="button" onclick={() => setReminder(120)}>2 hours</button>
        <button type="button" onclick={() => setReminderAt(13)}>1pm today</button>
        <button type="button" onclick={() => setReminderAt(6, true)}>6am tomorrow</button>
        <button type="button" onclick={() => showCustomTime = true}>Custom...</button>
      </div>
    {/if}
    {#if showCustomTime}
      <input type="datetime-local" bind:value={remindAt} />
    {/if}
    <div class="form-actions">
      <button type="submit">{editingNote ? 'Update' : 'Save'}</button>
      {#if editingNote}
        <button type="button" class="cancel-btn" onclick={cancelEdit}>Cancel</button>
      {/if}
    </div>
  </form>

  <ul class="notes">
    {#each notes as note (note.id)}
      {@const reminderState = getReminderState(note)}
      <li
        class="{editingNote?.id === note.id ? 'editing' : ''} {reminderState ? `reminder-${reminderState}` : ''}"
        style={swipeNoteId === note.id ? `transform: translateX(${swipeOffset}px)` : ''}
        onclick={() => editNote(note)}
        ontouchstart={(e) => handleTouchStart(e, note.id)}
        ontouchmove={handleTouchMove}
        ontouchend={() => handleTouchEnd(note)}
      >
        {#if swipeNoteId === note.id && swipeOffset < -30}
          <div class="swipe-hint delete-hint">delete</div>
        {/if}
        {#if swipeNoteId === note.id && swipeOffset > 30 && note.remind_at && !note.done}
          <div class="swipe-hint done-hint">done</div>
        {/if}
        <div class="note-text">{note.text}</div>
        {#if note.remind_at}
          <div class="note-meta reminder-time">
            {#if note.done}
              <span class="done-badge">done</span>
            {/if}
            Reminder: {formatReminder(note.remind_at)}
          </div>
        {/if}
        <div class="note-meta">
          {formatDate(note.created_at)}
          {#if note._synced}
            <span class="synced-badge">synced</span>
          {/if}
        </div>
      </li>
    {/each}
  </ul>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #1a1a2e;
    color: #e0e0e0;
  }
  main {
    max-width: 600px;
    margin: 0 auto;
    padding: 1rem;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  h1 { margin: 0; font-size: 1.4rem; color: #7a9eb8; }
  .header-actions { display: flex; gap: 0.5rem; }
  .settings {
    background: #16213e;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }
  .settings label { display: block; margin-bottom: 0.5rem; }
  .settings input { width: 100%; box-sizing: border-box; }
  .install-prompt {
    background: #1e3a5f;
    border: 1px solid #4a6fa5;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  .install-prompt p {
    margin: 0;
    flex: 1;
    font-size: 0.9rem;
  }
  .dismiss-btn {
    background: transparent;
    color: #888;
    font-size: 0.8rem;
    padding: 0.25rem 0.5rem;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }
  input[type="text"], input[type="password"], input[type="datetime-local"], textarea {
    padding: 0.75rem;
    border: 1px solid #333;
    border-radius: 6px;
    background: #16213e;
    color: #e0e0e0;
    font-size: 1rem;
  }
  textarea {
    resize: vertical;
    font-family: inherit;
    min-height: 4.5rem;
  }
  .note-options {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .reminder-preview {
    font-size: 0.85rem;
    color: #7a9eb8;
  }
  .clear-btn {
    background: transparent;
    color: #888;
    padding: 0.2rem 0.5rem;
    font-size: 0.8rem;
  }
  .quick-times {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .quick-times button {
    background: #16213e;
    border: 1px solid #333;
    font-size: 0.8rem;
    padding: 0.4rem 0.6rem;
  }
  button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    background: #4a6fa5;
    color: #e0e0e0;
    cursor: pointer;
    font-size: 0.9rem;
  }
  button:disabled { opacity: 0.5; }
  .form-actions {
    display: flex;
    gap: 0.5rem;
  }
  .cancel-btn {
    background: transparent;
    border: 1px solid #4a6fa5;
  }
  .notes { list-style: none; padding: 0; }
  .notes li {
    background: #16213e;
    padding: 0.75rem;
    border-radius: 8px;
    margin-bottom: 0.5rem;
    cursor: pointer;
    transition: border-color 0.15s;
    border: 2px solid transparent;
  }
  .notes li:hover {
    border-color: #333;
  }
  .notes li.editing {
    border-color: #4a6fa5;
  }
  .note-text { margin-bottom: 0.25rem; white-space: pre-wrap; }
  .note-meta { font-size: 0.8rem; color: #888; }
  .reminder-time { color: #7a9eb8; }
  .synced-badge {
    background: #0f3460;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    font-size: 0.7rem;
  }
  .error {
    background: #3d2a2a;
    color: #d98a8a;
    padding: 0.75rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }
  /* Reminder state colors */
  .notes li.reminder-future {
    border-left: 3px solid #4a9e6a;
  }
  .notes li.reminder-overdue {
    border-left: 3px solid #c47a5a;
  }
  .notes li.reminder-done {
    opacity: 0.6;
    border-left: 3px solid #555;
  }
  .done-badge {
    background: #3a5a4a;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    font-size: 0.7rem;
    margin-right: 0.3rem;
  }
  /* Swipe styles */
  .notes li {
    position: relative;
    touch-action: pan-y;
    transition: transform 0.1s ease-out;
  }
  .swipe-hint {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    font-size: 0.8rem;
    font-weight: bold;
    text-transform: uppercase;
  }
  .delete-hint {
    right: 0.75rem;
    color: #c47a5a;
  }
  .done-hint {
    left: 0.75rem;
    color: #4a9e6a;
  }
</style>
