import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  type CalendarForm,
  type CalendarSummary,
  getCalendar,
  listCalendars,
  newCalendarForm,
} from "./calendarApi";
import CalendarEditor from "./CalendarEditor";
import CalendarList from "./CalendarList";
import {
  NO_UNIVERSE,
  type PendingEdit,
  type UniverseState,
  closeFlow,
  closeWindowFlow,
  createUniverse,
  newFlow,
  openFlow,
  saveAsFlow,
  saveFlow,
  windowTitle,
} from "./fileFlows";
import NewUniverseForm from "./NewUniverseForm";
import { type MenuAction, NEEDS_OPEN_UNIVERSE, shortcutFor } from "./shortcuts";

/** Matches `AppInfo` in crates/uni-app/src/lib.rs. */
interface AppInfo {
  name: string;
  version: string;
}

type Status = { kind: "loading" } | { kind: "ready"; info: AppInfo } | { kind: "unavailable" };

/** The main area while a universe is open: its calendar list, or one calendar in the editor. */
type View =
  { kind: "list" } | { kind: "editor"; id: number | null; initial: CalendarForm; opened: number };

export default function App() {
  const [status, setStatus] = useState<Status>({ kind: "loading" });
  const [universe, setUniverse] = useState<UniverseState>(NO_UNIVERSE);
  const [naming, setNaming] = useState(false);
  const [calendars, setCalendars] = useState<CalendarSummary[]>([]);
  const [view, setView] = useState<View>({ kind: "list" });
  // The calendar edit in progress, if it has changes (FR-012).
  const pending = useRef<PendingEdit | null>(null);
  const opened = useRef(0);

  // Menu and close events arrive outside React; they read the latest state from here.
  const current = useRef(universe);
  // One flow at a time, so a held-down shortcut can't open two dialogs.
  const busy = useRef(false);

  const run = useCallback(async (flow: (state: UniverseState) => Promise<UniverseState | null>) => {
    if (busy.current) return;
    busy.current = true;
    try {
      const next = await flow(current.current);
      if (next !== null) setUniverse(next);
    } finally {
      busy.current = false;
    }
  }, []);

  const startNew = useCallback(async () => {
    if (busy.current) return;
    busy.current = true;
    try {
      if (await newFlow(current.current)) setNaming(true);
    } finally {
      busy.current = false;
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    invoke<AppInfo>("app_info").then(
      (info) => !cancelled && setStatus({ kind: "ready", info }),
      () => !cancelled && setStatus({ kind: "unavailable" }),
    );
    invoke<UniverseState>("universe_state").then(
      (state) => !cancelled && state && setUniverse(state),
      () => undefined,
    );
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const flows: Record<MenuAction, () => void> = {
      new: () => void startNew(),
      open: () =>
        void run(async (state) => {
          const next = await openFlow(state);
          if (next !== null) setView({ kind: "list" });
          return next;
        }),
      save: () => void run(saveFlow),
      "save-as": () => void run(saveAsFlow),
      close: () =>
        void run(async (state) => {
          const next = await closeFlow(state);
          if (next !== null) {
            setNaming(false);
            setView({ kind: "list" });
          }
          return next;
        }),
    };
    const unlisten = [
      listen<string>("menu", (event) => flows[event.payload as MenuAction]?.()),
      listen("close-requested", () => void closeWindowFlow(current.current)),
    ];
    // The native menu never sees its shortcuts while the web view has the focus
    // (see shortcuts.ts), so they're handled here.
    const onKeyDown = (event: KeyboardEvent) => {
      const action = shortcutFor(event);
      if (action === null) return;
      event.preventDefault(); // e.g. Ctrl+O, Ctrl+S and Ctrl+N are also browser commands
      if (event.repeat) return;
      if (NEEDS_OPEN_UNIVERSE.has(action) && !current.current.open) return;
      flows[action]();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      unlisten.forEach((p) =>
        p.then(
          (stop) => stop?.(),
          () => undefined,
        ),
      );
    };
  }, [run, startNew]);

  useEffect(() => {
    current.current = universe;
    document.title = windowTitle(universe);
  }, [universe]);

  // The calendar list follows the open universe.
  useEffect(() => {
    let cancelled = false;
    if (!universe.open) return;
    listCalendars().then(
      (list) => !cancelled && setCalendars(list),
      () => !cancelled && setCalendars([]),
    );
    return () => {
      cancelled = true;
    };
  }, [universe]);

  const edit = useCallback(async (load: () => Promise<CalendarForm>, id: number | null) => {
    try {
      const initial = await load();
      opened.current += 1;
      setView({ kind: "editor", id, initial, opened: opened.current });
    } catch {
      setView({ kind: "list" });
    }
  }, []);
  const backToList = useCallback(() => setView({ kind: "list" }), []);
  const onPending = useCallback((p: PendingEdit | null) => {
    pending.current = p;
  }, []);
  const afterChange = useCallback((state: UniverseState) => {
    setUniverse(state);
    setView({ kind: "list" });
  }, []);

  const heading = universe.open
    ? universe.name
    : status.kind === "ready"
      ? status.info.name
      : "Universe";

  return (
    <main className={universe.open && !naming ? "app workspace" : "app"}>
      <h1>{heading}</h1>
      <p className="version">
        {status.kind === "ready" && status.info.version}
        {status.kind === "unavailable" && "Version unavailable"}
      </p>
      {naming ? (
        <NewUniverseForm
          onCreate={async (name) => {
            setUniverse(await createUniverse(name));
            setNaming(false);
            setView({ kind: "list" });
          }}
          onCancel={() => setNaming(false)}
        />
      ) : universe.open ? (
        view.kind === "editor" ? (
          <CalendarEditor
            key={view.opened}
            id={view.id}
            initial={view.initial}
            onKept={afterChange}
            onClose={backToList}
            onDeleted={afterChange}
            onPending={onPending}
          />
        ) : (
          <CalendarList
            calendars={calendars}
            onAdd={() => void edit(newCalendarForm, null)}
            onOpen={(id) => void edit(() => getCalendar(id), id)}
          />
        )
      ) : (
        <section className="empty">
          <p>No universe open</p>
          <div className="actions">
            <button type="button" onClick={() => void startNew()}>
              New universe
            </button>
            <button type="button" onClick={() => void run(openFlow)}>
              Open universe…
            </button>
          </div>
        </section>
      )}
    </main>
  );
}
