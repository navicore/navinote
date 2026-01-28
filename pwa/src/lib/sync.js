import { apiFetch } from './api.js';
import { getAllNotes, putNote, deleteNote } from './db.js';

export async function syncNotes() {
  const errors = [];

  // Push local notes to server
  const local = await getAllNotes();
  const pending = local.filter(n => !n._synced);

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

  // Pull all from server
  try {
    const remote = await apiFetch('/api/notes');
    for (const note of remote) {
      await putNote({ ...note, _synced: true });
    }
  } catch (e) {
    errors.push(e.message);
  }

  if (errors.length > 0) {
    throw new Error(errors[0]);
  }
}
