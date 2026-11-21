import { describe, expect, it } from "vitest";

import { asterForm, startingForm, tallowForm } from "./calendarFixtures";
import { dateValues, draftFromForm, fieldKey, formFromDraft, unitsFromEra } from "./calendarDraft";

describe("calendarDraft: the anchor", () => {
  it("gives a draft's units from the era's unit down", () => {
    expect(unitsFromEra(draftFromForm(asterForm()))).toEqual([
      "orbit",
      "span",
      "sol",
      "bell",
      "pulse",
    ]);
    expect(unitsFromEra(draftFromForm(tallowForm()))).toEqual(["cycle", "round", "watch"]);
    expect(unitsFromEra(draftFromForm(startingForm()))).toEqual(["unit"]);
    const unknown = draftFromForm(asterForm());
    unknown.era.unit = "aeon";
    expect(unitsFromEra(unknown)).toEqual([]);
  });

  it("round-trips an anchor, dropping trailing empty values", () => {
    const anchor = { kind: "equation", base: 1, date: [1, 1, 1], baseDate: [412, 3, 9] } as const;
    const draft = draftFromForm(
      tallowForm({ ...anchor, date: [...anchor.date], baseDate: [...anchor.baseDate] }),
    );
    expect(draft.anchor).toEqual({
      kind: "equation",
      base: 1,
      date: ["1", "1", "1"],
      baseDate: ["412", "3", "9"],
    });
    draft.anchor!.baseDate.push("", "");
    expect(formFromDraft(draft).form?.anchor).toEqual(anchor);
    expect(draftFromForm(asterForm()).anchor).toBeNull();
    expect(formFromDraft(draftFromForm(asterForm())).form?.anchor).toBeNull();
  });

  it("refuses a value that isn't a whole number at its field", () => {
    const draft = draftFromForm(tallowForm({ kind: "fixed", base: 1, date: [1], baseDate: [1] }));
    draft.anchor!.baseDate = ["412", "3a"];
    expect(formFromDraft(draft).error).toEqual({
      message: "must be a whole number",
      at: { kind: "anchor", field: "baseDate", index: 1 },
    });
    expect(dateValues(["1", "", "x"])).toEqual({ values: null, bad: 2 });
    expect(dateValues(["1", "", "3", ""])).toEqual({ values: [1, null, 3], bad: null });
  });

  it("keys anchor locations", () => {
    expect(fieldKey({ kind: "anchor", field: "baseDate", index: 1 })).toBe("anchor.baseDate.1");
    expect(fieldKey({ kind: "anchor", field: "base" })).toBe("anchor.base");
    expect(fieldKey({ kind: "anchor" })).toBe("anchor");
  });
});
