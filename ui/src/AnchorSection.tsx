import { useEffect, useRef, useState } from "react";

import {
  type AnchorBase,
  type CalendarDetails,
  type DateCheck,
  type FieldError,
  type Location,
  anchorBases,
  converterCalendars,
  readDate,
} from "./calendarApi";
import {
  type AnchorDraft,
  type Draft,
  WHOLE_NUMBER_MESSAGE,
  dateValues,
  fieldKey,
  formFromDraft,
  startingAnchor,
  unitsFromEra,
} from "./calendarDraft";
import { convertedDateText, valuesText } from "./convertedDate";

interface Props {
  /** The stored calendar's id, or null for a new one. */
  id: number | null;
  draft: Draft;
  /** The refusal from the last keep, if any. */
  error: FieldError | null;
  /** Another calendar of the universe can be placed, so an unanchored one can be anchored. */
  canAnchor: boolean;
  details: Pick<CalendarDetails, "rounded" | "note"> | null;
  onChange: (anchor: AnchorDraft | null) => void;
}

/** The result of checking one date as it's typed. */
type Check =
  | { kind: "none" }
  | { kind: "notWhole"; index: number }
  | { kind: "definition" }
  | { kind: "checked"; result: DateCheck };

export const DEFINITION_FIRST = "Checked once the calendar's definition is valid.";

/**
 * **Place in time** (spec 008, research R7): the calendar's anchor, entered as a base calendar
 * chosen from a list, an anchor date and a base date, each checked as it's typed and read
 * back as a sentence, and the anchor's kind. A calendar that isn't anchored says so, and can be
 * anchored when another calendar can be placed. An anchor, once kept, can't be removed.
 */
