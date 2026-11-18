// The calendar commands and their shapes
// (specs/007-calendars-in-project/contracts/calendar-commands.md). They match
// crates/uni-app/src/calendars.rs.

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

export interface CalendarForm {
  name: string;
  groupings: GroupingForm[];
  fundamental: FundamentalForm;
  subdivisions: SubdivisionForm[];
  era: { name: string; unit: string };
}

export type Location =
  | { kind: "calendarName" }
  | { kind: "era"; field: "name" | "unit" }
  | { kind: "grouping"; index: number; field?: "name" | "firstNumber" | "memberUnit" }
  | { kind: "member"; grouping: number; index: number; field?: "name" | "count" }
  | { kind: "rule"; grouping: number; index: number; field?: "condition" | "member" | "change" }
  | { kind: "fundamental"; field?: "name" | "length" | "firstNumber" }
  | { kind: "subdivision"; index: number; field?: "name" | "parts" | "firstNumber" };

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

export function getCalendar(id: number): Promise<CalendarForm> {
  return invoke<CalendarForm>("get_calendar", { id });
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
