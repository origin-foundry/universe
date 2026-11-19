import type { CalendarSummary } from "./calendarApi";

interface Props {
  calendars: CalendarSummary[];
  onAdd: () => void;
  onOpen: (id: number) => void;
  /** Opens the converter (spec 008). */
  onConvert?: () => void;
}

/** The open universe's calendars, in order (FR-001), each opening in the editor. */
export default function CalendarList({ calendars, onAdd, onOpen, onConvert }: Props) {
  // FR-008: the converter needs a calendar that can be placed.
  const canConvert = calendars.some((c) => c.problem === null);
  return (
    <section className="calendars">
      <h2>Calendars</h2>
      {calendars.length === 0 ? (
        <p className="hint">No calendars yet</p>
      ) : (
        <ul aria-label="Calendars">
          {calendars.map((c) => (
            <li key={c.id}>
              <button type="button" className="link" onClick={() => onOpen(c.id)}>
                {c.name}
              </button>
              {c.problem !== null && (
                <span className="problem"> Can&apos;t be used: {c.problem}</span>
              )}
            </li>
          ))}
        </ul>
      )}
      <div className="actions">
        <button type="button" onClick={onAdd}>
          Add calendar
        </button>
        {onConvert !== undefined && (
          <button type="button" disabled={!canConvert} onClick={onConvert}>
            Converter
          </button>
        )}
      </div>
    </section>
  );
}
