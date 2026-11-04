//! The open universe: an in-memory copy of a project file, saved crash-safely and locked while
//! open (specs/003-project-file/research.md, R2 to R5).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, MAIN_DB, OptionalExtension};

use crate::schema::{self, APPLICATION_ID, SCHEMA_VERSION};
use crate::{StoreError, UniverseName};

/// A universe open in the app.
///
/// The app works on an in-memory database; nothing reaches the file until [`save`] or
/// [`save_as`]. Once the universe has a file, that file is locked until the universe is
/// dropped, so a second window or program can't open it at the same time.
///
/// ```
/// use uni_store::{OpenUniverse, UniverseName};
///
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("aster.universe");
///
/// let mut universe = OpenUniverse::new(UniverseName::new("Aster")?, "0.1.0")?;
/// assert!(universe.is_dirty());
/// universe.save_as(&path)?;
/// assert!(!universe.is_dirty());
/// drop(universe); // closes the universe and releases the file
///
/// let reopened = OpenUniverse::open(&path, "0.1.0")?;
/// assert_eq!(reopened.name()?.as_str(), "Aster");
/// # Ok::<(), uni_store::StoreError>(())
/// ```
///
/// [`save`]: OpenUniverse::save
/// [`save_as`]: OpenUniverse::save_as
pub struct OpenUniverse {
    db: Connection,
    app_version: String,
    /// The file and the locked handle on it, once the universe has been saved or opened.
    file: Option<(PathBuf, File)>,
    dirty: bool,
    fail_before_rename: bool,
}

impl OpenUniverse {
    /// A new universe called `name`, not yet saved anywhere.
    pub fn new(name: UniverseName, app_version: &str) -> Result<OpenUniverse, StoreError> {
        let mut db = Connection::open_in_memory()?;
        schema::create_v1(&mut db, &name, app_version, now_unix())?;
        Ok(OpenUniverse {
            db,
            app_version: app_version.to_owned(),
            file: None,
            dirty: true,
            fail_before_rename: false,
        })
    }

    /// Opens the project file at `path` and locks it.
    ///
    /// Files that aren't Universe project files, or come from a newer version, are refused
    /// without being changed. A file from an older version is copied to
    /// `<name>.v<old>.universe` beside it and migrated in memory; the migrated universe has
    /// unsaved changes and reaches the file at the next save.
    pub fn open(path: &Path, app_version: &str) -> Result<OpenUniverse, StoreError> {
        let mut handle = File::open(path).map_err(|e| StoreError::io("open", path, &e))?;
        lock(&handle, path)?;

        // On Windows the lock is mandatory: no other handle, SQLite's included, can read the
        // file while it's held. So the bytes are read through the locked handle and handed
        // to SQLite in memory.
        let mut bytes = Vec::new();
        handle
            .read_to_end(&mut bytes)
            .map_err(|e| StoreError::io("read", path, &e))?;
        if bytes.is_empty() {
            return Err(not_ours("the file is empty"));
        }

        let mut db = Connection::open_in_memory()?;
        db.deserialize_read_exact(MAIN_DB, bytes.as_slice(), bytes.len(), false)?;

        let id: i32 = db.pragma_query_value(None, "application_id", |r| r.get(0))?;
        if id != APPLICATION_ID {
            return Err(not_ours("it belongs to another program"));
        }
        let check: String = db.pragma_query_value(None, "quick_check", |r| r.get(0))?;
        if check != "ok" {
            return Err(not_ours("it's damaged"));
        }
        let version: u32 = db
            .query_row("SELECT schema_version FROM meta", [], |r| r.get(0))
            .optional()
            .map_err(|_| not_ours("it's damaged"))?
            .ok_or_else(|| not_ours("it's damaged"))?;

        let mut dirty = false;
        if version > SCHEMA_VERSION {
            return Err(StoreError::NewerVersion {
                file: version,
                app: SCHEMA_VERSION,
            });
        }
        if version < SCHEMA_VERSION {
            keep_copy(path, version, &bytes)?;
            schema::migrate(&mut db, version)?;
            dirty = true;
        }

        Ok(OpenUniverse {
            db,
            app_version: app_version.to_owned(),
            file: Some((path.to_owned(), handle)),
            dirty,
            fail_before_rename: false,
        })
    }

    /// Saves to the universe's file.
    ///
    /// [`StoreError::NoPath`] if it has never been saved, and [`StoreError::Missing`] if its
    /// file has been moved or deleted since; in both cases nothing is written.
    pub fn save(&mut self) -> Result<(), StoreError> {
        let path = match &self.file {
            None => return Err(StoreError::NoPath),
            Some((path, _)) => path.clone(),
        };
        if !self.still_holds(&path) {
            return Err(StoreError::Missing { path });
        }
        self.write_to(&path)
    }

    /// Saves to `path` and carries on in that file; the lock moves to it.
    pub fn save_as(&mut self, path: &Path) -> Result<(), StoreError> {
        let is_current = self.file.as_ref().is_some_and(|(p, _)| p == path);
        if !is_current && path.exists() {
            // Refuse to replace a file that something else has open.
            if let Ok(other) = File::open(path) {
                lock(&other, path)?;
            }
        }
        self.write_to(path)
    }

