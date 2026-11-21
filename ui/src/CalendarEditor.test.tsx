import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
const dialog = vi.hoisted(() => ({ open: vi.fn(), save: vi.fn(), message: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => dialog);

import type { CalendarForm, FieldError, Location } from "./calendarApi";
import { fieldKey } from "./calendarDraft";
import CalendarEditor from "./CalendarEditor";
import { ASTER_RULE, SPANS, asterForm, startingForm } from "./calendarFixtures";
import type { UniverseState } from "./fileFlows";

const dirty: UniverseState = { open: true, name: "Test", path: null, dirty: true };

function editor(initial: CalendarForm = startingForm(), id: number | null = null) {
  const props = {
    onKept: vi.fn(),
    onClose: vi.fn(),
    onDeleted: vi.fn(),
    onPending: vi.fn(),
  };
  render(<CalendarEditor id={id} initial={initial} {...props} />);
  return props;
}

const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
const click = (name: string) => fireEvent.click(screen.getByRole("button", { name }));
const keep = () => click("Keep changes");

afterEach(() => {
  invoke.mockReset();
  Object.values(dialog).forEach((f) => f.mockReset());
});

describe("CalendarEditor: entering a calendar", () => {
  it("starts from the starting form, with the length labelled as an SI physical input", () => {
    editor();
    expect(screen.getByLabelText("Calendar name")).toHaveValue("Calendar");
    expect(screen.getByLabelText("Fundamental unit name")).toHaveValue("unit");
    const length = screen.getByLabelText("Length, in SI seconds (physical)");
    expect(length).toHaveValue("");
    expect(screen.getByLabelText("Era name")).toHaveValue("Era");
    expect(screen.getByLabelText("Era's unit")).toHaveValue("unit");
    expect(screen.queryByLabelText(/grouping 1/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/subdivision 1/i)).not.toBeInTheDocument();
  });

  it("never shows an epoch or any absolute-timeline value (FR-009)", () => {
    editor(asterForm(), 1);
    const text = document.body.textContent ?? "";
    expect(text).not.toMatch(/epoch|GCT|timeline|instant/i);
    expect(screen.queryByLabelText(/epoch/i)).not.toBeInTheDocument();
  });

  it("shows the engine's default in empty first numbers and sends them as null (FR-007)", async () => {
    invoke.mockResolvedValue({ state: dirty, id: 1 });
    editor(asterForm(), 1);
    expect(screen.getByLabelText("Grouping 3 first number")).toHaveAttribute("placeholder", "1");
    expect(screen.getByLabelText("Grouping 3 member unit first number")).toHaveAttribute(
      "placeholder",
      "1",
    );
    expect(screen.getByLabelText("Fundamental unit first number")).toHaveAttribute(
      "placeholder",
      "1",
    );
    expect(screen.getByLabelText("Subdivision 1 first number")).toHaveAttribute("placeholder", "0");
    keep();
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    expect(invoke).toHaveBeenCalledWith("put_calendar", { id: 1, form: asterForm() });
  });

  it("enters Aster section by section and sends exactly that form", async () => {
    invoke.mockResolvedValue({ state: dirty, id: 7 });
    const { onKept } = editor();
    type("Calendar name", "Aster");
    click("Add grouping");
    click("Add grouping");
    click("Add grouping");
    type("Grouping 1 name", "age");
    type("Grouping 1 member 1 count", "10");
    type("Grouping 2 name", "cohort");
    type("Grouping 2 member 1 count", "10");
    type("Grouping 3 name", "orbit");
    type("Grouping 3 member unit name", "span");
    SPANS.forEach(([name, count], j) => {
      if (j > 0) click("Add member to grouping 3");
      type(`Grouping 3 member ${j + 1} name`, name);
      type(`Grouping 3 member ${j + 1} count`, String(count));
    });
    type("Fundamental unit name", "sol");
    type("Length, in SI seconds (physical)", "88775.244");
    click("Add subdivision");
    click("Add subdivision");
    type("Subdivision 1 name", "bell");
    type("Subdivision 1 parts", "20");
    type("Subdivision 2 name", "pulse");
    type("Subdivision 2 parts", "100");
    type("Era name", "After Landing");
    type("Era's unit", "orbit");
    click("Add adjustment rule to grouping 3");
    type("Grouping 3 rule 1 condition", ASTER_RULE);
    type("Grouping 3 rule 1 member", "Twelfth");
    type("Grouping 3 rule 1 change", "1");
    keep();
    await waitFor(() => expect(onKept).toHaveBeenCalledWith(dirty, 7));
    expect(invoke).toHaveBeenCalledWith("put_calendar", { id: null, form: asterForm() });
    // Typing the whole of Aster takes several seconds when the suite runs in parallel.
  }, 20_000);

  it("sends the length as the text typed, never as a number", async () => {
    invoke.mockResolvedValue({ state: dirty, id: 1 });
    editor();
    type("Length, in SI seconds (physical)", "355/113");
    keep();
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    const [, args] = invoke.mock.calls[0]!;
    expect(args.form.fundamental.length).toBe("355/113");
  });

  it("refuses a whole-number field that isn't one without calling the core", () => {
    editor(asterForm(), 1);
    type("Grouping 3 member 4 count", "12a");
    keep();
    const input = screen.getByLabelText("Grouping 3 member 4 count");
    expect(input).toHaveAttribute("aria-invalid", "true");
    expect(screen.getByRole("alert")).toHaveTextContent("must be a whole number");
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("CalendarEditor: lists (FR-006)", () => {
  it("adds, removes and moves groupings", async () => {
    invoke.mockResolvedValue({ state: dirty, id: 1 });
    editor(asterForm(), 1);
    click("Move grouping 3 up");
    expect(screen.getByLabelText("Grouping 2 name")).toHaveValue("orbit");
    click("Move grouping 1 down");
    expect(screen.getByLabelText("Grouping 1 name")).toHaveValue("orbit");
    expect(screen.getByLabelText("Grouping 2 name")).toHaveValue("age");
    click("Remove grouping 3");
    click("Add grouping");
    expect(screen.getByLabelText("Grouping 3 name")).toHaveValue("");
    click("Remove grouping 3");
    keep();
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    const [, args] = invoke.mock.calls[0]!;
    expect(args.form.groupings.map((g: { name: string }) => g.name)).toEqual(["orbit", "age"]);
  });

  it("adds, removes and moves members, rules and subdivisions", async () => {
    invoke.mockResolvedValue({ state: dirty, id: 1 });
    editor(asterForm(), 1);
    click("Move member 12 of grouping 3 up");
    expect(screen.getByLabelText("Grouping 3 member 11 name")).toHaveValue("Twelfth");
    click("Move member 11 of grouping 3 down");
    click("Remove member 1 of grouping 3");
    expect(screen.getByLabelText("Grouping 3 member 1 name")).toHaveValue("Second");
    click("Add adjustment rule to grouping 3");
    type("Grouping 3 rule 2 condition", "orbit % 7 == 0");
    click("Move rule 2 of grouping 3 up");
    expect(screen.getByLabelText("Grouping 3 rule 1 condition")).toHaveValue("orbit % 7 == 0");
    click("Remove rule 1 of grouping 3");
    click("Move subdivision 2 up");
    expect(screen.getByLabelText("Subdivision 1 name")).toHaveValue("pulse");
    click("Remove subdivision 2");
    keep();
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    const form = invoke.mock.calls[0]![1].form as CalendarForm;
    expect(form.groupings[2]!.members.map((m) => m.name)).toEqual(
      SPANS.slice(1).map(([name]) => name),
    );
    expect(form.groupings[2]!.rules).toEqual([
      { condition: ASTER_RULE, member: "Twelfth", change: 1 },
    ]);
    expect(form.subdivisions).toEqual([{ name: "pulse", parts: 100, firstNumber: null }]);
  });

  it("disables moves that would leave the list", () => {
    editor(asterForm(), 1);
    expect(screen.getByRole("button", { name: "Move grouping 1 up" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Move grouping 3 down" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Move subdivision 1 up" })).toBeDisabled();
  });

  it("shows the rules of the era's unit, and of any grouping that has some", () => {
    editor(asterForm(), 1);
    const orbit = screen.getByRole("group", { name: "Grouping 3" });
    expect(within(orbit).getByText("Adjustment rules")).toBeInTheDocument();
    const age = screen.getByRole("group", { name: "Grouping 1" });
    expect(within(age).queryByText("Adjustment rules")).not.toBeInTheDocument();
  });
});

describe("CalendarEditor: refusals are shown where they belong (Story 3)", () => {
  const cases: Location[] = [
    { kind: "calendarName" },
    { kind: "era", field: "name" },
    { kind: "era", field: "unit" },
    { kind: "grouping", index: 1 },
    { kind: "grouping", index: 2, field: "name" },
    { kind: "grouping", index: 2, field: "firstNumber" },
    { kind: "grouping", index: 2, field: "memberUnit" },
    { kind: "member", grouping: 2, index: 3 },
    { kind: "member", grouping: 2, index: 3, field: "name" },
    { kind: "member", grouping: 2, index: 3, field: "count" },
    { kind: "rule", grouping: 2, index: 0 },
    { kind: "rule", grouping: 2, index: 0, field: "condition" },
    { kind: "rule", grouping: 2, index: 0, field: "member" },
    { kind: "rule", grouping: 2, index: 0, field: "change" },
    { kind: "fundamental" },
    { kind: "fundamental", field: "name" },
    { kind: "fundamental", field: "length" },
    { kind: "fundamental", field: "firstNumber" },
    { kind: "subdivision", index: 1 },
    { kind: "subdivision", index: 1, field: "name" },
    { kind: "subdivision", index: 1, field: "parts" },
    { kind: "subdivision", index: 1, field: "firstNumber" },
  ];

  it.each(cases)("shows a refusal at %o next to it, focused", async (at) => {
    const refusal: FieldError = { message: "the engine says no", at };
    invoke.mockRejectedValueOnce(refusal);
    editor(asterForm(), 1);
    keep();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("the engine says no");
    const target = document.querySelector<HTMLElement>(`[data-field="${fieldKey(at)}"]`)!;
    expect(target).not.toBeNull();
    await waitFor(() => expect(target).toHaveFocus());
    const isInput = at.kind === "calendarName" || ("field" in at && at.field !== undefined);
    if (isInput) {
      expect(target).toHaveAttribute("aria-invalid", "true");
      expect(target.closest("li, fieldset, form")).toContainElement(alert);
    } else {
      expect(target).toContainElement(alert);
    }
  });

  it("keeps everything typed, and keeping again after a fix succeeds", async () => {
    invoke.mockRejectedValueOnce({
      message: "two units are named bell",
      at: { kind: "subdivision", index: 1, field: "name" },
    });
    invoke.mockResolvedValueOnce({ state: dirty, id: 1 });
    const { onKept } = editor(asterForm(), 1);
    type("Subdivision 2 name", "bell");
    type("Grouping 3 member 1 count", "57");
    keep();
    expect(await screen.findByRole("alert")).toHaveTextContent("two units are named bell");
    expect(screen.getByLabelText("Subdivision 2 name")).toHaveValue("bell");
    expect(screen.getByLabelText("Grouping 3 member 1 count")).toHaveValue("57");
    expect(screen.getByLabelText("Grouping 3 rule 1 condition")).toHaveValue(ASTER_RULE);
    expect(onKept).not.toHaveBeenCalled();

    type("Subdivision 2 name", "pulse");
    keep();
    await waitFor(() => expect(onKept).toHaveBeenCalledWith(dirty, 1));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows an expression error with its position", async () => {
    invoke.mockRejectedValueOnce({
      message: "rule 1 of orbit: expected a value at 1:8",
      at: { kind: "rule", grouping: 2, index: 0, field: "condition" },
    });
    editor(asterForm(), 1);
    type("Grouping 3 rule 1 condition", "orbit %");
    keep();
    expect(await screen.findByRole("alert")).toHaveTextContent("at 1:8");
    expect(screen.getByLabelText("Grouping 3 rule 1 condition")).toHaveAttribute(
      "aria-invalid",
      "true",
    );
  });

  it("shows a missing length at the length field", async () => {
    invoke.mockRejectedValueOnce({
      message: "the fundamental unit's length is missing",
      at: { kind: "fundamental", field: "length" },
    });
    editor();
    keep();
    const length = screen.getByLabelText("Length, in SI seconds (physical)");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "the fundamental unit's length is missing",
    );
    expect(length).toHaveAttribute("aria-invalid", "true");
    expect(length).toHaveAttribute("aria-describedby", screen.getByRole("alert").id);
  });
});
