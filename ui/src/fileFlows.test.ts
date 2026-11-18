import { afterEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
const dialog = vi.hoisted(() => ({ open: vi.fn(), save: vi.fn(), message: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => dialog);

import {
  type PendingEdit,
  type UniverseState,
  closeFlow,
  closeWindowFlow,
  newFlow,
  openFlow,
  saveFlow,
  UNIVERSE_FILTER,
} from "./fileFlows";

const none: UniverseState = { open: false, name: null, path: null, dirty: false };
const clean: UniverseState = {
  open: true,
  name: "Aster",
  path: "C:\\a\\Aster.universe",
  dirty: false,
};
const untitled: UniverseState = { open: true, name: "Aster", path: null, dirty: true };
const dirty: UniverseState = { ...clean, dirty: true };

const MISSING =
  "C:\\a\\Aster.universe has been moved or deleted. Use Save As to save it somewhere.";

afterEach(() => {
  invoke.mockReset();
  Object.values(dialog).forEach((f) => f.mockReset());
});

describe("New", () => {
  it("asks nothing when the open universe has no unsaved changes", async () => {
    expect(await newFlow(clean)).toBe(true);
    expect(await newFlow(none)).toBe(true);
    expect(dialog.message).not.toHaveBeenCalled();
  });

  it("asks Save / Don't save / Cancel when there are unsaved changes, and Cancel stops", async () => {
    dialog.message.mockResolvedValue("Cancel");
    expect(await newFlow(dirty)).toBe(false);
    const [, options] = dialog.message.mock.calls[0]!;
    expect(options.buttons).toEqual({ yes: "Save", no: "Don't save", cancel: "Cancel" });
    expect(invoke).not.toHaveBeenCalled();
  });

  it("goes on without saving after Don't save", async () => {
    dialog.message.mockResolvedValue("Don't save");
    expect(await newFlow(dirty)).toBe(true);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("saves first after Save", async () => {
    dialog.message.mockResolvedValue("Save");
    invoke.mockResolvedValue(clean);
    expect(await newFlow(dirty)).toBe(true);
    expect(invoke).toHaveBeenCalledWith("save_universe");
  });
});

describe("Save", () => {
  it("opens the Save dialog for an untitled universe, suggesting its name", async () => {
    dialog.save.mockResolvedValue("C:\\a\\Aster.universe");
    invoke.mockResolvedValue(clean);
    expect(await saveFlow(untitled)).toEqual(clean);
    expect(dialog.save).toHaveBeenCalledWith({
      defaultPath: "Aster.universe",
      filters: [UNIVERSE_FILTER],
    });
    expect(UNIVERSE_FILTER.extensions).toEqual(["universe"]);
    expect(invoke).toHaveBeenCalledWith("save_universe_as", { path: "C:\\a\\Aster.universe" });
  });

  it("adds .universe when the chosen name has no extension", async () => {
    dialog.save.mockResolvedValue("C:\\a\\Aster");
    invoke.mockResolvedValue(clean);
    await saveFlow(untitled);
    expect(invoke).toHaveBeenCalledWith("save_universe_as", { path: "C:\\a\\Aster.universe" });
  });

  it("changes nothing when the dialog is cancelled", async () => {
    dialog.save.mockResolvedValue(null);
    expect(await saveFlow(untitled)).toBeNull();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("saves a universe that has a file without asking", async () => {
    invoke.mockResolvedValue(clean);
    expect(await saveFlow(dirty)).toEqual(clean);
    expect(dialog.save).not.toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith("save_universe");
  });

  it("shows the owner the store's message when saving fails", async () => {
    invoke.mockRejectedValue("Couldn't write C:\\a\\Aster.universe: disk full.");
    expect(await saveFlow(dirty)).toBeNull();
    expect(dialog.message).toHaveBeenCalledWith(
      "Couldn't write C:\\a\\Aster.universe: disk full.",
      expect.objectContaining({ kind: "error" }),
    );
  });

  it("offers Save As when the file has been moved or deleted", async () => {
    invoke.mockRejectedValueOnce(MISSING).mockResolvedValueOnce(clean);
    dialog.message.mockResolvedValue("Save As…");
    dialog.save.mockResolvedValue("C:\\b\\Aster.universe");
    expect(await saveFlow(dirty)).toEqual(clean);
    expect(dialog.message.mock.calls[0]?.[0]).toBe(MISSING);
    expect(invoke).toHaveBeenLastCalledWith("save_universe_as", { path: "C:\\b\\Aster.universe" });
  });
});

describe("Open", () => {
  it("opens the chosen file, filtered to .universe", async () => {
    dialog.open.mockResolvedValue("C:\\a\\Aster.universe");
    invoke.mockResolvedValue(clean);
    expect(await openFlow(none)).toEqual(clean);
    expect(dialog.open).toHaveBeenCalledWith(
      expect.objectContaining({ multiple: false, directory: false, filters: [UNIVERSE_FILTER] }),
    );
    expect(invoke).toHaveBeenCalledWith("open_universe", { path: "C:\\a\\Aster.universe" });
  });

  it("changes nothing when the dialog is cancelled", async () => {
    dialog.open.mockResolvedValue(null);
    expect(await openFlow(none)).toBeNull();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("shows why a file was refused", async () => {
    dialog.open.mockResolvedValue("C:\\a\\notes.universe");
    invoke.mockRejectedValue("This isn't a Universe project file (it isn't a database file).");
    expect(await openFlow(none)).toBeNull();
    expect(dialog.message).toHaveBeenCalledWith(
      "This isn't a Universe project file (it isn't a database file).",
      expect.objectContaining({ kind: "error" }),
    );
  });
});

describe("Closing the window", () => {
  it("does nothing after Cancel", async () => {
    dialog.message.mockResolvedValue("Cancel");
    await closeWindowFlow(dirty);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("exits without saving after Don't save", async () => {
    dialog.message.mockResolvedValue("Don't save");
    await closeWindowFlow(dirty);
    expect(invoke).toHaveBeenCalledWith("exit_app");
    expect(invoke).not.toHaveBeenCalledWith("save_universe");
  });

  it("stays open if saving fails", async () => {
    dialog.message.mockResolvedValueOnce("Save").mockResolvedValue("Ok");
    invoke.mockRejectedValue("Couldn't write C:\\a\\Aster.universe: disk full.");
    await closeWindowFlow(dirty);
    expect(invoke).not.toHaveBeenCalledWith("exit_app");
  });
});

describe("A calendar edit in progress (FR-012)", () => {
  /** An edit of Aster whose Keep gives `kept` (null: the core refused it). */
  function pending(kept: UniverseState | null): PendingEdit {
    return { calendar: "Aster", keep: vi.fn(async () => kept), discard: vi.fn() };
  }
  const QUESTION = "Keep or discard your changes to Aster?";

  it("is asked about first, with Keep / Discard / Cancel", async () => {
    dialog.message.mockResolvedValue("Cancel");
    const edit = pending(dirty);
    expect(await closeFlow(clean, edit)).toBeNull();
    expect(dialog.message).toHaveBeenCalledTimes(1);
    const [text, options] = dialog.message.mock.calls[0]!;
    expect(text).toBe(QUESTION);
    expect(options.buttons).toEqual({ yes: "Keep", no: "Discard", cancel: "Cancel" });
    expect(edit.keep).not.toHaveBeenCalled();
    expect(edit.discard).not.toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("is kept, then the unsaved universe is asked about with the kept state", async () => {
    dialog.message.mockResolvedValueOnce("Keep").mockResolvedValueOnce("Cancel");
    const edit = pending(dirty);
    expect(await closeFlow(clean, edit)).toBeNull();
    expect(edit.keep).toHaveBeenCalled();
    expect(dialog.message).toHaveBeenCalledTimes(2);
    expect(dialog.message.mock.calls[1]![0]).toBe("Save changes to Aster?");
  });

  it("stops the flow when keeping is refused", async () => {
    dialog.message.mockResolvedValue("Keep");
    const edit = pending(null);
    expect(await closeFlow(dirty, edit)).toBeNull();
    expect(dialog.message).toHaveBeenCalledTimes(1);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("is discarded, then the flow goes on as before", async () => {
    dialog.message.mockResolvedValue("Discard");
    invoke.mockResolvedValue(none);
    const edit = pending(dirty);
    expect(await closeFlow(clean, edit)).toEqual(none);
    expect(edit.discard).toHaveBeenCalled();
    expect(edit.keep).not.toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith("close_universe");
  });

  it("is asked about by New, Open and the window's close button too", async () => {
    dialog.message.mockResolvedValue("Cancel");
    expect(await newFlow(clean, pending(dirty))).toBe(false);
    expect(await openFlow(clean, pending(dirty))).toBeNull();
    await closeWindowFlow(clean, pending(dirty));
    expect(dialog.message.mock.calls.map((c) => c[0])).toEqual([QUESTION, QUESTION, QUESTION]);
    expect(dialog.open).not.toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("lets the window close once discarded, with nothing else unsaved", async () => {
    dialog.message.mockResolvedValue("Discard");
    await closeWindowFlow(clean, pending(dirty));
    expect(invoke).toHaveBeenCalledWith("exit_app");
  });

  it("isn't asked about when there is none", async () => {
    invoke.mockResolvedValue(none);
    expect(await closeFlow(clean, null)).toEqual(none);
    await closeWindowFlow(clean);
    expect(dialog.message).not.toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith("exit_app");
  });
});
