// The File menu's keyboard shortcuts, handled in the UI.
//
// On Windows the keyboard focus is inside the web view (WebView2), which runs as its own
// child window, so key presses never reach the native menu's accelerators: the menu shows
// Ctrl+N and so on, but only the UI ever sees them. Keep this table in step with
// `build_menu` in crates/uni-app/src/lib.rs.

/** The File menu items, as sent in `menu` events. */
export type MenuAction = "new" | "open" | "save" | "save-as" | "close";

/** The parts of a `KeyboardEvent` a shortcut depends on. */
export interface KeyPress {
  key: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}

const SHORTCUTS: Record<string, MenuAction> = {
  n: "new",
  o: "open",
  s: "save",
  "shift+s": "save-as",
  w: "close",
};

/** The File menu item a key press stands for, or null. */
export function shortcutFor(press: KeyPress): MenuAction | null {
  if (!press.ctrlKey || press.altKey || press.metaKey) return null;
  const combo = `${press.shiftKey ? "shift+" : ""}${press.key.toLowerCase()}`;
  return SHORTCUTS[combo] ?? null;
}

/** Items that, as in the menu, do nothing while no universe is open. */
export const NEEDS_OPEN_UNIVERSE: ReadonlySet<MenuAction> = new Set(["save", "save-as", "close"]);
