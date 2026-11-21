import { message } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";

import {
  type Conversion,
  type ConverterCalendar,
  type DateError,
  convert,
  converterCalendars,
} from "./calendarApi";
import { WHOLE_NUMBER_MESSAGE, dateValues } from "./calendarDraft";
import { PRECISIONS, type Precision, convertedDateText } from "./convertedDate";

interface Props {
  /** The precision switch, held by the app so it lasts while the app is open (FR-010a). */
  precision: Precision;
  onPrecision: (precision: Precision) => void;
  onBack: () => void;
}

type Result =
  | { kind: "none" }
  | { kind: "notWhole"; index: number }
  | { kind: "refused"; error: DateError }
  | { kind: "converted"; conversion: Conversion };

export const NO_OTHER = "There's no other calendar to convert into.";
export const ROUNDED =
  "This date starts between two of the timeline's finest steps; it's shown from the next one.";

/**
 * The converter (spec 008, research R9): a date entered in one calendar, value by value, and
 * the same moment in every other calendar, converted as the owner types. It has no unsaved
 * state.
 */
export default function Converter({ precision, onPrecision, onBack }: Props) {
  const [calendars, setCalendars] = useState<ConverterCalendar[] | null>(null);
  const [from, setFrom] = useState<number | null>(null);
  const [values, setValues] = useState<string[]>([]);
  const [result, setResult] = useState<Result>({ kind: "none" });
  const seq = useRef(0);

  useEffect(() => {
    let cancelled = false;
    converterCalendars().then(
      (list) => {
        if (cancelled) return;
        setCalendars(list);
        setFrom(list.find((c) => c.problem === null)?.id ?? null);
      },
      (failure) => void message(String(failure), { title: "Universe", kind: "error" }),
    );
    return () => {
      cancelled = true;
    };
  }, []);

  const source = calendars?.find((c) => c.id === from) ?? null;

  // Converts whenever every filled field holds a whole number.
  const key = JSON.stringify([from, values]);
  useEffect(() => {
    const mine = ++seq.current;
    const settle = (next: Result) => {
      if (mine === seq.current) setResult(next);
    };
    if (from === null) return settle({ kind: "none" });
    const parsed = dateValues(values);
    if (parsed.bad !== null) return settle({ kind: "notWhole", index: parsed.bad });
    if (parsed.values.length === 0) return settle({ kind: "none" });
    convert(from, parsed.values).then(
      (outcome) =>
        settle(
          outcome.kind === "refused"
            ? { kind: "refused", error: outcome.error }
            : { kind: "converted", conversion: outcome.conversion },
        ),
      (failure) => {
        settle({ kind: "none" });
        void message(String(failure), { title: "Universe", kind: "error" });
      },
    );
    // `key` stands for the calendar and the values.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);

  const problemAt = (index: number): string | null => {
    if (result.kind === "notWhole") return result.index === index ? WHOLE_NUMBER_MESSAGE : null;
    if (result.kind === "refused")
      return (result.error.index ?? 0) === index ? result.error.message : null;
    return null;
  };

  const units = source?.units ?? [];
  return (
    <section className="converter">
      <h2>Converter</h2>
      <div className="actions">
        <button type="button" onClick={onBack}>
          Back to calendars
        </button>
      </div>
      {calendars !== null && (
        <>
          <div className="field">
            <label>
              <span>Calendar</span>
              <select
                aria-label="Calendar"
                value={from ?? ""}
                onChange={(e) => {
                  setFrom(Number(e.target.value));
                  setValues([]);
                }}
              >
                {calendars.map((c) => (
                  <option key={c.id} value={c.id} disabled={c.problem !== null}>
                    {c.name}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <fieldset className="date-fields" aria-label="Date">
            <legend>Date</legend>
            {units.map((unit, i) => {
              const problem = problemAt(i);
              return (
                <div className="field" key={`${from}-${i}`}>
                  <label>
                    <span>{unit}</span>
                    <input
                      aria-label={unit}
                      inputMode="numeric"
                      value={values[i] ?? ""}
                      aria-invalid={problem !== null ? true : undefined}
                      onChange={(e) => {
                        const next = [...values];
                        while (next.length <= i) next.push("");
                        next[i] = e.target.value;
                        setValues(next);
                      }}
                    />
                  </label>
                  {problem !== null && (
                    <p role="alert" className="error">
                      {problem}
                    </p>
                  )}
                </div>
              );
            })}
          </fieldset>
          <div className="field" role="radiogroup" aria-label="Precision">
            <span>Precision</span>
            {PRECISIONS.map((p) => (
              <label key={p.value}>
                <input
                  type="radio"
                  name="precision"
                  checked={precision === p.value}
                  onChange={() => onPrecision(p.value)}
                />
                {p.label}
              </label>
            ))}
          </div>
          <Results result={result} precision={precision} />
        </>
      )}
    </section>
  );
}

function Results({ result, precision }: { result: Result; precision: Precision }) {
  if (result.kind !== "converted") return null;
  const { rounded, results, unusable } = result.conversion;
  return (
    <section aria-label="Results" className="results">
      {rounded && <p className="hint">{ROUNDED}</p>}
      {results.length === 0 && unusable.length === 0 && <p className="hint">{NO_OTHER}</p>}
      {results.length > 0 && (
        <ul>
          {results.map((r) => (
            <li key={r.id}>
              <strong>{r.name}:</strong> <span>{convertedDateText(r.date, precision)}</span>
            </li>
          ))}
        </ul>
      )}
      {unusable.length > 0 && (
        <ul aria-label="Calendars that can't be placed">
          {unusable.map((u) => (
            <li key={u.name} className="problem">
              {u.name} can&apos;t be used: {u.problem}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
