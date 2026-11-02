import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

/** Matches `AppInfo` in crates/uni-app/src/lib.rs. */
interface AppInfo {
  name: string;
  version: string;
}

type Status = { kind: "loading" } | { kind: "ready"; info: AppInfo } | { kind: "unavailable" };

export default function App() {
  const [status, setStatus] = useState<Status>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    invoke<AppInfo>("app_info").then(
      (info) => !cancelled && setStatus({ kind: "ready", info }),
      () => !cancelled && setStatus({ kind: "unavailable" }),
    );
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="app">
      <h1>{status.kind === "ready" ? status.info.name : "Universe"}</h1>
      <p className="version">
        {status.kind === "ready" && status.info.version}
        {status.kind === "unavailable" && "Version unavailable"}
      </p>
      <p className="empty">No universe open</p>
    </main>
  );
}
