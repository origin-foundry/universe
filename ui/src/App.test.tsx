import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
const listen = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn(), message: vi.fn() }));

import App from "./App";
import type { UniverseState } from "./fileFlows";

const none: UniverseState = { open: false, name: null, path: null, dirty: false };

/** Routes `invoke` by command; `state` is what the core reports. */
function core(state: UniverseState, info: unknown = { name: "Universe", version: "9.9.9" }) {
  invoke.mockImplementation(async (command: string, args?: { name?: string }) => {
    switch (command) {
      case "app_info":
        if (info instanceof Error) throw info;
        return info;
      case "universe_state":
        return state;
      case "new_universe":
        return { open: true, name: args?.name ?? null, path: null, dirty: true };
      default:
        throw new Error(`unexpected command ${command}`);
    }
  });
}

/** Handlers the App registered with `listen`, by event name. */
const handlers = new Map<string, (event: { payload: unknown }) => void>();

describe("App", () => {
  beforeEach(() => {
    listen.mockImplementation(
      async (name: string, handler: (event: { payload: unknown }) => void) => {
        handlers.set(name, handler);
        return () => handlers.delete(name);
      },
    );
  });

  afterEach(() => {
    invoke.mockReset();
    listen.mockReset();
    handlers.clear();
  });

  it("shows the name, the version and the empty screen's buttons", async () => {
    core(none);
    render(<App />);
    expect(await screen.findByText("9.9.9")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Universe" })).toBeInTheDocument();
    expect(screen.getByText("No universe open")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "New universe" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open universe…" })).toBeInTheDocument();
    expect(document.title).toBe("Universe");
  });

  it("says the version is unavailable if the core can't be reached", async () => {
    core(none, new Error("no core"));
    render(<App />);
    expect(await screen.findByText("Version unavailable")).toBeInTheDocument();
    expect(screen.getByText("No universe open")).toBeInTheDocument();
  });

  it("shows an open universe's name in the window and the title", async () => {
    core({ open: true, name: "Aster", path: "C:\\a\\Aster.universe", dirty: false });
    render(<App />);
    expect(await screen.findByRole("heading", { name: "Aster" })).toBeInTheDocument();
    expect(screen.queryByText("No universe open")).not.toBeInTheDocument();
    expect(document.title).toBe("Aster — Universe");
  });

  it("marks unsaved changes in the title", async () => {
    core({ open: true, name: "Aster", path: null, dirty: true });
    render(<App />);
    await waitFor(() => expect(document.title).toBe("Aster • — Universe"));
  });

  it("creates a universe from the New button", async () => {
    core(none);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "New universe" }));
    fireEvent.change(await screen.findByLabelText("Name"), { target: { value: "Aster" } });
    fireEvent.click(screen.getByRole("button", { name: "Create" }));
    expect(await screen.findByRole("heading", { name: "Aster" })).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("new_universe", { name: "Aster" });
    expect(document.title).toBe("Aster • — Universe");
  });

  it("opens the name form with Ctrl+N, which the native menu never sees", async () => {
    core(none);
    render(<App />);
    await screen.findByText("9.9.9");
    const event = new KeyboardEvent("keydown", { key: "n", ctrlKey: true, cancelable: true });
    act(() => {
      window.dispatchEvent(event);
    });
    expect(await screen.findByLabelText("Name")).toBeInTheDocument();
    expect(event.defaultPrevented).toBe(true);
  });

  it("ignores Ctrl+S while no universe is open, as the menu does", async () => {
    core(none);
    render(<App />);
    await screen.findByText("9.9.9");
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "s", ctrlKey: true }));
    });
    await Promise.resolve();
    expect(invoke).not.toHaveBeenCalledWith("save_universe");
  });

  it("opens the name form from the File menu's New", async () => {
    core(none);
    render(<App />);
    await waitFor(() => expect(handlers.has("menu")).toBe(true));
    act(() => handlers.get("menu")!({ payload: "new" }));
    expect(await screen.findByLabelText("Name")).toBeInTheDocument();
  });
});
