import { describe, expect, it } from "vitest";

import { shortcutFor } from "./shortcuts";

function key(
  k: string,
  mods: { ctrl?: boolean; shift?: boolean; alt?: boolean; meta?: boolean } = {},
) {
  return {
    key: k,
    ctrlKey: mods.ctrl ?? false,
    shiftKey: mods.shift ?? false,
    altKey: mods.alt ?? false,
    metaKey: mods.meta ?? false,
  };
}

describe("shortcutFor", () => {
  it("maps the File menu's shortcuts to its items", () => {
    expect(shortcutFor(key("n", { ctrl: true }))).toBe("new");
    expect(shortcutFor(key("o", { ctrl: true }))).toBe("open");
    expect(shortcutFor(key("s", { ctrl: true }))).toBe("save");
    expect(shortcutFor(key("S", { ctrl: true, shift: true }))).toBe("save-as");
    expect(shortcutFor(key("w", { ctrl: true }))).toBe("close");
  });

  it("works with Caps Lock on", () => {
    expect(shortcutFor(key("N", { ctrl: true }))).toBe("new");
    expect(shortcutFor(key("s", { ctrl: true, shift: true }))).toBe("save-as");
  });

  it("ignores other keys and modifier combinations", () => {
    expect(shortcutFor(key("n"))).toBeNull();
    expect(shortcutFor(key("x", { ctrl: true }))).toBeNull();
    expect(shortcutFor(key("n", { ctrl: true, alt: true }))).toBeNull();
    expect(shortcutFor(key("n", { ctrl: true, shift: true }))).toBeNull();
    expect(shortcutFor(key("w", { ctrl: true, shift: true }))).toBeNull();
  });
});
