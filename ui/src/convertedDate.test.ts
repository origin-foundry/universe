import { describe, expect, it } from "vitest";

import type { ConvertedDate } from "./calendarApi";
import { convertedDateText } from "./convertedDate";

const v = (unit: string, value: number, name: string | null = null) => ({ unit, value, name });

const aster = (partway: ConvertedDate["partway"]): ConvertedDate => ({
  above: [v("age", 5), v("cohort", 42)],
  values: [v("orbit", 412), v("span", 3, "Third"), v("sol", 9), v("bell", 7), v("pulse", 41)],
  partway,
});

const START = "age 5, cohort 42, orbit 412, span 3 (Third), sol 9, bell 7";

describe("convertedDateText", () => {
  it("shows a moment at a unit's start with no mark at every precision", () => {
    for (const precision of ["partway", "decimal", "exact"] as const) {
      expect(convertedDateText(aster(null), precision)).toBe(`${START}, pulse 41`);
    }
  });

  it("says partway through, with no amount, by default", () => {
    const date = aster({ decimal: "0.370", decimalExact: true, exact: "37/100" });
    expect(convertedDateText(date, "partway")).toBe(`${START}, pulse 41, partway through`);
  });

  it("gives the decimal, marked when it isn't exact", () => {
    const exact = aster({ decimal: "0.370", decimalExact: true, exact: "37/100" });
    expect(convertedDateText(exact, "decimal")).toBe(`${START}, pulse 41.370`);
    const third = aster({ decimal: "0.333", decimalExact: false, exact: "1/3" });
    expect(convertedDateText(third, "decimal")).toBe(`${START}, pulse ≈ 41.333`);
  });

  it("gives the exact fraction", () => {
    const date = aster({ decimal: "0.370", decimalExact: true, exact: "37/100" });
    expect(convertedDateText(date, "exact")).toBe(`${START}, pulse 41 + 37/100`);
  });

  it("keeps a member's name after the number", () => {
    const date: ConvertedDate = {
      above: [],
      values: [v("orbit", 1), v("span", 2, "Second")],
      partway: { decimal: "0.500", decimalExact: true, exact: "1/2" },
    };
    expect(convertedDateText(date, "decimal")).toBe("orbit 1, span 2.500 (Second)");
    expect(convertedDateText(date, "exact")).toBe("orbit 1, span 2 + 1/2 (Second)");
  });

  it("doesn't fold a fraction into a negative number", () => {
    const date: ConvertedDate = {
      above: [],
      values: [v("tick", -3)],
      partway: { decimal: "0.250", decimalExact: true, exact: "1/4" },
    };
    expect(convertedDateText(date, "decimal")).toBe("tick -3 + 0.250");
  });
});
