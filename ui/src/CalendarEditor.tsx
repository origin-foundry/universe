import { message } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";

import {
  type CalendarForm,
  type FieldError,
  type Location,
  deleteCalendar,
  putCalendar,
} from "./calendarApi";
import {
  type Draft,
  draftFromForm,
  emptyGrouping,
  emptyMember,
  emptyRule,
  emptySubdivision,
  fieldKey,
  formFromDraft,
  moved,
  sameDraft,
} from "./calendarDraft";
import type { PendingEdit, UniverseState } from "./fileFlows";

interface Props {
  /** The stored calendar's id, or null for a new one. */
  id: number | null;
  initial: CalendarForm;
  /** The calendar was kept: the universe's new state and the calendar's id. */
  onKept: (state: UniverseState, id: number) => void;
  /** Back to the list, keeping nothing. */
  onClose: () => void;
  onDeleted: (state: UniverseState) => void;
  /** The edit in progress, for the keep-or-discard question (FR-012), or null if none. */
  onPending: (pending: PendingEdit | null) => void;
}

const DELETE = "Delete";
const CANCEL = "Cancel";

/**
 * The plain calendar editor (research R10): the calendar's name, its groupings top to bottom
 * with their members and rules, the fundamental unit, the subdivisions and the era. The form
 * is checked as a whole by the core when it's kept, and every refusal is shown at its field.
 */
