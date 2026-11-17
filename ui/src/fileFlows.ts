// The New, Open, Save, Save As and Close flows, shared by the File menu and the buttons
// (specs/003-project-file/research.md, R8). Each flow returns the new state, or null when
// the owner cancelled or something failed (and they've been told why).

import { invoke } from "@tauri-apps/api/core";
import { message, open, save } from "@tauri-apps/plugin-dialog";

/** Matches `UniverseState` in crates/uni-app/src/lib.rs. */
export interface UniverseState {
  open: boolean;
  name: string | null;
  path: string | null;
  dirty: boolean;
}

/** A calendar edit in progress, which New, Open, Close and Exit ask about first (FR-012). */
export interface PendingEdit {
  /** The calendar's name, for the question. */
  calendar: string;
  /** Keeps the edit: the universe's new state, or null if the core refused it. */
  keep: () => Promise<UniverseState | null>;
  discard: () => void;
}

export const NO_UNIVERSE: UniverseState = { open: false, name: null, path: null, dirty: false };

export const UNIVERSE_FILTER = { name: "Universe project", extensions: ["universe"] };

const SAVE = "Save";
const DONT_SAVE = "Don't save";
const CANCEL = "Cancel";
const SAVE_AS = "Save As…";
const KEEP = "Keep";
const DISCARD = "Discard";

/** The window title: `Aster — Universe`, with `•` while there are unsaved changes. */
export function windowTitle(state: UniverseState): string {
  if (!state.open || state.name === null) return "Universe";
  return `${state.name}${state.dirty ? " •" : ""} — Universe`;
}

async function showError(error: unknown): Promise<void> {
  await message(String(error), { title: "Universe", kind: "error" });
}

/** A file name for `name`, without the characters Windows doesn't allow. */
function suggestedFileName(name: string | null): string {
  const safe = (name ?? "Untitled").replace(/[<>:"/\\|?*]/g, "-");
  return `${safe}.universe`;
}

function withExtension(path: string): string {
  return path.toLowerCase().endsWith(".universe") ? path : `${path}.universe`;
}

/** Asks where to save, then saves there. */
export async function saveAsFlow(state: UniverseState): Promise<UniverseState | null> {
  const chosen = await save({
    defaultPath: suggestedFileName(state.name),
    filters: [UNIVERSE_FILTER],
  });
  if (chosen === null) return null;
  try {
    return await invoke<UniverseState>("save_universe_as", { path: withExtension(chosen) });
  } catch (error) {
    await showError(error);
    return null;
  }
}

/** Saves to the universe's file, or asks where if it has none yet. */
export async function saveFlow(state: UniverseState): Promise<UniverseState | null> {
  if (state.path === null) return saveAsFlow(state);
  try {
    return await invoke<UniverseState>("save_universe");
  } catch (error) {
    if (String(error).includes("has been moved or deleted")) {
      const answer = await message(String(error), {
        title: "Universe",
        kind: "warning",
        buttons: { ok: SAVE_AS, cancel: CANCEL },
      });
      return answer === SAVE_AS || answer === "Ok" ? saveAsFlow(state) : null;
    }
    await showError(error);
    return null;
  }
}

/**
 * If there are unsaved changes, asks Save / Don't save / Cancel. Resolves to whether the
 * caller may go on to replace or close the universe.
 */
export async function confirmLeave(state: UniverseState): Promise<boolean> {
  if (!state.open || !state.dirty) return true;
  const answer = await message(`Save changes to ${state.name ?? "this universe"}?`, {
    title: "Universe",
    kind: "warning",
    buttons: { yes: SAVE, no: DONT_SAVE, cancel: CANCEL },
  });
  if (answer === SAVE || answer === "Yes") return (await saveFlow(state)) !== null;
  return answer === DONT_SAVE || answer === "No";
}

/**
 * If a calendar edit is in progress, asks Keep / Discard / Cancel about it first (FR-012).
 * Resolves to the state to carry on with (after Keep, the universe has the kept calendar),
 * or null to stop: after Cancel, or when keeping is refused and the editor shows why.
 */
export async function confirmPendingEdit(
  state: UniverseState,
  pending: PendingEdit | null,
): Promise<UniverseState | null> {
  if (pending === null) return state;
  const answer = await message(`Keep or discard your changes to ${pending.calendar}?`, {
    title: "Universe",
    kind: "warning",
    buttons: { yes: KEEP, no: DISCARD, cancel: CANCEL },
  });
  if (answer === KEEP || answer === "Yes") return pending.keep();
  if (answer === DISCARD || answer === "No") {
    pending.discard();
    return state;
  }
  return null;
}

/** Both questions in order: the calendar edit, then the unsaved universe. */
async function confirmAll(state: UniverseState, pending: PendingEdit | null): Promise<boolean> {
  const next = await confirmPendingEdit(state, pending);
  return next !== null && confirmLeave(next);
}

/** New: resolves to whether to show the name form. */
export async function newFlow(
  state: UniverseState,
  pending: PendingEdit | null = null,
): Promise<boolean> {
  return confirmAll(state, pending);
}

/** Creates the universe; rejects with the core's message if the name is refused. */
export async function createUniverse(name: string): Promise<UniverseState> {
  return invoke<UniverseState>("new_universe", { name });
}

/** Open: asks about unsaved changes, then for a file, then opens it. */
export async function openFlow(
  state: UniverseState,
  pending: PendingEdit | null = null,
): Promise<UniverseState | null> {
  if (!(await confirmAll(state, pending))) return null;
  const chosen = await open({ multiple: false, directory: false, filters: [UNIVERSE_FILTER] });
  if (chosen === null) return null;
  try {
    return await invoke<UniverseState>("open_universe", { path: chosen });
  } catch (error) {
    await showError(error);
    return null;
  }
}

/** Close: asks about unsaved changes, then returns to no universe. */
export async function closeFlow(
  state: UniverseState,
  pending: PendingEdit | null = null,
): Promise<UniverseState | null> {
  if (!(await confirmAll(state, pending))) return null;
  return invoke<UniverseState>("close_universe");
}

/** The window's ✕ (or File → Exit): ask what needs asking, then exit unless cancelled. */
export async function closeWindowFlow(
  state: UniverseState,
  pending: PendingEdit | null = null,
): Promise<void> {
  if (await confirmAll(state, pending)) await invoke("exit_app");
}