export default function AnchorSection({ id, draft, error, canAnchor, details, onChange }: Props) {
  const anchor = draft.anchor;
  const [bases, setBases] = useState<AnchorBase[] | null>(null);
  const [baseUnits, setBaseUnits] = useState<Map<number, string[]> | null>(null);
  const [own, setOwn] = useState<Check>({ kind: "none" });
  const [base, setBase] = useState<Check>({ kind: "none" });
  const anchored = anchor !== null;

  // The base list and every calendar's units, once there's an anchor to edit.
  const needBases = anchored && bases === null;
  const needUnits = anchored && baseUnits === null;
  useEffect(() => {
    if (!needBases) return;
    let cancelled = false;
    anchorBases(id).then(
      (list) => !cancelled && setBases(list),
      () => undefined,
    );
    return () => {
      cancelled = true;
    };
  }, [needBases, id]);
  useEffect(() => {
    if (!needUnits) return;
    let cancelled = false;
    converterCalendars().then(
      (calendars) => !cancelled && setBaseUnits(new Map(calendars.map((c) => [c.id, c.units]))),
      () => undefined,
    );
    return () => {
      cancelled = true;
    };
  }, [needUnits]);

  // The anchor date, against the draft's own definition.
  const definition = anchor === null ? "" : JSON.stringify({ ...draft, anchor: null, name: "" });
  const ownText = JSON.stringify(anchor?.date ?? null);
  const ownSeq = useRef(0);
  useEffect(() => {
    if (anchor === null) return;
    const seq = ++ownSeq.current;
    const settle = (check: Check) => {
      if (seq === ownSeq.current) setOwn(check);
    };
    const parsed = dateValues(anchor.date);
    if (parsed.bad !== null) return settle({ kind: "notWhole", index: parsed.bad });
    const { form } = formFromDraft({ ...draft, anchor: null });
    if (form === null) return settle({ kind: "definition" });
    readDate({ form }, parsed.values).then(
      (result) =>
        settle("definition" in result ? { kind: "definition" } : { kind: "checked", result }),
      () => settle({ kind: "none" }),
    );
    // `definition` and `ownText` stand for the draft and the date.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [definition, ownText]);

  // The base date, against the stored base calendar.
  const baseText = JSON.stringify([anchor?.base ?? null, anchor?.baseDate ?? null]);
  const baseSeq = useRef(0);
  useEffect(() => {
    if (anchor === null) return;
    const seq = ++baseSeq.current;
    const settle = (check: Check) => {
      if (seq === baseSeq.current) setBase(check);
    };
    const parsed = dateValues(anchor.baseDate);
    if (parsed.bad !== null) return settle({ kind: "notWhole", index: parsed.bad });
    if (anchor.base === null) return settle({ kind: "none" });
    readDate({ id: anchor.base }, parsed.values).then(
      (result) => settle({ kind: "checked", result }),
      () => settle({ kind: "none" }),
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [baseText]);

  if (anchor === null) {
    return (
      <fieldset className="anchor" data-field={fieldKey({ kind: "anchor" })} tabIndex={-1}>
        <legend>Place in time</legend>
        <p>Not anchored.</p>
        <Problem error={error} at={{ kind: "anchor" }} />
        <Problem error={error} at={{ kind: "anchor", field: "base" }} />
        {canAnchor && (
          <button
            type="button"
            onClick={() =>
              void anchorBases(id).then(
                (list) => {
                  setBases(list);
                  const first = list.find((b) => b.ineligible === null);
                  onChange(startingAnchor(first?.id ?? null));
                },
                () => undefined,
              )
            }
          >
            Anchor to a base calendar
          </button>
        )}
      </fieldset>
    );
  }

  const set = (change: Partial<AnchorDraft>) => onChange({ ...anchor, ...change });
  const chosen = bases?.find((b) => b.id === anchor.base);
  const baseName =
    chosen?.name ??
    (details?.note?.kind === "gone" ? details.note.name : null) ??
    "the base calendar";
  const ownUnits = unitsFromEra(draft);
  const units = anchor.base === null ? [] : (baseUnits?.get(anchor.base) ?? []);

  const sentence = readBack(draft.name.trim(), anchor, own, baseName, base);

  return (
    <fieldset className="anchor" data-field={fieldKey({ kind: "anchor" })} tabIndex={-1}>
      <legend>Place in time</legend>
      <Problem error={error} at={{ kind: "anchor" }} />

      <div className="field">
        <label>
          <span>Base calendar</span>
          <select
            aria-label="Base calendar"
            data-field={fieldKey({ kind: "anchor", field: "base" })}
            value={anchor.base ?? ""}
            onChange={(e) => set({ base: Number(e.target.value), baseDate: ["1"] })}
          >
            {(anchor.base === null || bases === null) && (
              <option value={anchor.base ?? ""} disabled>
                {anchor.base === null ? "Choose a base calendar" : baseName}
              </option>
            )}
            {bases?.map((b) => (
              <option key={b.id} value={b.id} disabled={b.ineligible !== null}>
                {b.name}
                {b.ineligible === "follows" && " (follows this calendar)"}
                {b.ineligible === "unplaceable" && " (can't be placed)"}
              </option>
            ))}
          </select>
        </label>
        <Problem error={error} at={{ kind: "anchor", field: "base" }} />
      </div>

      <DateFields
        label="Anchor date"
        field="date"
        units={ownUnits}
        values={anchor.date}
        check={own}
        error={error}
        onChange={(date) => set({ date })}
      />
      {own.kind === "definition" && <p className="hint">{DEFINITION_FIRST}</p>}

      <DateFields
        label="Base date"
        field="baseDate"
        units={units}
        values={anchor.baseDate}
        check={base}
        error={error}
        onChange={(baseDate) => set({ baseDate })}
      />

      {sentence !== null && <p className="read-back">{sentence}</p>}

      <div className="field" role="radiogroup" aria-label="Kind of anchor">
        <label>
          <input
            type="radio"
            name="anchor-kind"
            checked={anchor.kind === "fixed"}
            onChange={() => set({ kind: "fixed" })}
          />
          Fix the moment: stays put if {baseName} is edited
        </label>
        <label>
          <input
            type="radio"
            name="anchor-kind"
            checked={anchor.kind === "equation"}
            onChange={() => set({ kind: "equation" })}
          />
          Keep the equation: follows {baseName}
        </label>
      </div>

      {details?.rounded && (
        <p className="hint">
          {baseName}&apos;s date starts between two of the timeline&apos;s finest steps; this
          calendar is placed from the next one.
        </p>
      )}
      {details?.note?.kind === "moved" && (
        <p className="note">
          {baseName} has changed since: this moment is now{" "}
          {convertedDateText(details.note.now, "partway")} in it.
        </p>
      )}
      {details?.note?.kind === "gone" && (
        <p className="note">Its base calendar, {details.note.name}, has since been deleted.</p>
      )}
    </fieldset>
  );
}

/** "Tallow's cycle 1, round 1, watch 1 began at the start of Aster's orbit 412, span 3 (Third), sol 9." */
function readBack(
  name: string,
  anchor: AnchorDraft,
  own: Check,
  baseName: string,
  base: Check,
): string | null {
  const values = (check: Check, entered: string[]) => {
    if (check.kind !== "checked" || !("values" in check.result)) return null;
    const parsed = dateValues(entered);
    const count = parsed.values?.length ?? 0;
    return count === 0 ? null : valuesText(check.result.values.slice(0, count));
  };
  const ownText = values(own, anchor.date);
  const baseText = values(base, anchor.baseDate);
  if (ownText === null || baseText === null) return null;
  return `${name}'s ${ownText} began at the start of ${baseName}'s ${baseText}.`;
}

interface DateFieldsProps {
  label: string;
  field: "date" | "baseDate";
  units: string[];
  values: string[];
  check: Check;
  error: FieldError | null;
  onChange: (values: string[]) => void;
}

/** One whole-number field per unit from the era's unit down, each with its fault. */
function DateFields({ label, field, units, values, check, error, onChange }: DateFieldsProps) {
  const count = Math.max(units.length, values.length, 1);
  const live = (index: number): string | null => {
    if (check.kind === "notWhole") return check.index === index ? WHOLE_NUMBER_MESSAGE : null;
    if (check.kind !== "checked" || !("message" in check.result)) return null;
    return (check.result.index ?? 0) === index ? check.result.message : null;
  };
  return (
    <fieldset
      className="date-fields"
      aria-label={label}
      data-field={fieldKey({ kind: "anchor", field })}
      tabIndex={-1}
    >
      <legend>{label}</legend>
      <Problem error={error} at={{ kind: "anchor", field }} />
      {Array.from({ length: count }, (_, i) => {
        const unit = units[i] ?? `value ${i + 1}`;
        const at: Location = { kind: "anchor", field, index: i };
        const kept = error !== null && fieldKey(error.at) === fieldKey(at) ? error.message : null;
        const problem = live(i) ?? kept;
        const key = fieldKey(at);
        return (
          <div className="field" key={i}>
            <label>
              <span>{unit}</span>
              <input
                aria-label={`${label} ${unit}`}
                data-field={key}
                inputMode="numeric"
                value={values[i] ?? ""}
                aria-invalid={problem !== null ? true : undefined}
                aria-describedby={problem !== null ? `${key}-error` : undefined}
                onChange={(e) => {
                  const next = [...values];
                  while (next.length <= i) next.push("");
                  next[i] = e.target.value;
                  onChange(next);
                }}
              />
            </label>
            {problem !== null && (
              <p role="alert" className="error" id={`${key}-error`}>
                {problem}
              </p>
            )}
          </div>
        );
      })}
    </fieldset>
  );
}

function Problem({ error, at }: { error: FieldError | null; at: Location }) {
  if (error === null || fieldKey(error.at) !== fieldKey(at)) return null;
  return (
    <p role="alert" className="error">
      {error.message}
    </p>
  );
}
