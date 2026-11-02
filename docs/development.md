# Development setup

How to build and run Universe on Windows 10 or 11. Everything is installed natively; there is no Docker setup (see `specs/001-app-skeleton/spec.md`, Clarifications).

1. Install the [prerequisites](#prerequisites).
2. Run the [prerequisites check](#check-your-setup).
3. Use the [commands](#commands).

If any download fails with a certificate error, see [Certificate errors](#certificate-errors) first.

## Prerequisites

| Tool | Version | Why |
|---|---|---|
| [Python](#python) | 3.11 or later | Repository scripts and checks |
| [Node.js](#nodejs) | 22 or later (with npm) | The user interface and the Tauri command-line tool |
| [Rust](#rust) | The version pinned in `rust-toolchain.toml` | The core and the app |
| [C++ Build Tools](#c-build-tools) | Visual Studio 2022 Build Tools, "Desktop development with C++" | Rust uses Microsoft's linker on Windows |
| [WebView2](#webview2) | Any current version | The app's window renders its UI with it |

All commands below are typed into **PowerShell**: Start menu → type `PowerShell` → **Windows PowerShell**. Commands that start with `winget` use Windows' built-in package manager.

### Python

1. Check what you have: `python --version`. If it says 3.11 or later, skip to the next section.
2. Otherwise: `winget install --id Python.Python.3.13 --exact --source winget`
3. Close and reopen PowerShell, then check again with `python --version`.

### Node.js

1. Check what you have: `node --version`. If it says `v22` or later, skip to step 4.
2. Otherwise: `winget install --id OpenJS.NodeJS.LTS --exact --source winget`
3. Close and reopen PowerShell, then run `node --version` and `npm --version`.
4. If your network intercepts encrypted connections (antivirus web scanning or a company proxy), do the Node.js step under [Certificate errors](#certificate-errors) now.

### Rust

1. Install `rustup`, Rust's installer and version manager: `winget install --id Rustlang.Rustup --exact --source winget`
2. Close and reopen PowerShell.
3. Check it: `rustup --version`.
4. You don't need to choose a Rust version. The first time you build in this repository, `rustup` reads `rust-toolchain.toml` and installs exactly that version (currently **1.99.1**), with `rustfmt` and `clippy`.
5. If step 1 or the first build fails with a certificate error, see [Certificate errors](#certificate-errors).

### C++ Build Tools

Rust on Windows needs Microsoft's C++ linker and the Windows SDK. They come in the free Visual Studio Build Tools. You don't need the full Visual Studio. This is the largest download, several gigabytes, and can take a while.

1. Run (one line):

   ```powershell
   winget install --id Microsoft.VisualStudio.2022.BuildTools --exact --source winget --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
   ```

2. A Visual Studio Installer window opens and shows progress. It closes by itself when it's done.
3. If you already have Visual Studio or the Build Tools without C++: open **Visual Studio Installer** from the Start menu → **Modify** → tick **Desktop development with C++** → **Modify**.

### WebView2

Windows 11 and an up-to-date Windows 10 already have it. The [prerequisites check](#check-your-setup) tells you if it's missing. If it is:

1. `winget install --id Microsoft.EdgeWebView2Runtime --exact --source winget`

## Check your setup

From the repository folder:

```powershell
python scripts/check_prereqs.py
```

Every line should start with `ok`. Anything else names the tool and links to the section above that fixes it. The check never installs anything. `npm run dev` and `npm run app` run it automatically and stop if something is missing.

## Commands

Run from the repository folder.

| Command | What it does |
|---|---|
| `npm ci` | Installs the UI's dependencies exactly as locked. Run it once after cloning, and again whenever `package-lock.json` changes. |
| `npm run dev` | Starts the app in development mode. UI changes appear in the window as soon as you save; Rust changes rebuild and restart the app. The first run compiles everything and takes several minutes. |
| `npm run app` | Builds the optimised app (no installer) and starts it. |

## Certificate errors

Some antivirus products and company networks inspect encrypted (HTTPS) traffic by re-signing it with their own certificate. Windows trusts that certificate, but tools with their own certificate list don't. The symptoms are errors mentioning `certificate`, `UNABLE_TO_GET_ISSUER_CERT_LOCALLY`, `SELF_SIGNED_CERT_IN_CHAIN` or `invalid peer certificate`.

- **Node.js and npm**: tell Node to use Windows' certificate store, permanently for your user:

  ```powershell
  setx NODE_OPTIONS "--use-system-ca"
  ```

  Close and reopen PowerShell afterwards. Terminals inside an app that was already running (for example, the Claude app or VS Code) won't see it until you restart that app. Until then, set it for the current terminal only:

  ```powershell
  $env:NODE_OPTIONS = "--use-system-ca"
  ```
- **winget**: the commands in this guide all use `--source winget`. The Microsoft Store source (`msstore`) pins its certificate, so interception breaks it with `0x8a15005e: The server certificate did not match any of the expected values`. If you see that error, add `--source winget` to the command.
- **rustup** (installing or updating Rust): if it reports a certificate error, make it download through Windows' own TLS for this session, then retry:

  ```powershell
  $env:RUSTUP_USE_CURL = "1"
  ```

- **Cargo** (downloading Rust crates) uses Windows' certificate store already. If it fails with a *revocation* error, which some intercepting proxies cause, run this for the session and retry:

  ```powershell
  $env:CARGO_HTTP_CHECK_REVOKE = "false"
  ```

- **uv / Spec Kit** (only needed to upgrade Spec Kit): add `--system-certs`, e.g. `uv tool upgrade --system-certs specify-cli`.

The prerequisites check tests an HTTPS request through Node and prints a `warning` line if it hits a certificate error.
