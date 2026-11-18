// Test data shared by the calendar UI tests: the Aster test calendar of specs 004 and 006, as
// the editor sends it, and the starting form the core serves (research R7).

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
  };
}

export function startingForm(): CalendarForm {
  return {
    name: "Calendar",
    groupings: [],
    fundamental: { name: "unit", length: "", firstNumber: null },
    subdivisions: [],
    era: { name: "Era", unit: "unit" },
  };
}
