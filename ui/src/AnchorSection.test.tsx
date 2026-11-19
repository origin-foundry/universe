import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
const dialog = vi.hoisted(() => ({ open: vi.fn(), save: vi.fn(), message: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => dialog);

import { DEFINITION_FIRST } from "./AnchorSection";
import type {
  AnchorBase,
  AnchorForm,
  CalendarDetails,
  CalendarForm,
  DateCheck,
  FieldError,
} from "./calendarApi";
import CalendarEditor from "./CalendarEditor";
import { asterForm, startingForm, tallowForm } from "./calendarFixtures";
import type { UniverseState } from "./fileFlows";

const dirty: UniverseState = { open: true, name: "Test", path: null, dirty: true };

const ASTER_UNITS = ["orbit", "span", "sol", "bell", "pulse"];
const TALLOW_UNITS = ["cycle", "round", "watch"];
const SPAN_NAMES = ["Prime", "Second", "Third"];

const BASES: AnchorBase[] = [
  { id: 1, name: "Aster", ineligible: null },
  { id: 3, name: "Wick", ineligible: "follows" },
  { id: 4, name: "Odd", ineligible: "unplaceable" },
];

/** A date read back as the core would: the given values, then first values, named. */
function reading(units: string[], date: (number | null)[]): DateCheck {
  return {
    values: units.map((unit, i) => {
      const value = date[i] ?? (i >= 3 ? 0 : 1);
      const name = unit === "span" ? (SPAN_NAMES[value - 1] ?? null) : null;
      return { unit, value, name };
    }),
  };
}

interface Core {
  bases?: AnchorBase[];
  put?: (form: CalendarForm) => unknown;
  definition?: boolean;
  followers?: string[];
}

/** A fake core: Aster (1), Tallow (2), Wick (3) and a calendar that can't be placed (4). */
function core({ bases = BASES, put, definition = false, followers = [] }: Core = {}) {
  invoke.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
    switch (command) {
      case "anchor_bases":
        return bases;
      case "converter_calendars":
        return [
          { id: 1, name: "Aster", units: ASTER_UNITS, problem: null },
          { id: 2, name: "Tallow", units: TALLOW_UNITS, problem: null },
          { id: 3, name: "Wick", units: ["beat"], problem: null },
        ];
      case "read_date": {
        const source = args!.source as { id: number } | { form: CalendarForm };
        const date = args!.date as (number | null)[];
        if ("form" in source) {
          if (definition) return { definition: "the fundamental unit's length is missing" };
          return reading(TALLOW_UNITS, date);
        }
        if (source.id === 1 && date[1] === 13) {
          return { message: "span 13 is outside 1 to 12", index: 1 };
        }
        return reading(source.id === 1 ? ASTER_UNITS : TALLOW_UNITS, date);
      }
      case "put_calendar":
        if (put) return put(args!.form as CalendarForm);
        return { state: dirty, id: 2 };
      case "calendar_followers":
        return followers;
      case "delete_calendar":
        return dirty;
      default:
        throw new Error(`unexpected command ${command}`);
    }
  });
}

const fixed = (base: number, date: number[], baseDate: number[]): AnchorForm => ({
  kind: "fixed",
  base,
  date,
  baseDate,
});

function editor(
  initial: CalendarForm,
  id: number | null,
  extra: { canAnchor?: boolean; details?: Pick<CalendarDetails, "rounded" | "note"> } = {},
) {
  const props = { onKept: vi.fn(), onClose: vi.fn(), onDeleted: vi.fn(), onPending: vi.fn() };
  render(<CalendarEditor id={id} initial={initial} {...extra} {...props} />);
  return props;
}

const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
const place = () => screen.getByRole("group", { name: "Place in time" });
const commands = () => invoke.mock.calls.map((c) => c[0] as string);

afterEach(() => {
  invoke.mockReset();
  Object.values(dialog).forEach((f) => f.mockReset());
});

