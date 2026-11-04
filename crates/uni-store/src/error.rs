//! Every way working with a project file can fail, with the message shown to the owner
//! (specs/003-project-file/data-model.md, StoreError).

use std::fmt;
use std::path::PathBuf;

/// Something went wrong creating, opening or saving a universe.
///
/// `Display` gives the message shown to the owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// The name is empty, only spaces, or longer than 200 characters.
    InvalidName,
    /// The file isn't SQLite, belongs to another program, or is damaged.
    NotAUniverseFile { reason: String },
    /// The file was saved by a newer version of the app.
    NewerVersion { file: u32, app: u32 },
    /// Another window or program has the file open.
    InUse,
    /// Save was used on a universe that has never been saved.
    NoPath,
    /// The universe's file was moved or deleted while it was open.
    Missing { path: PathBuf },
    /// Reading, writing or replacing a file failed.
    Io {
        action: &'static str,
        path: PathBuf,
        reason: String,
    },
    /// An unexpected SQLite error.
    Database { reason: String },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::InvalidName => {
                write!(f, "A universe needs a name of 1 to 200 characters.")
            }
            StoreError::NotAUniverseFile { reason } => {
                write!(f, "This isn't a Universe project file ({reason}).")
            }
            StoreError::NewerVersion { file, app } => write!(
                f,
                "This file was saved by a newer version of Universe \
                 (file version {file}, this app reads up to {app})."
            ),
            StoreError::InUse => write!(f, "This file is open in another window or program."),
            StoreError::NoPath => {
                write!(f, "This universe hasn't been saved yet. Use Save As.")
            }
            StoreError::Missing { path } => write!(
                f,
                "{} has been moved or deleted. Use Save As to save it somewhere.",
                path.display()
            ),
            StoreError::Io {
                action,
                path,
                reason,
            } => write!(f, "Couldn't {action} {}: {reason}.", path.display()),
            StoreError::Database { reason } => {
                write!(f, "Something went wrong with the project file: {reason}.")
            }
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        use rusqlite::ErrorCode::{DatabaseCorrupt, NotADatabase};
        match error.sqlite_error_code() {
            Some(NotADatabase) => StoreError::NotAUniverseFile {
                reason: "it isn't a database file".to_owned(),
            },
            Some(DatabaseCorrupt) => StoreError::NotAUniverseFile {
                reason: "it's damaged".to_owned(),
            },
            _ => StoreError::Database {
                reason: error.to_string(),
            },
        }
    }
}

impl StoreError {
    /// An [`StoreError::Io`] from a standard I/O error.
    pub(crate) fn io(
        action: &'static str,
        path: impl Into<PathBuf>,
        error: &std::io::Error,
    ) -> Self {
        StoreError::Io {
            action,
            path: path.into(),
            reason: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::ffi;

    fn sqlite_error(code: i32) -> rusqlite::Error {
        rusqlite::Error::SqliteFailure(ffi::Error::new(code), None)
    }

    #[test]
    fn every_message_matches_the_data_model() {
        let cases = [
            (
                StoreError::InvalidName,
                "A universe needs a name of 1 to 200 characters.",
            ),
            (
                StoreError::NotAUniverseFile {
                    reason: "it's damaged".into(),
                },
                "This isn't a Universe project file (it's damaged).",
            ),
            (
                StoreError::NewerVersion { file: 2, app: 1 },
                "This file was saved by a newer version of Universe \
                 (file version 2, this app reads up to 1).",
            ),
            (
                StoreError::InUse,
                "This file is open in another window or program.",
            ),
            (
                StoreError::NoPath,
                "This universe hasn't been saved yet. Use Save As.",
            ),
            (
                StoreError::Missing {
                    path: PathBuf::from("aster.universe"),
                },
                "aster.universe has been moved or deleted. Use Save As to save it somewhere.",
            ),
            (
                StoreError::Io {
                    action: "write",
                    path: PathBuf::from("aster.universe"),
                    reason: "disk full".into(),
                },
                "Couldn't write aster.universe: disk full.",
            ),
            (
                StoreError::Database {
                    reason: "oops".into(),
                },
                "Something went wrong with the project file: oops.",
            ),
        ];
        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn sqlite_errors_for_foreign_or_damaged_files_are_not_a_universe_file() {
        assert_eq!(
            StoreError::from(sqlite_error(ffi::SQLITE_NOTADB)),
            StoreError::NotAUniverseFile {
                reason: "it isn't a database file".into()
            }
        );
        assert_eq!(
            StoreError::from(sqlite_error(ffi::SQLITE_CORRUPT)),
            StoreError::NotAUniverseFile {
                reason: "it's damaged".into()
            }
        );
        assert!(matches!(
            StoreError::from(sqlite_error(ffi::SQLITE_FULL)),
            StoreError::Database { .. }
        ));
    }
}
