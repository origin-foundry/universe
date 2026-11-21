// What the editor holds while the owner types: every input as text. A draft becomes a
// `CalendarForm` only when it's kept; it's never stored (FR-012).

import type { AnchorForm, AnchorKind, CalendarForm, FieldError, Location } from "./calendarApi";

export interface MemberDraft {
  name: string;
  count: string;
}

export interface RuleDraft {
  condition: string;
  member: string;
  change: string;
}

export interface GroupingDraft {
  name: string;
  firstNumber: string;
  memberUnitName: string;
  memberUnitFirstNumber: string;
  members: MemberDraft[];
  rules: RuleDraft[];
}

export interface SubdivisionDraft {
  name: string;
  parts: string;
  firstNumber: string;
}

/** The anchor while it's edited: each date's values as text, from the era's unit down. */
export interface AnchorDraft {
  kind: AnchorKind;
  base: number | null;
  date: string[];
  baseDate: string[];
}

export interface Draft {
  name: string;
  groupings: GroupingDraft[];
  fundamental: { name: string; length: string; firstNumber: string };
  subdivisions: SubdivisionDraft[];
  era: { name: string; unit: string };
  /** Null for a calendar that isn't anchored. */
  anchor: AnchorDraft | null;
}

export const WHOLE_NUMBER_MESSAGE = "must be a whole number";

const text = (n: number | null): string => (n === null ? "" : String(n));

export function draftFromForm(form: CalendarForm): Draft {
  return {
    name: form.name,
    groupings: form.groupings.map((g) => ({
      name: g.name,
      firstNumber: text(g.firstNumber),
      memberUnitName: g.memberUnit?.name ?? "",
      memberUnitFirstNumber: text(g.memberUnit?.firstNumber ?? null),
      members: g.members.map((m) => ({ name: m.name ?? "", count: text(m.count) })),
      rules: g.rules.map((r) => ({
        condition: r.condition,
        member: r.member,
        change: text(r.change),
      })),
    })),
    fundamental: {
      name: form.fundamental.name,
      length: form.fundamental.length,
      firstNumber: text(form.fundamental.firstNumber),
    },
    subdivisions: form.subdivisions.map((s) => ({
      name: s.name,
      parts: text(s.parts),
      firstNumber: text(s.firstNumber),
    })),
    era: { ...form.era },
    anchor: form.anchor ? anchorDraft(form.anchor) : null,
  };
}

function anchorDraft(anchor: AnchorForm): AnchorDraft {
  return {
    kind: anchor.kind,
    base: anchor.base,
    date: anchor.date.map(text),
    baseDate: anchor.baseDate.map(text),
  };
}

/** A new anchor, as a new calendar starts (FR-007): fixing the moment, both dates at 1. */
export const startingAnchor = (base: number | null): AnchorDraft => ({
  kind: "fixed",
  base,
  date: ["1"],
  baseDate: ["1"],
});

/**
 * The draft's units from the era's unit down, by name: the fields of its anchor date. Empty
 * when the era's unit isn't one of the draft's units yet.
 */
export function unitsFromEra(draft: Draft): string[] {
  const chain: string[] = [];
  for (const g of draft.groupings) {
    chain.push(g.name.trim());
    if (g.memberUnitName.trim() !== "") chain.push(g.memberUnitName.trim());
  }
  chain.push(draft.fundamental.name.trim());
  chain.push(...draft.subdivisions.map((s) => s.name.trim()));
  const era = chain.indexOf(draft.era.unit.trim());
  return era < 0 || draft.era.unit.trim() === "" ? [] : chain.slice(era);
}

/**
 * A date's values as whole numbers (empty ones null, trailing ones left out), or the index of
 * the first value that isn't a whole number.
 */
export function dateValues(
  values: string[],
): { values: (number | null)[]; bad: null } | { values: null; bad: number } {
  const parsed: (number | null)[] = [];
  for (const [i, value] of values.entries()) {
    try {
      parsed.push(whole(value, { kind: "anchor" }));
    } catch {
      return { values: null, bad: i };
    }
  }
  while (parsed.length > 0 && parsed.at(-1) === null) parsed.pop();
  return { values: parsed, bad: null };
}