export default function CalendarEditor({
  id,
  initial,
  onKept,
  onClose,
  onDeleted,
  onPending,
}: Props) {
  const [start] = useState(() => draftFromForm(initial));
  const [draft, setDraft] = useState(start);
  const [error, setError] = useState<FieldError | null>(null);
  // `keep` runs from the keep-or-discard question, outside a render: it reads the draft here.
  const latest = useRef(draft);
  useEffect(() => {
    latest.current = draft;
  }, [draft]);

  const edit = (change: (d: Draft) => void) =>
    setDraft((d) => {
      const next = structuredClone(d);
      change(next);
      return next;
    });

  const keep = useCallback(async (): Promise<UniverseState | null> => {
    const { form, error: local } = formFromDraft(latest.current);
    if (local !== null) {
      setError(local);
      return null;
    }
    try {
      const outcome = await putCalendar(id, form);
      if (outcome.kind === "refused") {
        setError(outcome.error);
        return null;
      }
      setError(null);
      onKept(outcome.state, outcome.id);
      return outcome.state;
    } catch (failure) {
      await message(String(failure), { title: "Universe", kind: "error" });
      return null;
    }
  }, [id, onKept]);

  const changed = !sameDraft(draft, start);
  useEffect(() => {
    onPending(
      changed ? { calendar: draft.name.trim() || initial.name, keep, discard: onClose } : null,
    );
  }, [changed, draft.name, initial.name, keep, onClose, onPending]);
  useEffect(() => () => onPending(null), [onPending]);

  // A refusal moves the focus to the input (or section) it concerns.
  useEffect(() => {
    if (error === null) return;
    const target = document.querySelector<HTMLElement>(`[data-field="${fieldKey(error.at)}"]`);
    target?.focus();
  }, [error]);

  async function remove() {
    if (id === null) return;
    const answer = await message(`Delete ${initial.name}?`, {
      title: "Universe",
      kind: "warning",
      buttons: { ok: DELETE, cancel: CANCEL },
    });
    if (answer !== DELETE && answer !== "Ok") return;
    try {
      onDeleted(await deleteCalendar(id));
    } catch (failure) {
      await message(String(failure), { title: "Universe", kind: "error" });
    }
  }

  const input = (
    label: string,
    aria: string,
    at: Location,
    value: string,
    set: (d: Draft, v: string) => void,
    placeholder?: string,
    showMessage = true,
  ) => (
    <Input
      label={label}
      aria={aria}
      at={at}
      error={error}
      value={value}
      placeholder={placeholder}
      showMessage={showMessage}
      onChange={(v) => edit((d) => set(d, v))}
    />
  );

  const eraUnit = draft.era.unit.trim();

  return (
    <form
      className="calendar-editor"
      aria-label="Calendar editor"
      onSubmit={(event) => {
        event.preventDefault();
        void keep();
      }}
    >
      <h2>{id === null ? "New calendar" : initial.name}</h2>
      {input("Calendar name", "Calendar name", { kind: "calendarName" }, draft.name, (d, v) => {
        d.name = v;
      })}

      <fieldset>
        <legend>Groupings, top to bottom</legend>
        {draft.groupings.map((g, i) => {
          const n = i + 1;
          const below = draft.groupings[i + 1]?.name.trim() || draft.fundamental.name.trim();
          const showRules =
            g.rules.length > 0 || (g.name.trim() !== "" && g.name.trim() === eraUnit);
          return (
            <fieldset
              key={i}
              className="grouping"
              aria-label={`Grouping ${n}`}
              data-field={fieldKey({ kind: "grouping", index: i })}
              tabIndex={-1}
            >
              <legend>Grouping {n}</legend>
              <SectionError at={{ kind: "grouping", index: i }} error={error} />
              {input(
                "Name",
                `Grouping ${n} name`,
                { kind: "grouping", index: i, field: "name" },
                g.name,
                (d, v) => {
                  d.groupings[i]!.name = v;
                },
              )}
              {input(
                "First number",
                `Grouping ${n} first number`,
                { kind: "grouping", index: i, field: "firstNumber" },
                g.firstNumber,
                (d, v) => {
                  d.groupings[i]!.firstNumber = v;
                },
                "1",
              )}
              {input(
                "Member unit",
                `Grouping ${n} member unit name`,
                { kind: "grouping", index: i, field: "memberUnit" },
                g.memberUnitName,
                (d, v) => {
                  d.groupings[i]!.memberUnitName = v;
                },
                undefined,
                false, // the member unit's message goes after its first number
              )}
              {input(
                "Member unit's first number",
                `Grouping ${n} member unit first number`,
                { kind: "grouping", index: i, field: "memberUnit" },
                g.memberUnitFirstNumber,
                (d, v) => {
                  d.groupings[i]!.memberUnitFirstNumber = v;
                },
                "1",
              )}
              <ListButtons
                what={`grouping ${n}`}
                index={i}
                length={draft.groupings.length}
                onMove={(to) => edit((d) => (d.groupings = moved(d.groupings, i, to)))}
                onRemove={() => edit((d) => void d.groupings.splice(i, 1))}
              />

              <h3>Members</h3>
              {below !== "" && <p className="hint">Each member&apos;s count is in {below}.</p>}
              <ol>
                {g.members.map((m, j) => {
                  const at = { kind: "member", grouping: i, index: j } as const;
                  const label = `Grouping ${n} member ${j + 1}`;
                  return (
                    <li key={j} data-field={fieldKey(at)} tabIndex={-1}>
                      <SectionError at={at} error={error} />
                      {input(
                        "Name (optional)",
                        `${label} name`,
                        { ...at, field: "name" },
                        m.name,
                        (d, v) => {
                          d.groupings[i]!.members[j]!.name = v;
                        },
                      )}
                      {input(
                        "Count",
                        `${label} count`,
                        { ...at, field: "count" },
                        m.count,
                        (d, v) => {
                          d.groupings[i]!.members[j]!.count = v;
                        },
                      )}
                      <ListButtons
                        what={`member ${j + 1} of grouping ${n}`}
                        index={j}
                        length={g.members.length}
                        onMove={(to) =>
                          edit(
                            (d) =>
                              (d.groupings[i]!.members = moved(d.groupings[i]!.members, j, to)),
                          )
                        }
                        onRemove={() => edit((d) => void d.groupings[i]!.members.splice(j, 1))}
                      />
                    </li>
                  );
                })}
              </ol>
              <button
                type="button"
                onClick={() => edit((d) => void d.groupings[i]!.members.push(emptyMember()))}
              >
                Add member to grouping {n}
              </button>

              {showRules && (
                <>
                  <h3>Adjustment rules</h3>
                  <p className="hint">
                    In each {g.name.trim() || "instance"} whose number makes the condition true, the
                    member is longer by the change (shorter when it&apos;s negative).
                  </p>
                  <ol>
                    {g.rules.map((r, k) => {
                      const at = { kind: "rule", grouping: i, index: k } as const;
                      const label = `Grouping ${n} rule ${k + 1}`;
                      return (
                        <li key={k} data-field={fieldKey(at)} tabIndex={-1}>
                          <SectionError at={at} error={error} />
                          <Input
                            label="Condition"
                            aria={`${label} condition`}
                            at={{ ...at, field: "condition" }}
                            error={error}
                            value={r.condition}
                            multiline
                            onChange={(v) =>
                              edit((d) => {
                                d.groupings[i]!.rules[k]!.condition = v;
                              })
                            }
                          />
                          {input(
                            "Member",
                            `${label} member`,
                            { ...at, field: "member" },
                            r.member,
                            (d, v) => {
                              d.groupings[i]!.rules[k]!.member = v;
                            },
                          )}
                          {input(
                            "Change",
                            `${label} change`,
                            { ...at, field: "change" },
                            r.change,
                            (d, v) => {
                              d.groupings[i]!.rules[k]!.change = v;
                            },
                          )}
                          <ListButtons
                            what={`rule ${k + 1} of grouping ${n}`}
                            index={k}
                            length={g.rules.length}
                            onMove={(to) =>
                              edit(
                                (d) =>
                                  (d.groupings[i]!.rules = moved(d.groupings[i]!.rules, k, to)),
                              )
                            }
                            onRemove={() => edit((d) => void d.groupings[i]!.rules.splice(k, 1))}
                          />
                        </li>
                      );
                    })}
                  </ol>
                  <button
                    type="button"
                    onClick={() => edit((d) => void d.groupings[i]!.rules.push(emptyRule()))}
                  >
                    Add adjustment rule to grouping {n}
                  </button>
                </>
              )}
            </fieldset>
          );
        })}
        <button type="button" onClick={() => edit((d) => void d.groupings.push(emptyGrouping()))}>
          Add grouping
        </button>
      </fieldset>

      <fieldset data-field={fieldKey({ kind: "fundamental" })} tabIndex={-1}>
        <legend>Fundamental unit</legend>
        <SectionError at={{ kind: "fundamental" }} error={error} />
        {input(
          "Name",
          "Fundamental unit name",
          { kind: "fundamental", field: "name" },
          draft.fundamental.name,
          (d, v) => {
            d.fundamental.name = v;
          },
        )}
        {input(
          "Length, in SI seconds (physical)",
          "Length, in SI seconds (physical)",
          { kind: "fundamental", field: "length" },
          draft.fundamental.length,
          (d, v) => {
            d.fundamental.length = v;
          },
        )}
        <p className="hint">
          A whole number, decimal or fraction, such as 88775.244 or 355/113. It&apos;s kept exactly.
        </p>
        {input(
          "First number",
          "Fundamental unit first number",
          { kind: "fundamental", field: "firstNumber" },
          draft.fundamental.firstNumber,
          (d, v) => {
            d.fundamental.firstNumber = v;
          },
          "1",
        )}
      </fieldset>

      <fieldset>
        <legend>Subdivisions, top to bottom</legend>
        {draft.subdivisions.map((s, i) => {
          const at = { kind: "subdivision", index: i } as const;
          const label = `Subdivision ${i + 1}`;
          return (
            <fieldset key={i} aria-label={label} data-field={fieldKey(at)} tabIndex={-1}>
              <legend>{label}</legend>
              <SectionError at={at} error={error} />
              {input("Name", `${label} name`, { ...at, field: "name" }, s.name, (d, v) => {
                d.subdivisions[i]!.name = v;
              })}
              {input("Parts", `${label} parts`, { ...at, field: "parts" }, s.parts, (d, v) => {
                d.subdivisions[i]!.parts = v;
              })}
              {input(
                "First number",
                `${label} first number`,
                { ...at, field: "firstNumber" },
                s.firstNumber,
                (d, v) => {
                  d.subdivisions[i]!.firstNumber = v;
                },
                "0",
              )}
              <ListButtons
                what={`subdivision ${i + 1}`}
                index={i}
                length={draft.subdivisions.length}
                onMove={(to) => edit((d) => (d.subdivisions = moved(d.subdivisions, i, to)))}
                onRemove={() => edit((d) => void d.subdivisions.splice(i, 1))}
              />
            </fieldset>
          );
        })}
        <button
          type="button"
          onClick={() => edit((d) => void d.subdivisions.push(emptySubdivision()))}
        >
          Add subdivision
        </button>
      </fieldset>

      <fieldset>
        <legend>Era</legend>
        {input("Name", "Era name", { kind: "era", field: "name" }, draft.era.name, (d, v) => {
          d.era.name = v;
        })}
        {input(
          "Era's unit",
          "Era's unit",
          { kind: "era", field: "unit" },
          draft.era.unit,
          (d, v) => {
            d.era.unit = v;
          },
        )}
      </fieldset>

      <div className="actions">
        <button type="submit">Keep changes</button>
        <button type="button" onClick={onClose}>
          Discard changes
        </button>
        {id !== null && (
          <button type="button" onClick={() => void remove()}>
            Delete calendar
          </button>
        )}
      </div>
    </form>
  );
}