describe("Place in time: a calendar that isn't anchored", () => {
  it("says so, with nothing to enter and no button when it's alone (Story 1, scenario 1)", () => {
    core();
    editor(asterForm(), 1);
    expect(within(place()).getByText("Not anchored.")).toBeInTheDocument();
    expect(screen.queryByLabelText("Base calendar")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Anchor to a base calendar" }),
    ).not.toBeInTheDocument();
    expect(document.body.textContent).not.toMatch(/reference|timeline|epoch|instant/i);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("can be anchored when another calendar can be placed, starting as a new one (FR-001a)", async () => {
    core({ bases: [{ id: 2, name: "Tallow", ineligible: null }] });
    editor(asterForm(), 1, { canAnchor: true });
    fireEvent.click(screen.getByRole("button", { name: "Anchor to a base calendar" }));
    expect(await screen.findByLabelText("Base calendar")).toHaveValue("2");
    expect(invoke).toHaveBeenCalledWith("anchor_bases", { id: 1 });
    expect(screen.getByLabelText("Anchor date orbit")).toHaveValue("1");
    expect(await screen.findByLabelText("Base date cycle")).toHaveValue("1");
    expect(screen.getByRole("radio", { name: /Fix the moment/ })).toBeChecked();
    // There's never a way to remove an anchor.
    expect(screen.queryByRole("button", { name: /remove anchor|not anchored/i })).toBeNull();
  });
});

describe("Place in time: entering an anchor", () => {
  it("starts a new calendar anchored to Aster at both first values, fixing the moment (FR-007)", async () => {
    core();
    editor({ ...startingForm(), anchor: fixed(1, [1], [1]) }, null);
    expect(await screen.findByRole("option", { name: "Aster" })).toBeInTheDocument();
    expect(screen.getByLabelText("Base calendar")).toHaveValue("1");
    expect(screen.getByLabelText("Anchor date unit")).toHaveValue("1");
    for (const [i, unit] of ASTER_UNITS.entries()) {
      expect(await screen.findByLabelText(`Base date ${unit}`)).toHaveValue(i === 0 ? "1" : "");
    }
    expect(
      screen.getByRole("radio", { name: "Fix the moment: stays put if Aster is edited" }),
    ).toBeChecked();
    expect(
      screen.getByRole("radio", { name: "Keep the equation: follows Aster" }),
    ).not.toBeChecked();
  });

  it("lists every other calendar, greying out those that can't be the base (FR-002a)", async () => {
    core();
    editor(tallowForm(fixed(1, [1], [1])), 2);
    const wick = await screen.findByRole("option", { name: "Wick (follows this calendar)" });
    expect(wick).toBeDisabled();
    expect(screen.getByRole("option", { name: "Odd (can't be placed)" })).toBeDisabled();
    expect(screen.getByRole("option", { name: "Aster" })).toBeEnabled();
  });

  it("resets the base date to the new base calendar's units", async () => {
    core({
      bases: [
        { id: 1, name: "Aster", ineligible: null },
        { id: 2, name: "Tallow", ineligible: null },
      ],
    });
    editor({ ...startingForm(), anchor: fixed(1, [1], [412, 3, 9]) }, null);
    await screen.findByLabelText("Base date pulse");
    fireEvent.change(screen.getByLabelText("Base calendar"), { target: { value: "2" } });
    expect(await screen.findByLabelText("Base date cycle")).toHaveValue("1");
    expect(screen.getByLabelText("Base date round")).toHaveValue("");
    expect(screen.queryByLabelText("Base date orbit")).not.toBeInTheDocument();
  });

  it("checks each date as it's typed and flags a refused value at its field (FR-002b)", async () => {
    core();
    editor(tallowForm(fixed(1, [1, 1, 1], [412])), null);
    await screen.findByLabelText("Base date span");
    type("Base date span", "13");
    const field = screen.getByLabelText("Base date span");
    await waitFor(() => expect(field).toHaveAttribute("aria-invalid", "true"));
    expect(document.getElementById(field.getAttribute("aria-describedby")!)).toHaveTextContent(
      "span 13 is outside 1 to 12",
    );
    expect(invoke).toHaveBeenCalledWith("read_date", { source: { id: 1 }, date: [412, 13] });
    expect(commands()).not.toContain("put_calendar");
    expect(screen.queryByText(/began at the start of/)).not.toBeInTheDocument();
  });

  it("reads the anchor back as a sentence with member names", async () => {
    core();
    editor(tallowForm(fixed(1, [1, 1, 1], [412, 3, 9])), null);
    expect(
      await screen.findByText(
        "Tallow's cycle 1, round 1, watch 1 began at the start of Aster's orbit 412, span 3 (Third), sol 9.",
      ),
    ).toBeInTheDocument();
    const draftCall = invoke.mock.calls.find(
      (c) => c[0] === "read_date" && "form" in (c[1].source as object),
    )!;
    expect(draftCall[1].date).toEqual([1, 1, 1]);
  });

  it("checks the anchor date only once the definition is valid", async () => {
    core({ definition: true });
    editor({ ...startingForm(), anchor: fixed(1, [1], [1]) }, null);
    expect(await screen.findByText(DEFINITION_FIRST)).toBeInTheDocument();
    expect(screen.queryByText(/began at the start of/)).not.toBeInTheDocument();
  });

  it("says a value that isn't a whole number without asking the core", async () => {
    core();
    editor(tallowForm(fixed(1, [1, 1, 1], [412, 3, 9])), null);
    await screen.findByText(/began at the start of/);
    const before = invoke.mock.calls.length;
    type("Anchor date round", "3a");
    expect(await screen.findByText("must be a whole number")).toBeInTheDocument();
    const after = invoke.mock.calls.slice(before).filter((c) => c[0] === "read_date");
    expect(after).toEqual([]);
  });

  it("keeps the entered anchor (Story 1, scenario 2)", async () => {
    core();
    const { onKept } = editor({ ...startingForm(), anchor: fixed(1, [1], [1]) }, null);
    fireEvent.change(screen.getByLabelText("Length, in SI seconds (physical)"), {
      target: { value: "30000" },
    });
    await screen.findByLabelText("Base date span");
    type("Base date orbit", "412");
    type("Base date span", "3");
    type("Base date sol", "9");
    fireEvent.click(screen.getByRole("radio", { name: /Keep the equation/ }));
    fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
    await waitFor(() => expect(onKept).toHaveBeenCalled());
    const put = invoke.mock.calls.find((c) => c[0] === "put_calendar")!;
    expect((put[1].form as CalendarForm).anchor).toEqual({
      kind: "equation",
      base: 1,
      date: [1],
      baseDate: [412, 3, 9],
    });
  });

  it("shows a refusal at each anchor location", async () => {
    const at = (location: FieldError["at"], message: string) => ({ message, at: location });
    const cases: [FieldError, string][] = [
      [at({ kind: "anchor", field: "base" }, "Choose a base calendar."), "Base calendar"],
      [
        at({ kind: "anchor", field: "date", index: 2 }, "watch 8 is outside 1 to 7"),
        "Anchor date watch",
      ],
      [
        at(
          { kind: "anchor", field: "baseDate", index: 1 },
          "span 13 is outside 1 to 12 in orbit 412",
        ),
        "Base date span",
      ],
    ];
    for (const [error, label] of cases) {
      core({
        put: () => {
          throw error;
        },
      });
      const { unmount } = render(
        <CalendarEditor
          id={2}
          initial={tallowForm(fixed(1, [1, 1, 1], [412, 3, 9]))}
          onKept={vi.fn()}
          onClose={vi.fn()}
          onDeleted={vi.fn()}
          onPending={vi.fn()}
        />,
      );
      await screen.findByLabelText("Base date span");
      fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
      const message = await screen.findByText(error.message);
      const field = screen.getByLabelText(label).closest(".field")!;
      expect(field).toContainElement(message);
      unmount();
      invoke.mockReset();
    }
    // An anchor out of range is shown at the section.
    core({
      put: () => {
        throw at(
          { kind: "anchor" },
          "This anchor would put the calendar beyond the timeline's range.",
        );
      },
    });
    editor(tallowForm(fixed(1, [1, 1, 1], [412, 3, 9])), 2);
    fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
    expect(await within(place()).findByText(/beyond the timeline's range/)).toBeInTheDocument();
  });

  it("shows no timeline value anywhere (FR-002, FR-012)", async () => {
    core();
    editor(tallowForm(fixed(1, [1, 1, 1], [412, 3, 9])), 2, {
      details: { rounded: false, note: null },
    });
    await screen.findByText(/began at the start of/);
    expect(document.body.textContent).not.toMatch(/epoch|instant|nanosecond|reference/i);
  });
});

describe("Place in time: kinds, notes and deleting (Story 3)", () => {
  it("sends the kind chosen", async () => {
    core();
    editor(tallowForm({ ...fixed(1, [1, 1, 1], [412, 3, 9]), kind: "equation" }), 2);
    expect(screen.getByRole("radio", { name: /Keep the equation/ })).toBeChecked();
    fireEvent.click(screen.getByRole("radio", { name: /Fix the moment/ }));
    fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
    await waitFor(() => expect(commands()).toContain("put_calendar"));
    const put = invoke.mock.calls.find((c) => c[0] === "put_calendar")!;
    expect((put[1].form as CalendarForm).anchor?.kind).toBe("fixed");
  });

  it("says where a fixed moment now falls in a base calendar that changed (FR-005a)", async () => {
    core();
    editor(tallowForm(fixed(1, [1, 1, 1], [412, 3, 9])), 2, {
      details: {
        rounded: false,
        note: {
          kind: "moved",
          now: {
            above: [],
            values: [
              { unit: "orbit", value: 412, name: null },
              { unit: "span", value: 2, name: "Second" },
              { unit: "sol", value: 50, name: null },
              { unit: "bell", value: 3, name: null },
              { unit: "pulse", value: 41, name: null },
            ],
            partway: { decimal: "0.370", decimalExact: true, exact: "37/100" },
          },
        },
      },
    });
    expect(
      await screen.findByText(
        "Aster has changed since: this moment is now orbit 412, span 2 (Second), sol 50, bell 3, pulse 41, partway through in it.",
      ),
    ).toBeInTheDocument();
  });

  it("says when the base calendar has been deleted (FR-006a)", async () => {
    core({ bases: [] });
    editor(tallowForm({ ...fixed(1, [1, 1, 1], [412, 3, 9]), base: null }), 2, {
      details: { rounded: false, note: { kind: "gone", name: "Aster" } },
    });
    expect(
      await screen.findByText("Its base calendar, Aster, has since been deleted."),
    ).toBeInTheDocument();
  });

  it("says when the base date starts between two of the timeline's finest steps", async () => {
    core();
    editor(tallowForm(fixed(1, [1, 1, 1], [412, 3, 9])), 2, {
      details: { rounded: true, note: null },
    });
    expect(
      await screen.findByText(
        "Aster's date starts between two of the timeline's finest steps; this calendar is placed from the next one.",
      ),
    ).toBeInTheDocument();
  });

  it("names the calendars that follow it before deleting (Story 3, scenario 4)", async () => {
    core({ followers: ["Tallow"] });
    dialog.message.mockResolvedValue("Delete");
    const { onDeleted } = editor(asterForm(), 1);
    fireEvent.click(screen.getByRole("button", { name: "Delete calendar" }));
    await waitFor(() => expect(onDeleted).toHaveBeenCalled());
    expect(invoke).toHaveBeenCalledWith("calendar_followers", { id: 1 });
    expect(dialog.message.mock.calls[0]![0]).toBe(
      "Delete Aster?\n\nTallow follows Aster and will be fixed where it stands.",
    );
  });

  it("shows a loop at the base calendar (Story 3, scenario 1)", async () => {
    const loop = {
      message: "Wick follows Tallow, so Tallow can't be anchored to it.",
      at: { kind: "anchor", field: "base" },
    };
    core({
      put: () => {
        throw loop;
      },
    });
    editor(tallowForm(fixed(1, [1, 1, 1], [412, 3, 9])), 2);
    fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
    const message = await screen.findByText(loop.message);
    expect(screen.getByLabelText("Base calendar").closest(".field")).toContainElement(message);
  });
});
