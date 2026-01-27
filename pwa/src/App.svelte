<script>
  import { onMount } from 'svelte';
  import { getAllNotes, putNote, deleteNote } from './lib/db.js';
  import { syncNotes } from './lib/sync.js';
  import { getToken, setToken } from './lib/api.js';

  let notes = $state([]);
  let text = $state('');
  let remindAt = $state('');
  let showReminder = $state(false);
  let syncing = $state(false);
  let token = $state(getToken());
  let showSettings = $state(!getToken());

  async function load() {
    notes = await getAllNotes();
  }

  onMount(load);

  async function addNote() {
    if (!text.trim()) return;
    const note = {
      id: crypto.randomUUID(),
      text: text.trim(),
      remind_at: showReminder && remindAt ? remindAt : null,
      synced: false,
      _synced: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    await putNote(note);
    text = '';
    remindAt = '';
    showReminder = false;
    await load();
    doSync();
  }

  async function removeNote(id) {
    await deleteNote(id);
    await load();
  }

  async function doSync() {
    syncing = true;
    try {
      await syncNotes();
      await load();
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
    <input
      type="text"
      bind:value={text}
      placeholder="Quick note..."
      autofocus
    />
    <div class="note-options">
      <label>
        <input type="checkbox" bind:checked={showReminder} />
        Remind me
      </label>
      {#if showReminder}
        <input type="datetime-local" bind:value={remindAt} />
      {/if}
    </div>
    <button type="submit">Save</button>
  </form>

  <ul class="notes">
    {#each notes as note (note.id)}
      <li>
        <div class="note-text">{note.text}</div>
        {#if note.remind_at}
          <div class="note-meta">Reminder: {formatDate(note.remind_at)}</div>
        {/if}
        <div class="note-meta">
          {formatDate(note.created_at)}
          {#if note._synced}
            <span class="synced-badge">synced</span>
          {/if}
        </div>
        <button class="delete-btn" onclick={() => removeNote(note.id)}>delete</button>
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
  h1 { margin: 0; font-size: 1.4rem; color: #e94560; }
  .header-actions { display: flex; gap: 0.5rem; }
  .settings {
    background: #16213e;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }
  .settings label { display: block; margin-bottom: 0.5rem; }
  .settings input { width: 100%; box-sizing: border-box; }
  form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }
  input[type="text"], input[type="password"], input[type="datetime-local"] {
    padding: 0.75rem;
    border: 1px solid #333;
    border-radius: 6px;
    background: #16213e;
    color: #e0e0e0;
    font-size: 1rem;
  }
  .note-options {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    background: #e94560;
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
  }
  button:disabled { opacity: 0.5; }
  .notes { list-style: none; padding: 0; }
  .notes li {
    background: #16213e;
    padding: 0.75rem;
    border-radius: 8px;
    margin-bottom: 0.5rem;
  }
  .note-text { margin-bottom: 0.25rem; }
  .note-meta { font-size: 0.8rem; color: #888; }
  .synced-badge {
    background: #0f3460;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    font-size: 0.7rem;
  }
  .delete-btn {
    background: transparent;
    color: #888;
    font-size: 0.8rem;
    padding: 0.2rem 0.5rem;
    margin-top: 0.25rem;
  }
</style>
