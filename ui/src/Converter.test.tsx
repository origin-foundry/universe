import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
const listen = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn(), message: vi.fn() }));

import App from "./App";
import type { CalendarSummary, Conversion, ConverterCalendar } from "./calendarApi";
import { NO_OTHER, ROUNDED } from "./Converter";
import type { UniverseState } from "./fileFlows";

const open: UniverseState = { open: true, name: "Test", path: null, dirty: false };

const v = (unit: string, value: number, name: string | null = null) => ({ unit, value, name });

const ASTER: ConverterCalendar = {
  id: 1,
  name: "Aster",
  units: ["orbit", "span", "sol", "bell", "pulse"],
  problem: null,
};
const TALLOW: ConverterCalendar = {
  id: 2,
  name: "Tallow",
  units: ["cycle", "round", "watch"],
  problem: null,
};

/** Aster as the converter gives it for Tallow's cycle 1, round 1, watch 1. */
const IN_ASTER: Conversion = {
  rounded: false,
  results: [
    {
      id: 1,
      name: "Aster",
      date: {
        above: [v("age", 5), v("cohort", 42)],
        values: [v("orbit", 412), v("span", 3, "Third"), v("sol", 9), v("bell", 0), v("pulse", 0)],
        partway: null,
      },
    },
  ],
  unusable: [],
};

/** Tallow as the converter gives it for an Aster date partway through a watch. */
const IN_TALLOW: Conversion = {
  rounded: false,
  results: [
    {
      id: 2,
      name: "Tallow",
      date: {
        above: [],
        values: [v("cycle", 1), v("round", 1), v("watch", 2)],
        partway: { decimal: "0.035", decimalExact: false, exact: "1785559/50000000" },
      },
    },
  ],
  unusable: [{ name: "Wick", problem: "it's anchored to Moss, which can't be placed" }],
};

interface Core {
  calendars?: CalendarSummary[];
  converter?: ConverterCalendar[];
  conversion?: (from: number, date: (number | null)[]) => unknown;
}

function core({
  calendars = [
    { id: 1, name: "Aster", problem: null },
    { id: 2, name: "Tallow", problem: null },
  ],
  converter = [ASTER, TALLOW],
  conversion = (from) => (from === 2 ? IN_ASTER : IN_TALLOW),
}: Core = {}) {
  invoke.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
    switch (command) {
      case "app_info":
        return { name: "Universe", version: "9.9.9" };
      case "universe_state":
        return open;
      case "list_calendars":
        return calendars;
      case "converter_calendars":
        return converter;
      case "convert":
        return conversion(args!.from as number, args!.date as (number | null)[]);
      default:
        throw new Error(`unexpected command ${command}`);
    }
  });
}

const converterButton = () => screen.findByRole("button", { name: "Converter" });
const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
const results = () => screen.findByRole("region", { name: "Results" });
const converts = () => invoke.mock.calls.filter((c) => c[0] === "convert");

/** The Converter button, once the calendar list has loaded and enabled it. */
async function enabledConverterButton() {
  const button = await converterButton();
  await waitFor(() => expect(button).toBeEnabled());
  return button;
}

async function openConverter() {
  render(<App />);
  fireEvent.click(await enabledConverterButton());
  await screen.findByLabelText("Calendar");
}

beforeEach(() => listen.mockResolvedValue(() => undefined));
afterEach(() => {
  invoke.mockReset();
  listen.mockReset();
});

