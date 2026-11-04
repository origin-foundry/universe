//! The project file's schema and its migrations (specs/003-project-file/data-model.md).

use rusqlite::{Connection, params};

use crate::{StoreError, UniverseName};

/// `PRAGMA application_id` of every Universe project file: ASCII "UNIV".
pub const APPLICATION_ID: i32 = 0x554E_4956;

/// The schema version this app writes, and the newest it can read.
pub const SCHEMA_VERSION: u32 = 1;

/// Migration steps: `MIGRATIONS[n]` takes a file from version `n` to `n + 1`.
///
/// Version 0 has only `meta`; it exists as a test fixture, to exercise this path.
pub const MIGRATIONS: &[&str] = &[
    // 0 → 1: the universe itself.
    "CREATE TABLE universe (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
     INSERT INTO universe (id, name) VALUES (1, 'Untitled');",
];

const _: () = assert!(MIGRATIONS.len() == SCHEMA_VERSION as usize);

/// Creates a version-0 file's contents: the identifier and `meta`, nothing else.
pub(crate) fn create_v0(
    conn: &Connection,
    app_version: &str,
    now_unix: i64,
) -> rusqlite::Result<()> {
    conn.pragma_update(None, "application_id", APPLICATION_ID)?;
    conn.execute_batch(
        "CREATE TABLE meta (
             schema_version INTEGER NOT NULL,
             app_version TEXT NOT NULL,
             created_at INTEGER NOT NULL
         );",
    )?;
    conn.execute(
        "INSERT INTO meta (schema_version, app_version, created_at) VALUES (0, ?1, ?2)",
        params![app_version, now_unix],
    )?;
    Ok(())
}

/// Fills an empty database with a current-version universe called `name`.
///
/// It goes through the same migrations as an old file, so new and migrated files can't
/// drift apart.
pub fn create_v1(
    conn: &mut Connection,
    name: &UniverseName,
    app_version: &str,
    now_unix: i64,
) -> Result<(), StoreError> {
    create_v0(conn, app_version, now_unix)?;
    migrate(conn, 0)?;
    conn.execute(
        "UPDATE universe SET name = ?1 WHERE id = 1",
        [name.as_str()],
    )?;
    Ok(())
}

/// Runs every migration from version `from` to [`SCHEMA_VERSION`], in one transaction.
pub fn migrate(conn: &mut Connection, from: u32) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    for step in &MIGRATIONS[from as usize..] {
        tx.execute_batch(step)?;
    }
    tx.execute("UPDATE meta SET schema_version = ?1", [SCHEMA_VERSION])?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_file_has_the_identifier_meta_and_the_name() {
        let mut conn = Connection::open_in_memory().unwrap();
        let name = UniverseName::new("Aster").unwrap();
        create_v1(&mut conn, &name, "0.1.0", 1_800_000_000).unwrap();

        let id: i32 = conn
            .pragma_query_value(None, "application_id", |r| r.get(0))
            .unwrap();
        assert_eq!(id, APPLICATION_ID);
        let meta: (u32, String, i64) = conn
            .query_row(
                "SELECT schema_version, app_version, created_at FROM meta",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(meta, (1, "0.1.0".to_owned(), 1_800_000_000));
        let stored: String = conn
            .query_row("SELECT name FROM universe WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stored, "Aster");
    }

    #[test]
    fn migrating_version_0_gives_an_untitled_universe() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_v0(&conn, "0.0.1", 0).unwrap();
        migrate(&mut conn, 0).unwrap();
        let stored: String = conn
            .query_row("SELECT name FROM universe WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stored, "Untitled");
    }
}
