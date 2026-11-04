//! The Universe desktop app: Tauri commands, background jobs and progress events.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use tauri::menu::{Menu, MenuBuilder, MenuItem, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent, Wry};
use uni_store::{OpenUniverse, UniverseName};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        version: APP_VERSION,
    }
}

/// What the UI is told about the open universe
/// (specs/003-project-file/contracts/store-and-commands.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UniverseState {
    pub open: bool,
    pub name: Option<String>,
    pub path: Option<String>,
    pub dirty: bool,
}

impl UniverseState {
    /// The window title: `Aster — Universe`, with `•` after the name while there are unsaved
    /// changes, or just `Universe` when nothing is open.
    pub fn window_title(&self) -> String {
        match (&self.name, self.dirty) {
            (Some(name), false) => format!("{name} — Universe"),
            (Some(name), true) => format!("{name} • — Universe"),
            (None, _) => "Universe".to_owned(),
        }
    }
}

/// Holds the one open universe. Independent of Tauri, so it can be tested on its own.
///
/// Errors are the owner-facing [`uni_store::StoreError`] messages.
#[derive(Default)]
pub struct Session {
    universe: Mutex<Option<OpenUniverse>>,
}

impl Session {
    fn lock(&self) -> MutexGuard<'_, Option<OpenUniverse>> {
        // A panic elsewhere can't leave the universe half-changed, so carry on with it.
        self.universe
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn describe(universe: &Option<OpenUniverse>) -> UniverseState {
        match universe {
            None => UniverseState {
                open: false,
                name: None,
                path: None,
                dirty: false,
            },
            Some(u) => UniverseState {
                open: true,
                name: u.name().ok().map(|n| n.as_str().to_owned()),
                path: u.path().map(|p| p.to_string_lossy().into_owned()),
                dirty: u.is_dirty(),
            },
        }
    }

    /// The current state.
    pub fn state(&self) -> UniverseState {
        Self::describe(&self.lock())
    }

    /// Replaces any open universe with a new one called `name`.
    pub fn new_universe(&self, name: &str) -> Result<UniverseState, String> {
        let name = UniverseName::new(name).map_err(|e| e.to_string())?;
        let universe = OpenUniverse::new(name, APP_VERSION).map_err(|e| e.to_string())?;
        let mut slot = self.lock();
        *slot = Some(universe);
        Ok(Self::describe(&slot))
    }

    /// Replaces any open universe with the one in the file at `path`.
    pub fn open(&self, path: &str) -> Result<UniverseState, String> {
        let mut slot = self.lock();
        let reopening = slot
            .as_ref()
            .and_then(OpenUniverse::path)
            .is_some_and(|p| p == Path::new(path));
        if reopening {
            // Our own lock would make the file look in use: let go of it first.
            *slot = None;
        }
        let universe =
            OpenUniverse::open(Path::new(path), APP_VERSION).map_err(|e| e.to_string())?;
        *slot = Some(universe);
        Ok(Self::describe(&slot))
    }

    /// Saves the open universe to its file.
    pub fn save(&self) -> Result<UniverseState, String> {
        let mut slot = self.lock();
        let universe = slot.as_mut().ok_or("No universe is open.")?;
        universe.save().map_err(|e| e.to_string())?;
        Ok(Self::describe(&slot))
    }

    /// Saves the open universe to `path` and carries on in that file.
    pub fn save_as(&self, path: &str) -> Result<UniverseState, String> {
        let mut slot = self.lock();
        let universe = slot.as_mut().ok_or("No universe is open.")?;
        universe
            .save_as(Path::new(path))
            .map_err(|e| e.to_string())?;
        Ok(Self::describe(&slot))
    }

    /// Closes the open universe, discarding unsaved changes (the UI has already asked).
    pub fn close(&self) -> UniverseState {
        let mut slot = self.lock();
        *slot = None;
        Self::describe(&slot)
    }
}

/// The File menu items whose availability follows the state.
struct FileMenu {
    save: MenuItem<Wry>,
    save_as: MenuItem<Wry>,
    close: MenuItem<Wry>,
}

/// Brings the window title and the File menu in line with `state`, and passes it on.
fn reflect(app: &AppHandle, state: UniverseState) -> UniverseState {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(&state.window_title());
    }
    if let Some(menu) = app.try_state::<FileMenu>() {
        for item in [&menu.save, &menu.save_as, &menu.close] {
            let _ = item.set_enabled(state.open);
        }
    }
    state
}

fn reflect_result(
    app: &AppHandle,
    result: Result<UniverseState, String>,
) -> Result<UniverseState, String> {
    result.map(|state| reflect(app, state))
}

#[tauri::command]
fn universe_state(session: State<'_, Session>) -> UniverseState {
    session.state()
}

#[tauri::command]
fn new_universe(
    app: AppHandle,
    session: State<'_, Session>,
    name: String,
) -> Result<UniverseState, String> {
    reflect_result(&app, session.new_universe(&name))
}