export const emptyGrouping = (): GroupingDraft => ({
  name: "",
  firstNumber: "",
  memberUnitName: "",
  memberUnitFirstNumber: "",
  members: [{ name: "", count: "" }],
  rules: [],
});
export const emptyMember = (): MemberDraft => ({ name: "", count: "" });
export const emptyRule = (): RuleDraft => ({ condition: "", member: "", change: "" });
export const emptySubdivision = (): SubdivisionDraft => ({ name: "", parts: "", firstNumber: "" });

class NotWhole extends Error {
  constructor(readonly at: Location) {
    super(WHOLE_NUMBER_MESSAGE);
  }
}

/** An empty field is `null`; anything but a whole number the UI can hold exactly is refused. */
function whole(value: string, at: Location): number | null {
  const trimmed = value.trim();
  if (trimmed === "") return null;
  const n = /^[+-]?\d+$/.test(trimmed) ? Number(trimmed) : NaN;
  if (!Number.isSafeInteger(n)) throw new NotWhole(at);
  return n;
}

/**
 * The form to send, or the first whole-number field that isn't one (in form order). The
 * length stays text, and empty optional fields are `null`, never the default.
 */
export function formFromDraft(
  draft: Draft,
): { form: CalendarForm; error: null } | { form: null; error: FieldError } {
  try {
    const form: CalendarForm = {
      name: draft.name,
      groupings: draft.groupings.map((g, index) => {
        const firstNumber = whole(g.firstNumber, { kind: "grouping", index, field: "firstNumber" });
        const memberUnitFirst = whole(g.memberUnitFirstNumber, {
          kind: "grouping",
          index,
          field: "memberUnit",
        });
        const hasMemberUnit = g.memberUnitName.trim() !== "" || memberUnitFirst !== null;
        return {
          name: g.name,
          firstNumber,
          memberUnit: hasMemberUnit
            ? { name: g.memberUnitName, firstNumber: memberUnitFirst }
            : null,
          members: g.members.map((m, j) => ({
            name: m.name.trim() === "" ? null : m.name,
            count: whole(m.count, { kind: "member", grouping: index, index: j, field: "count" }),
          })),
          rules: g.rules.map((r, k) => ({
            condition: r.condition,
            member: r.member,
            change: whole(r.change, { kind: "rule", grouping: index, index: k, field: "change" }),
          })),
        };
      }),
      fundamental: {
        name: draft.fundamental.name,
        length: draft.fundamental.length,
        firstNumber: whole(draft.fundamental.firstNumber, {
          kind: "fundamental",
          field: "firstNumber",
        }),
      },
      subdivisions: draft.subdivisions.map((s, index) => ({
        name: s.name,
        parts: whole(s.parts, { kind: "subdivision", index, field: "parts" }),
        firstNumber: whole(s.firstNumber, { kind: "subdivision", index, field: "firstNumber" }),
      })),
      era: { ...draft.era },
      anchor: draft.anchor === null ? null : anchorForm(draft.anchor),
    };
    return { form, error: null };
  } catch (refused) {
    if (refused instanceof NotWhole) {
      return { form: null, error: { message: refused.message, at: refused.at } };
    }
    throw refused;
  }
}

function anchorForm(anchor: AnchorDraft): AnchorForm {
  const date = (values: string[], field: "date" | "baseDate") => {
    const parsed = dateValues(values);
    if (parsed.bad !== null) throw new NotWhole({ kind: "anchor", field, index: parsed.bad });
    return parsed.values;
  };
  return {
    kind: anchor.kind,
    base: anchor.base,
    date: date(anchor.date, "date"),
    baseDate: date(anchor.baseDate, "baseDate"),
  };
}

/** A stable key for a location, matching the `data-field` of the input or section it names. */
export function fieldKey(at: Location): string {
  const field = "field" in at && at.field !== undefined ? `.${at.field}` : "";
  switch (at.kind) {
    case "calendarName":
      return "calendarName";
    case "era":
      return `era${field}`;
    case "grouping":
    case "subdivision":
      return `${at.kind}.${at.index}${field}`;
    case "member":
    case "rule":
      return `${at.kind}.${at.grouping}.${at.index}${field}`;
    case "fundamental":
      return `fundamental${field}`;
    case "anchor":
      return `anchor${field}${at.index === undefined ? "" : `.${at.index}`}`;
  }
}

export function sameDraft(a: Draft, b: Draft): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** `list` with the item at `from` moved to `to` (which must be in range). */
export function moved<T>(list: T[], from: number, to: number): T[] {
  const next = [...list];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item!);
  return next;
}
