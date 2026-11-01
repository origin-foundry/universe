# Contract: `app_info` Tauri command

The single call from the UI into the Rust core in this feature (FR-007).

**Invocation (UI)**: `invoke<AppInfo>("app_info")`, with no arguments.

**Response**:

```ts
interface AppInfo {
  name: string;    // "Universe"
  version: string; // the workspace version from Cargo.toml, e.g. "0.1.0"
}
```

**Errors**: none expected. If the call fails, the window shows "Version unavailable" in place of the version and still shows "No universe open". It never shows a blank screen.

**Tests**:

- **Rust:** a unit test that `app_info()` returns the name "Universe" and the crate's `CARGO_PKG_VERSION`.
- **UI:** a component test with `invoke` mocked, checking that the window shows "Universe", the mocked version and "No universe open". A second test makes `invoke` reject and checks for "Version unavailable".
