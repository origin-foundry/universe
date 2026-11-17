import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
const listen = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn(), message: vi.fn() }));

import App from "./App";
import type { CalendarSummary } from "./calendarApi";
import { startingForm } from "./calendarFixtures";
import type { UniverseState } from "./fileFlows";

const open: UniverseState = { open: true, name: "Test", path: null, dirty: false };

/** A fake core holding a list of calendars. */
function core(calendars: CalendarSummary[]) {
  let state = open;
  invoke.mockImplementation(async (command: string, args?: { form?: { name: string } }) => {
    switch (command) {
      case "app_info":
        return { name: "Universe", version: "9.9.9" };
      case "universe_state":
        return state;
      case "list_calendars":
        return calendars;
      case "new_calendar_form":
        return startingForm();
      case "put_calendar": {
        state = { ...state, dirty: true };
        const id = calendars.length + 1;
        calendars = [...calendars, { id, name: args!.form!.name, problem: null }];
        return { state, id };
      }
      default:
        throw new Error(`unexpected command ${command}`);
    }
  });
}

beforeEach(() => listen.mockResolvedValue(() => undefined));
afterEach(() => {
  invoke.mockReset();
  listen.mockReset();
});

describe("CalendarList", () => {
  it("lists the open universe's calendars and offers Add calendar", async () => {
    core([
      { id: 1, name: "Aster", problem: null },
      { id: 2, name: "Tessel", problem: null },
    ]);
    render(<App />);
    const list = await screen.findByRole("list", { name: "Calendars" });
    expect(
      within(list)
        .getAllByRole("listitem")
        .map((li) => li.textContent),
    ).toEqual(["Aster", "Tessel"]);
    expect(screen.getByRole("button", { name: "Add calendar" })).toBeInTheDocument();
  });

  it("says when there are no calendars yet", async () => {
    core([]);
    render(<App />);
    expect(await screen.findByText("No calendars yet")).toBeInTheDocument();
  });

  it("opens the editor on the starting form, and lists the calendar once kept", async () => {
    core([]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Add calendar" }));
    expect(await screen.findByLabelText("Calendar name")).toHaveValue("Calendar");
    expect(screen.getByLabelText("Length, in SI seconds (physical)")).toHaveValue("");
    fireEvent.change(screen.getByLabelText("Calendar name"), { target: { value: "Aster" } });
    fireEvent.change(screen.getByLabelText("Length, in SI seconds (physical)"), {
      target: { value: "88775.244" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
    const list = await screen.findByRole("list", { name: "Calendars" });
    expect(within(list).getByText("Aster")).toBeInTheDocument();
    await waitFor(() => expect(document.title).toBe("Test • — Universe"));
  });

  it("shows a stored calendar's problem beside its name (FR-015)", async () => {
    core([{ id: 1, name: "Odd", problem: "rule 1 of orbit changes Thirteenth" }]);
    render(<App />);
    const item = await screen.findByRole("listitem");
    expect(item).toHaveTextContent("Odd");
    expect(
      within(item).getByText(/Can't be used: rule 1 of orbit changes Thirteenth/),
    ).toBeInTheDocument();
  });
});
