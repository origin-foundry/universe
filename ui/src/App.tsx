import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  NO_UNIVERSE,
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

export default function App() {
  const [status, setStatus] = useState<Status>({ kind: "loading" });
  const [universe, setUniverse] = useState<UniverseState>(NO_UNIVERSE);
  const [naming, setNaming] = useState(false);

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
      open: () => void run(openFlow),
      save: () => void run(saveFlow),
      "save-as": () => void run(saveAsFlow),
      close: () =>
        void run(async (state) => {
          const next = await closeFlow(state);
          if (next !== null) setNaming(false);
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

  const heading = universe.open
    ? universe.name
    : status.kind === "ready"
      ? status.info.name
      : "Universe";

  return (
    <main className="app">
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
          }}
          onCancel={() => setNaming(false)}
        />
      ) : (
        !universe.open && (
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
        )
      )}
    </main>
  );
}
