// Story 2 through the whole window: editing, discarding and deleting a stored calendar, and
// the keep-or-discard question in front of New, Open, Close and Exit (FR-012).

import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
const listen = vi.hoisted(() => vi.fn());
const dialog = vi.hoisted(() => ({ open: vi.fn(), save: vi.fn(), message: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/plugin-dialog", () => dialog);

import App from "./App";
import { asterForm } from "./calendarFixtures";
import type { UniverseState } from "./fileFlows";

const clean: UniverseState = { open: true, name: "Test", path: "C:\\t.universe", dirty: false };
const none: UniverseState = { open: false, name: null, path: null, dirty: false };
const QUESTION = "Keep or discard your changes to Aster?";

/** A fake core holding Aster as calendar 1 in a saved universe. */
function core() {
  let state = clean;
  let calendars = [{ id: 1, name: "Aster", problem: null }];
  invoke.mockImplementation(async (command: string) => {
    switch (command) {
      case "app_info":
        return { name: "Universe", version: "9.9.9" };
      case "universe_state":
        return state;
      case "list_calendars":
        return calendars;
      case "get_calendar":
        return { form: asterForm(), rounded: false, note: null };
      case "calendar_followers":
        return [];
      case "put_calendar":
        state = { ...state, dirty: true };
        return { state, id: 1 };
      case "delete_calendar":
        state = { ...state, dirty: true };
        calendars = [];
        return state;
      case "close_universe":
        state = none;
        return state;
      case "exit_app":
        return undefined;
      default:
        throw new Error(`unexpected command ${command}`);
    }
  });
}

const handlers = new Map<string, (event: { payload: unknown }) => void>();

beforeEach(() => {
  listen.mockImplementation(
    async (name: string, handler: (event: { payload: unknown }) => void) => {
      handlers.set(name, handler);
      return () => handlers.delete(name);
    },
  );
  core();
});

afterEach(() => {
  invoke.mockReset();
  listen.mockReset();
  Object.values(dialog).forEach((f) => f.mockReset());
  handlers.clear();
});

const commands = () => invoke.mock.calls.map((c) => c[0] as string);

async function openAster() {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Aster" }));
  await screen.findByLabelText("Calendar name");
}

function changeACount() {
  fireEvent.change(screen.getByLabelText("Grouping 3 member 1 count"), {
    target: { value: "57" },
  });
}

describe("Editing a stored calendar", () => {
  it("opens its stored form", async () => {
    await openAster();
    expect(invoke).toHaveBeenCalledWith("get_calendar", { id: 1 });
    expect(screen.getByLabelText("Calendar name")).toHaveValue("Aster");
    expect(screen.getByLabelText("Grouping 3 rule 1 member")).toHaveValue("Twelfth");
    expect(screen.getByRole("button", { name: "Delete calendar" })).toBeInTheDocument();
  });

  it("keeps a changed count under the calendar's id", async () => {
    await openAster();
    changeACount();
    fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
    await screen.findByRole("list", { name: "Calendars" });
    const form = asterForm();
    form.groupings[2]!.members[0]!.count = 57;
    expect(invoke).toHaveBeenCalledWith("put_calendar", { id: 1, form });
    await waitFor(() => expect(document.title).toBe("Test • — Universe"));
  });

  it("discards an edit without calling the core", async () => {
    await openAster();
    changeACount();
    const before = invoke.mock.calls.length;
    fireEvent.click(screen.getByRole("button", { name: "Discard changes" }));
    expect(await screen.findByRole("list", { name: "Calendars" })).toBeInTheDocument();
    expect(invoke.mock.calls.slice(before).map((c) => c[0])).not.toContain("put_calendar");
    await waitFor(() => expect(document.title).toBe("Test — Universe"));
  });

  it("deletes only after the owner confirms", async () => {
    await openAster();
    dialog.message.mockResolvedValueOnce("Cancel");
    fireEvent.click(screen.getByRole("button", { name: "Delete calendar" }));
    await waitFor(() => expect(dialog.message).toHaveBeenCalledTimes(1));
    expect(dialog.message.mock.calls[0]![0]).toBe("Delete Aster?");
    expect(commands()).not.toContain("delete_calendar");

    dialog.message.mockResolvedValueOnce("Delete");
    fireEvent.click(screen.getByRole("button", { name: "Delete calendar" }));
    expect(await screen.findByText("No calendars yet")).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("delete_calendar", { id: 1 });
  });
});

describe("The keep-or-discard question (FR-012)", () => {
  it("comes before closing from the File menu, and Discard then closes", async () => {
    await openAster();
    changeACount();
    dialog.message.mockResolvedValue("Discard");
    act(() => handlers.get("menu")!({ payload: "close" }));
    expect(await screen.findByText("No universe open")).toBeInTheDocument();
    expect(dialog.message).toHaveBeenCalledTimes(1);
    expect(dialog.message.mock.calls[0]![0]).toBe(QUESTION);
    expect(commands()).toContain("close_universe");
    expect(commands()).not.toContain("put_calendar");
  });

  it("keeps the edit, then asks about the unsaved universe", async () => {
    await openAster();
    changeACount();
    dialog.message.mockResolvedValueOnce("Keep").mockResolvedValueOnce("Cancel");
    act(() => handlers.get("menu")!({ payload: "new" }));
    await waitFor(() => expect(dialog.message).toHaveBeenCalledTimes(2));
    expect(dialog.message.mock.calls.map((c) => c[0])).toEqual([QUESTION, "Save changes to Test?"]);
    expect(commands()).toContain("put_calendar");
  });

  it("stops when the kept edit is refused, showing why", async () => {
    await openAster();
    invoke.mockImplementationOnce(async () => {
      throw { message: "orbit has no members", at: { kind: "grouping", index: 2 } };
    });
    changeACount();
    dialog.message.mockResolvedValue("Keep");
    act(() => handlers.get("menu")!({ payload: "open" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("orbit has no members");
    expect(dialog.message).toHaveBeenCalledTimes(1);
    expect(dialog.open).not.toHaveBeenCalled();
  });

  it("comes before Ctrl+W, which the native menu never sees", async () => {
    await openAster();
    changeACount();
    dialog.message.mockResolvedValue("Cancel");
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "w", ctrlKey: true }));
    });
    await waitFor(() => expect(dialog.message).toHaveBeenCalledWith(QUESTION, expect.anything()));
    expect(commands()).not.toContain("close_universe");
  });

  it("comes before the window's close button and File → Exit", async () => {
    await openAster();
    changeACount();
    dialog.message.mockResolvedValue("Discard");
    act(() => handlers.get("close-requested")!({ payload: null }));
    await waitFor(() => expect(commands()).toContain("exit_app"));
    expect(dialog.message.mock.calls[0]![0]).toBe(QUESTION);
  });

  it("is not asked with no changes", async () => {
    await openAster();
    act(() => handlers.get("menu")!({ payload: "close" }));
    expect(await screen.findByText("No universe open")).toBeInTheDocument();
    expect(dialog.message).not.toHaveBeenCalled();
  });
});
