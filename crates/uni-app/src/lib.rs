//! The Universe desktop app: Tauri commands, background jobs and progress events.

use serde::Serialize;

/// What the window shows about the app (specs/001-app-skeleton/contracts/app-info-command.md).
#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// The app's name and the workspace version, for the UI.
///
/// Not `pub`: Tauri can't generate its glue for public commands in the crate root.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "Universe",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// Builds and runs the app until its window is closed.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("Universe failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_reports_name_and_workspace_version() {
        let info = app_info();
        assert_eq!(info.name, "Universe");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    }
}
