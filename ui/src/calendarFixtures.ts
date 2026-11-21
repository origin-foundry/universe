// Test data shared by the calendar UI tests: the Aster test calendar of specs 004 and 006 and
// spec 008's Tallow, as the editor sends them, and the starting form the core serves.

import type { CalendarForm } from "./calendarApi";

export const ASTER_RULE = "orbit % 5 == 0 && (orbit % 100 != 0 || orbit % 500 == 0)";

export const SPANS: [string, number][] = [
  ["Prime", 56],
  ["Second", 56],
  ["Third", 55],
  ["Fourth", 56],
  ["Fifth", 56],
  ["Sixth", 55],
  ["Seventh", 56],
  ["Eighth", 56],
  ["Ninth", 55],
  ["Tenth", 56],
  ["Eleventh", 56],
  ["Twelfth", 55],
];

const oneMember = (name: string) => ({
  name,
  firstNumber: null,
  memberUnit: null,
  members: [{ name: null, count: 10 }],
  rules: [],
});

export function asterForm(): CalendarForm {
  return {
    name: "Aster",
    groupings: [
      oneMember("age"),
      oneMember("cohort"),
      {
        name: "orbit",
        firstNumber: null,
        memberUnit: { name: "span", firstNumber: null },
        members: SPANS.map(([name, count]) => ({ name, count })),
        rules: [{ condition: ASTER_RULE, member: "Twelfth", change: 1 }],
      },
    ],
    fundamental: { name: "sol", length: "88775.244", firstNumber: null },
    subdivisions: [
      { name: "bell", parts: 20, firstNumber: null },
      { name: "pulse", parts: 100, firstNumber: null },
    ],
    era: { name: "After Landing", unit: "orbit" },
    anchor: null,
  };
}

export function startingForm(): CalendarForm {
  return {
    name: "Calendar",
    groupings: [],
    fundamental: { name: "unit", length: "", firstNumber: null },
    subdivisions: [],
    era: { name: "Era", unit: "unit" },
    anchor: null,
  };
}

/** The spec's Tallow: a cycle of 52 rounds, a round of 7 watches, a watch of 30,000 s. */
export function tallowForm(anchor: CalendarForm["anchor"] = null): CalendarForm {
  const one = (name: string, count: number) => ({
    name,
    firstNumber: null,
    memberUnit: null,
    members: [{ name: null, count }],
    rules: [],
  });
  return {
    name: "Tallow",
    groupings: [one("cycle", 52), one("round", 7)],
    fundamental: { name: "watch", length: "30000", firstNumber: null },
    subdivisions: [],
    era: { name: "Count", unit: "cycle" },
    anchor,
  };
}
