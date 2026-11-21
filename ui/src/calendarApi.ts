// The calendar commands and their shapes
// (specs/007-calendars-in-project/contracts/calendar-commands.md, and spec 008's
// contracts/anchors-and-converter.md). They match crates/uni-app/src/calendars.rs and
// converter.rs.

import { invoke } from "@tauri-apps/api/core";

import type { UniverseState } from "./fileFlows";

export interface MemberUnitForm {
  name: string;
  firstNumber: number | null;
}

export interface MemberForm {
  name: string | null;
  count: number | null;
}

export interface RuleForm {
  condition: string;
  member: string;
  change: number | null;
}

export interface GroupingForm {
  name: string;
  firstNumber: number | null;
  memberUnit: MemberUnitForm | null;
  members: MemberForm[];
  rules: RuleForm[];
}

export interface FundamentalForm {
  name: string;
  /** In SI seconds, exactly as typed; never turned into a number here. */
  length: string;
  firstNumber: number | null;
}

export interface SubdivisionForm {
  name: string;
  parts: number | null;
  firstNumber: number | null;
}

export type AnchorKind = "fixed" | "equation";

/** A calendar's link into time (spec 008): its anchor date equated with a base date. */
export interface AnchorForm {
  kind: AnchorKind;
  /** The base calendar's id; null only once it has been deleted. */
  base: number | null;
  /** From the era's unit down; trailing empty values are left out. */
  date: (number | null)[];
  baseDate: (number | null)[];
}

export interface CalendarForm {
  name: string;
  groupings: GroupingForm[];
  fundamental: FundamentalForm;
  subdivisions: SubdivisionForm[];
  era: { name: string; unit: string };
  /** Null for a calendar that isn't anchored. */
  anchor?: AnchorForm | null;
}

/** One value of a date, with its unit's name and its member's name, if it has one. */
export interface DateValue {
  unit: string;
  value: number;
  name: string | null;
}

/** How far into its smallest unit a moment falls, as text the core worked out exactly. */
export interface Partway {
  decimal: string;
  decimalExact: boolean;
  exact: string;
}

export interface ConvertedDate {
  above: DateValue[];
  values: DateValue[];
  partway: Partway | null;
}

/** What the editor shows about a stored calendar's anchor. */
export interface CalendarDetails {
  form: CalendarForm;
  rounded: boolean;
  note: { kind: "moved"; now: ConvertedDate } | { kind: "gone"; name: string } | null;
}

export interface AnchorBase {
  id: number;
  name: string;
  ineligible: "follows" | "unplaceable" | null;
}

export interface DateError {
  message: string;
  index: number | null;
}

export type DateCheck = { values: DateValue[] } | DateError | { definition: string };

export interface ConverterCalendar {
  id: number;
  name: string;
  units: string[];
  problem: string | null;
}

export interface Conversion {
  rounded: boolean;
  results: { id: number; name: string; date: ConvertedDate }[];
  unusable: { name: string; problem: string }[];
}

export type Location =
  | { kind: "calendarName" }
  | { kind: "era"; field: "name" | "unit" }
  | { kind: "grouping"; index: number; field?: "name" | "firstNumber" | "memberUnit" }
  | { kind: "member"; grouping: number; index: number; field?: "name" | "count" }
  | { kind: "rule"; grouping: number; index: number; field?: "condition" | "member" | "change" }
  | { kind: "fundamental"; field?: "name" | "length" | "firstNumber" }
  | { kind: "subdivision"; index: number; field?: "name" | "parts" | "firstNumber" }
  | { kind: "anchor"; field?: "base" | "kind" | "date" | "baseDate"; index?: number };

export interface FieldError {
  message: string;
  at: Location;
}

export interface CalendarSummary {
  id: number;
  name: string;
  /** The engine's reason when a stored calendar can't be used (FR-015). */
  problem: string | null;
}

/** A `FieldError` is an object with `at`; every other failure is a plain message. */
export function isFieldError(error: unknown): error is FieldError {
  return typeof error === "object" && error !== null && "at" in error && "message" in error;
}

export function listCalendars(): Promise<CalendarSummary[]> {
  return invoke<CalendarSummary[]>("list_calendars");
}

export function getCalendar(id: number): Promise<CalendarDetails> {
  return invoke<CalendarDetails>("get_calendar", { id });
}

export function newCalendarForm(): Promise<CalendarForm> {
  return invoke<CalendarForm>("new_calendar_form");
}

export type PutOutcome =
  { kind: "kept"; state: UniverseState; id: number } | { kind: "refused"; error: FieldError };

/**
 * Adds (`id` null) or replaces a calendar. A form the core refuses comes back as `refused`;
 * any other failure rejects with its message.
 */
export async function putCalendar(id: number | null, form: CalendarForm): Promise<PutOutcome> {
  try {
    const result = await invoke<{ state: UniverseState; id: number }>("put_calendar", {
      id,
      form,
    });
    return { kind: "kept", ...result };
  } catch (error) {
    if (isFieldError(error)) return { kind: "refused", error };
    throw error;
  }
}

export function deleteCalendar(id: number): Promise<UniverseState> {
  return invoke<UniverseState>("delete_calendar", { id });
}

export function calendarFollowers(id: number): Promise<string[]> {
  return invoke<string[]>("calendar_followers", { id });
}

export function anchorBases(id: number | null): Promise<AnchorBase[]> {
  return invoke<AnchorBase[]>("anchor_bases", { id });
}

/** Checks a date in a stored calendar or in the editor's draft. */
export function readDate(
  source: { id: number } | { form: CalendarForm },
  date: (number | null)[],
): Promise<DateCheck> {
  return invoke<DateCheck>("read_date", { source, date });
}

export function converterCalendars(): Promise<ConverterCalendar[]> {
  return invoke<ConverterCalendar[]>("converter_calendars");
}

export type ConvertOutcome =
  { kind: "converted"; conversion: Conversion } | { kind: "refused"; error: DateError };

/** A date the calendar refuses comes back as `refused`; any other failure rejects. */
export async function convert(from: number, date: (number | null)[]): Promise<ConvertOutcome> {
  try {
    const conversion = await invoke<Conversion>("convert", { from, date });
    return { kind: "converted", conversion };
  } catch (error) {
    if (isDateError(error)) return { kind: "refused", error };
    throw error;
  }
}

export function isDateError(value: unknown): value is DateError {
  return typeof value === "object" && value !== null && "message" in value && "index" in value;
}