function errorAt(error: FieldError | null, at: Location): string | null {
  return error !== null && fieldKey(error.at) === fieldKey(at) ? error.message : null;
}

interface InputProps {
  label: string;
  /** The input's accessible name, unique in the form. */
  aria: string;
  at: Location;
  error: FieldError | null;
  value: string;
  /** For an optional number: the engine's default, shown greyed (FR-007). */
  placeholder?: string;
  multiline?: boolean;
  /** False when another input of the same location shows the message. */
  showMessage?: boolean;
  onChange: (value: string) => void;
}

function Input({
  label,
  aria,
  at,
  error,
  value,
  placeholder,
  multiline,
  showMessage = true,
  onChange,
}: InputProps) {
  const key = fieldKey(at);
  const problem = errorAt(error, at);
  const common = {
    "aria-label": aria,
    "data-field": key,
    "aria-invalid": problem !== null ? true : undefined,
    "aria-describedby": problem !== null ? `${key}-error` : undefined,
    value,
    placeholder,
  };
  return (
    <div className="field">
      <label>
        <span>{label}</span>
        {multiline ? (
          <textarea rows={2} {...common} onChange={(e) => onChange(e.target.value)} />
        ) : (
          <input {...common} onChange={(e) => onChange(e.target.value)} />
        )}
      </label>
      {problem !== null && showMessage && (
        <p role="alert" className="error" id={`${key}-error`}>
          {problem}
        </p>
      )}
    </div>
  );
}

/** A refusal that concerns a whole section rather than one input. */
function SectionError({ at, error }: { at: Location; error: FieldError | null }): ReactNode {
  const problem = errorAt(error, at);
  return problem === null ? null : (
    <p role="alert" className="error">
      {problem}
    </p>
  );
}

interface ListButtonsProps {
  /** e.g. "member 2 of grouping 3", for the buttons' accessible names. */
  what: string;
  index: number;
  length: number;
  onMove: (to: number) => void;
  onRemove: () => void;
}

function ListButtons({ what, index, length, onMove, onRemove }: ListButtonsProps) {
  return (
    <div className="list-buttons">
      <button
        type="button"
        aria-label={`Move ${what} up`}
        disabled={index === 0}
        onClick={() => onMove(index - 1)}
      >
        ↑
      </button>
      <button
        type="button"
        aria-label={`Move ${what} down`}
        disabled={index === length - 1}
        onClick={() => onMove(index + 1)}
      >
        ↓
      </button>
      <button type="button" aria-label={`Remove ${what}`} onClick={onRemove}>
        Remove
      </button>
    </div>
  );
}
