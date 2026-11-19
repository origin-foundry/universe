// Rendering a date the core converted (spec 008, research R10). The core sends only whole
// numbers and text, so nothing here computes a fraction: the precision only picks which of the
// core's forms to show.

import type { ConvertedDate, DateValue } from "./calendarApi";

/** How far into its smallest unit a moment is shown: the converter's switch (FR-010). */
export type Precision = "partway" | "decimal" | "exact";

export const PRECISIONS: { value: Precision; label: string }[] = [
  { value: "partway", label: "Partway" },
  { value: "decimal", label: "Decimal" },
  { value: "exact", label: "Exact" },
];

/** "span 3 (Third)", or "sol 9" for a value with no member name. */
export function valueText(v: DateValue, number: string = String(v.value)): string {
  return v.name === null ? `${v.unit} ${number}` : `${v.unit} ${number} (${v.name})`;
}

/** Values as a list: "orbit 412, span 3 (Third), sol 9". */
export function valuesText(values: DateValue[]): string {
  return values.map((v) => valueText(v)).join(", ");
}

/**
 * The whole date, the units above the era's unit first, with how far into the smallest unit
 * the moment falls at `precision`. A moment at a unit's start has no mark.
 */
export function convertedDateText(date: ConvertedDate, precision: Precision): string {
  const all = [...date.above, ...date.values];
  const last = all.at(-1);
  if (date.partway === null || last === undefined) return valuesText(all);
  const head = all.slice(0, -1).map((v) => valueText(v));
  const { decimal, decimalExact, exact } = date.partway;
  let tail: string;
  switch (precision) {
    case "partway":
      tail = `${valueText(last)}, partway through`;
      break;
    case "decimal": {
      // The core's decimal is "0.370": its digits go after the value's own.
      const digits = decimal.replace(/^-?0/, "");
      const number = last.value >= 0 ? `${last.value}${digits}` : `${last.value} + 0${digits}`;
      tail = valueText(last, decimalExact ? number : `≈ ${number}`);
      break;
    }
    case "exact":
      tail = valueText(last, `${last.value} + ${exact}`);
      break;
  }
  return [...head, tail].join(", ");
}