describe("Converter", () => {
  it("is offered once a calendar can be placed, and goes back to the list (FR-008)", async () => {
    core({ calendars: [{ id: 1, name: "Odd", problem: "rule 1 of orbit changes Thirteenth" }] });
    render(<App />);
    await screen.findByRole("list", { name: "Calendars" });
    expect(await converterButton()).toBeDisabled();
    cleanup();
    core();
    await openConverter();
    expect(screen.getByRole("heading", { name: "Converter" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Back to calendars" }));
    expect(await screen.findByRole("list", { name: "Calendars" })).toBeInTheDocument();
  });

  it("shows one field per unit of the chosen calendar", async () => {
    core();
    await openConverter();
    for (const unit of ASTER.units) expect(screen.getByLabelText(unit)).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Calendar"), { target: { value: "2" } });
    for (const unit of TALLOW.units) expect(screen.getByLabelText(unit)).toHaveValue("");
    expect(screen.queryByLabelText("orbit")).not.toBeInTheDocument();
  });

  it("converts as the owner types, naming units and members (FR-009)", async () => {
    core();
    await openConverter();
    fireEvent.change(screen.getByLabelText("Calendar"), { target: { value: "2" } });
    type("cycle", "1");
    type("round", "1");
    type("watch", "1");
    const list = await results();
    expect(
      await within(list).findByText(
        "age 5, cohort 42, orbit 412, span 3 (Third), sol 9, bell 0, pulse 0",
      ),
    ).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("convert", { from: 2, date: [1, 1, 1] });
  });

  it("says a value isn't a whole number without asking the core", async () => {
    core();
    await openConverter();
    type("orbit", "7b");
    expect(await screen.findByText("must be a whole number")).toBeInTheDocument();
    expect(converts()).toEqual([]);
  });

  it("shows a refused date at its field and no results (FR-011)", async () => {
    core({
      conversion: () => {
        throw { message: "span 13 is outside 1 to 12", index: 1 };
      },
    });
    await openConverter();
    type("orbit", "412");
    type("span", "13");
    const message = await screen.findByText("span 13 is outside 1 to 12");
    expect(screen.getByLabelText("span").closest(".field")).toContainElement(message);
    expect(screen.queryByRole("region", { name: "Results" })).not.toBeInTheDocument();
  });

  it("says when there's no other calendar to convert into (Story 2, scenario 6)", async () => {
    core({
      calendars: [{ id: 1, name: "Aster", problem: null }],
      converter: [ASTER],
      conversion: () => ({ rounded: false, results: [], unusable: [] }),
    });
    await openConverter();
    type("orbit", "1");
    expect(await screen.findByText(NO_OTHER)).toBeInTheDocument();
  });

  it("lists calendars that can't be placed with their problems (FR-006)", async () => {
    core();
    await openConverter();
    type("orbit", "412");
    const list = await screen.findByRole("list", { name: "Calendars that can't be placed" });
    expect(list).toHaveTextContent(
      "Wick can't be used: it's anchored to Moss, which can't be placed",
    );
  });

  it("says when the date starts between two of the timeline's finest steps", async () => {
    core({ conversion: () => ({ ...IN_ASTER, rounded: true }) });
    await openConverter();
    type("orbit", "1");
    expect(await screen.findByText(ROUNDED)).toBeInTheDocument();
  });

  it("switches precision without asking the core, keeping it until the app restarts (FR-010, FR-010a)", async () => {
    core();
    await openConverter();
    type("orbit", "412");
    const list = await results();
    expect(
      await within(list).findByText("cycle 1, round 1, watch 2, partway through"),
    ).toBeInTheDocument();
    const asked = converts().length;
    fireEvent.click(screen.getByRole("radio", { name: "Decimal" }));
    expect(within(list).getByText("cycle 1, round 1, watch ≈ 2.035")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("radio", { name: "Exact" }));
    expect(
      within(list).getByText("cycle 1, round 1, watch 2 + 1785559/50000000"),
    ).toBeInTheDocument();
    expect(converts().length).toBe(asked);
    // Back to the list and back again: still exact.
    fireEvent.click(screen.getByRole("button", { name: "Back to calendars" }));
    fireEvent.click(await enabledConverterButton());
    await screen.findByLabelText("Calendar");
    expect(screen.getByRole("radio", { name: "Exact" })).toBeChecked();
    // A fresh app starts at partway.
    cleanup();
    await openConverter();
    expect(screen.getByRole("radio", { name: "Partway" })).toBeChecked();
    await waitFor(() => expect(screen.getByRole("radio", { name: "Exact" })).not.toBeChecked());
  });
});