#[tauri::command]
fn open_universe(
    app: AppHandle,
    session: State<'_, Session>,
    path: String,
) -> Result<UniverseState, String> {
    reflect_result(&app, session.open(&path))
}

#[tauri::command]
fn save_universe(app: AppHandle, session: State<'_, Session>) -> Result<UniverseState, String> {
    reflect_result(&app, session.save())
}

#[tauri::command]
fn save_universe_as(
    app: AppHandle,
    session: State<'_, Session>,
    path: String,
) -> Result<UniverseState, String> {
    reflect_result(&app, session.save_as(&path))
}

#[tauri::command]
fn close_universe(app: AppHandle, session: State<'_, Session>) -> UniverseState {
    reflect(&app, session.close())
}

/// Ends the app once the UI has finished the unsaved-changes prompt on closing the window.
///
/// A command rather than the window-close permission, so the UI's capabilities stay at the
/// three dialogs.
#[tauri::command]
fn exit_app(app: AppHandle, session: State<'_, Session>) {
    session.close();
    app.exit(0);
}

/// File menu item ids. Each one but Exit is sent to the UI as a `menu` event.
const MENU_EVENTS: [&str; 5] = ["new", "open", "save", "save-as", "close"];
const EXIT: &str = "exit";

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let item = |id: &str, text: &str, accelerator: &str, enabled: bool| {
        MenuItemBuilder::with_id(id, text)
            .accelerator(accelerator)
            .enabled(enabled)
            .build(app)
    };
    let save = item("save", "&Save", "CmdOrCtrl+S", false)?;
    let save_as = item("save-as", "Save &As…", "CmdOrCtrl+Shift+S", false)?;
    let close = item("close", "&Close", "CmdOrCtrl+W", false)?;
    let file = SubmenuBuilder::new(app, "&File")
        .item(&item("new", "&New universe", "CmdOrCtrl+N", true)?)
        .item(&item("open", "&Open universe…", "CmdOrCtrl+O", true)?)
        .separator()
        .item(&save)
        .item(&save_as)
        .separator()
        .item(&close)
        .separator()
        .item(&MenuItemBuilder::with_id(EXIT, "E&xit").build(app)?)
        .build()?;
    app.manage(FileMenu {
        save,
        save_as,
        close,
    });
    MenuBuilder::new(app).item(&file).build()
}

/// Builds and runs the app until its window is closed.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Session::default())
        .menu(build_menu)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == EXIT {
                // Same path as the window's ✕, so unsaved changes are asked about.
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.close();
                }
            } else if let Some(&name) = MENU_EVENTS.iter().find(|&&m| m == id) {
                let _ = app.emit("menu", name);
            }
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let dirty = window.state::<Session>().state().dirty;
                if dirty {
                    // The UI asks Save / Don't save / Cancel, then calls `exit_app`.
                    api.prevent_close();
                    let _ = window.emit("close-requested", ());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            universe_state,
            new_universe,
            open_universe,
            save_universe,
            save_universe_as,
            close_universe,
            exit_app
        ])
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

    #[test]
    fn the_session_follows_new_save_as_close_and_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aster.universe");
        let path_text = path.to_str().unwrap();
        let session = Session::default();
        assert!(!session.state().open);

        let state = session.new_universe("Aster").unwrap();
        assert_eq!(
            state,
            UniverseState {
                open: true,
                name: Some("Aster".into()),
                path: None,
                dirty: true
            }
        );

        let state = session.save_as(path_text).unwrap();
        assert!(!state.dirty);
        assert_eq!(state.path.as_deref(), Some(path_text));

        assert!(!session.close().open);

        let state = session.open(path_text).unwrap();
        assert_eq!(state.name.as_deref(), Some("Aster"));
        assert!(!state.dirty);
    }

    #[test]
    fn reopening_the_open_file_is_not_in_use() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aster.universe");
        let path_text = path.to_str().unwrap();
        let session = Session::default();
        session.new_universe("Aster").unwrap();
        session.save_as(path_text).unwrap();
        assert_eq!(
            session.open(path_text).unwrap().name.as_deref(),
            Some("Aster")
        );
    }

    #[test]
    fn session_errors_are_the_owner_facing_messages() {
        let session = Session::default();
        assert_eq!(
            session.new_universe("   ").unwrap_err(),
            "A universe needs a name of 1 to 200 characters."
        );
        session.new_universe("Aster").unwrap();
        assert_eq!(
            session.save().unwrap_err(),
            "This universe hasn't been saved yet. Use Save As."
        );
    }

    #[test]
    fn the_window_title_shows_the_name_and_unsaved_changes() {
        let mut state = Session::default().state();
        assert_eq!(state.window_title(), "Universe");
        state.name = Some("Aster".into());
        state.dirty = false;
        assert_eq!(state.window_title(), "Aster — Universe");
        state.dirty = true;
        assert_eq!(state.window_title(), "Aster • — Universe");
    }
}