    /// The universe's name.
    pub fn name(&self) -> Result<UniverseName, StoreError> {
        let stored: String =
            self.db
                .query_row("SELECT name FROM universe WHERE id = 1", [], |r| r.get(0))?;
        UniverseName::new(&stored)
    }

    /// The file the universe was last opened from or saved to, if any.
    pub fn path(&self) -> Option<&Path> {
        self.file.as_ref().map(|(path, _)| path.as_path())
    }

    /// Whether there are changes that haven't been saved.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// When the universe was first created, in whole seconds since 1970-01-01 UTC.
    pub fn created_at_unix(&self) -> Result<i64, StoreError> {
        Ok(self
            .db
            .query_row("SELECT created_at FROM meta", [], |r| r.get(0))?)
    }

    /// Makes every save fail just before its rename while `on`, to test crash safety.
    #[cfg(feature = "test-hooks")]
    pub fn fail_saves_before_rename(&mut self, on: bool) {
        self.fail_before_rename = on;
    }

    /// Whether the file at `path` is still the one this universe has locked.
    ///
    /// A fresh handle can't take the lock while ours holds it; if it can, the file there is
    /// a different one (or there's none).
    fn still_holds(&self, path: &Path) -> bool {
        match File::open(path) {
            Ok(fresh) => matches!(fresh.try_lock(), Err(TryLockError::WouldBlock)),
            Err(_) => false,
        }
    }

    /// Writes the in-memory database to a temporary file beside `path`, then renames it over
    /// `path`, so the file is always either the complete old version or the complete new one.
    fn write_to(&mut self, path: &Path) -> Result<(), StoreError> {
        let temp = temp_path(path);
        let result = self.write_via(path, &temp);
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        let handle = result?;
        self.file = Some((path.to_owned(), handle)); // drops the old handle and its lock
        self.dirty = false;
        Ok(())
    }

    fn write_via(&mut self, path: &Path, temp: &Path) -> Result<File, StoreError> {
        File::create_new(temp).map_err(|e| StoreError::io("write", path, &e))?;
        self.db
            .execute("UPDATE meta SET app_version = ?1", [&self.app_version])?;
        self.db
            .backup(MAIN_DB, temp, None::<fn(rusqlite::backup::Progress)>)
            .map_err(|e| StoreError::Io {
                action: "write",
                path: path.to_owned(),
                reason: e.to_string(),
            })?;

        let handle = OpenOptions::new()
            .read(true)
            .write(true)
            .open(temp)
            .map_err(|e| StoreError::io("write", path, &e))?;
        handle
            .sync_all()
            .map_err(|e| StoreError::io("write", path, &e))?;
        // The lock is taken on the new file before it replaces the old one, so there's no
        // moment when the file at `path` is unlocked.
        lock(&handle, path)?;

        if self.fail_before_rename {
            return Err(StoreError::Io {
                action: "replace",
                path: path.to_owned(),
                reason: "the test hook stopped the save".to_owned(),
            });
        }
        fs::rename(temp, path).map_err(|e| StoreError::io("replace", path, &e))?;
        Ok(handle)
    }
}

/// Takes the exclusive lock on `handle`, or says the file is in use.
fn lock(handle: &File, path: &Path) -> Result<(), StoreError> {
    match handle.try_lock() {
        Ok(()) => Ok(()),
        Err(TryLockError::WouldBlock) => Err(StoreError::InUse),
        Err(TryLockError::Error(e)) => Err(StoreError::io("lock", path, &e)),
    }
}

fn not_ours(reason: &str) -> StoreError {
    StoreError::NotAUniverseFile {
        reason: reason.to_owned(),
    }
}

/// `<name>.universe.saving-<random>` in the same folder, so the rename stays within one
/// file system.
fn temp_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let mut name = path.file_name().unwrap_or_default().to_owned();
    name.push(format!(".saving-{}-{nanos:09}", std::process::id()));
    path.with_file_name(name)
}

/// Writes `bytes` (the file as it was) to `<stem>.v<version>.universe` beside `path`, adding
/// ` (2)`, ` (3)`… rather than replacing an earlier copy.
fn keep_copy(path: &Path, version: u32, bytes: &[u8]) -> Result<(), StoreError> {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    for n in 1u32.. {
        let suffix = if n == 1 {
            String::new()
        } else {
            format!(" ({n})")
        };
        let copy = path.with_file_name(format!("{stem}.v{version}{suffix}.universe"));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&copy)
        {
            Ok(mut file) => {
                use std::io::Write;
                return file
                    .write_all(bytes)
                    .and_then(|()| file.sync_all())
                    .map_err(|e| StoreError::io("keep a copy of", copy, &e));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(StoreError::io("keep a copy of", copy, &e)),
        }
    }
    unreachable!("there is always a free copy name")
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
