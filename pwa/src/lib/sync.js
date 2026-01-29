import { apiFetch } from './api.js';
import { getAllNotes, putNote, deleteNote } from './db.js';

export async function syncNotes() {
  const errors = [];

  const local = await getAllNotes();

  // Push deletions to server
  const deleted = local.filter(n => n._deleted && n._synced);
  for (const note of deleted) {
    try {
      await apiFetch(`/api/notes/${note.id}`, { method: 'DELETE' });
      await deleteNote(note.id);
    } catch (e) {
      // 404 means already deleted on server, remove locally
      if (e.message.includes('404')) {
        await deleteNote(note.id);
      } else {
        errors.push(e.message);
      }
    }
  }

  // Push new/updated notes to server
  const pending = local.filter(n => !n._synced && !n._deleted);
  for (const note of pending) {
    try {
      const saved = await apiFetch('/api/notes', {
        method: 'POST',
        body: JSON.stringify({ text: note.text, remind_at: note.remind_at || null }),
      });
      // Replace local with server version
      await deleteNote(note.id);
      await putNote({ ...saved, _synced: true });
    } catch (e) {
      errors.push(e.message);
    }
  }

  // Pull all from server (only non-deleted)
  try {
    const remote = await apiFetch('/api/notes');
    const localIds = new Set((await getAllNotes()).map(n => n.id));
    for (const note of remote) {
      await putNote({ ...note, _synced: true });
    }
    // Remove local notes that no longer exist on server (deleted from another device)
    const remoteIds = new Set(remote.map(n => n.id));
    for (const note of await getAllNotes()) {
      if (note._synced && !note._deleted && !remoteIds.has(note.id)) {
        await deleteNote(note.id);
      }
    }
  } catch (e) {
    errors.push(e.message);
  }

  if (errors.length > 0) {
    throw new Error(errors[0]);
  }
}
