//! The project file's schema and its migrations (specs/003-project-file/data-model.md; the
//! calendar tables of version 2 are in specs/007-calendars-in-project/research.md, R2).

use rusqlite::{Connection, params};

use crate::{StoreError, UniverseName};

/// `PRAGMA application_id` of every Universe project file: ASCII "UNIV".
pub const APPLICATION_ID: i32 = 0x554E_4956;

/// The schema version this app writes, and the newest it can read.
pub const SCHEMA_VERSION: u32 = 3;

/// Migration steps: `MIGRATIONS[n]` takes a file from version `n` to `n + 1`.
///
/// Version 0 has only `meta`; it exists as a test fixture, to exercise this path.
pub const MIGRATIONS: &[&str] = &[
    // 0 → 1: the universe itself.
    "CREATE TABLE universe (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
     INSERT INTO universe (id, name) VALUES (1, 'Untitled');",
    // 1 → 2: calendars (spec 007). Each kind of unit keeps only its own columns.
    "CREATE TABLE calendar (
         id INTEGER PRIMARY KEY,
         seq INTEGER NOT NULL UNIQUE,
         name TEXT NOT NULL UNIQUE
     );
     CREATE TABLE unit (
         id INTEGER PRIMARY KEY,
         calendar_id INTEGER NOT NULL REFERENCES calendar (id) ON DELETE CASCADE,
         seq INTEGER NOT NULL,
         kind TEXT NOT NULL CHECK (kind IN ('grouping', 'fundamental', 'subdivision')),
         name TEXT NOT NULL,
         first_number INTEGER,
         member_unit_name TEXT,
         member_unit_first_number INTEGER,
         length TEXT,
         parts INTEGER,
         UNIQUE (calendar_id, seq),
         CHECK ((kind = 'fundamental') = (length IS NOT NULL)),
         CHECK ((kind = 'subdivision') = (parts IS NOT NULL)),
         CHECK (kind = 'grouping'
                OR (member_unit_name IS NULL AND member_unit_first_number IS NULL))
     );
     CREATE TABLE unit_member (
         id INTEGER PRIMARY KEY,
         unit_id INTEGER NOT NULL REFERENCES unit (id) ON DELETE CASCADE,
         seq INTEGER NOT NULL,
         name TEXT,
         count INTEGER NOT NULL,
         UNIQUE (unit_id, seq)
     );
     CREATE TABLE rule (
         id INTEGER PRIMARY KEY,
         unit_id INTEGER NOT NULL REFERENCES unit (id) ON DELETE CASCADE,
         seq INTEGER NOT NULL,
         condition_src TEXT NOT NULL,
         member TEXT NOT NULL,
         change INTEGER NOT NULL,
         UNIQUE (unit_id, seq)
     );
     CREATE TABLE era (
         id INTEGER PRIMARY KEY,
         calendar_id INTEGER NOT NULL REFERENCES calendar (id) ON DELETE CASCADE,
         seq INTEGER NOT NULL,
         name TEXT NOT NULL,
         unit_name TEXT NOT NULL,
         UNIQUE (calendar_id, seq)
     );",
    // 2 → 3: anchors (spec 008, research R3). A calendar with no row is the reference
    // calendar. Every calendar of a version-2 file is at the hidden zero, so anchoring each
    // after the first to it, fixed at their first dates and the zero, keeps them all in place.
    "CREATE TABLE anchor (
         calendar_id INTEGER PRIMARY KEY REFERENCES calendar (id) ON DELETE CASCADE,
         kind TEXT NOT NULL CHECK (kind IN ('fixed', 'equation')),
         base_calendar_id INTEGER REFERENCES calendar (id) ON DELETE SET NULL,
         base_name TEXT,
         date TEXT NOT NULL,
         base_date TEXT NOT NULL,
         at_secs INTEGER,
         at_nanos INTEGER,
         CHECK (base_calendar_id IS NULL OR base_calendar_id != calendar_id),
         CHECK ((at_secs IS NULL) = (at_nanos IS NULL)),
         CHECK (at_nanos IS NULL OR at_nanos BETWEEN 0 AND 999999999),
         CHECK (kind = 'fixed' OR base_calendar_id IS NOT NULL)
     );
     INSERT INTO anchor (calendar_id, kind, base_calendar_id, base_name, date, base_date,
                         at_secs, at_nanos)
         SELECT id, 'fixed', (SELECT id FROM calendar ORDER BY seq LIMIT 1), NULL, '[1]', '[1]',
                0, 0
         FROM calendar
         WHERE seq > (SELECT MIN(seq) FROM calendar);",
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

/// Switches on foreign keys, so deleting a calendar deletes its units, members, rules and era.
/// SQLite keeps this setting per connection, so every connection a universe uses calls it.
pub(crate) fn enable_foreign_keys(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", true)
}

/// Fills an empty database with a current-version universe called `name`.
///
/// It goes through the same migrations as an old file, so new and migrated files can't
/// drift apart.
pub fn create_current(
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

    fn current(name: &str) -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        enable_foreign_keys(&conn).unwrap();
        create_current(&mut conn, &UniverseName::new(name).unwrap(), "0.1.0", 0).unwrap();
        conn
    }

    fn tables(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn a_new_file_has_the_identifier_meta_and_the_name() {
        let mut conn = Connection::open_in_memory().unwrap();
        let name = UniverseName::new("Aster").unwrap();
        create_current(&mut conn, &name, "0.1.0", 1_800_000_000).unwrap();

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
        assert_eq!(meta, (3, "0.1.0".to_owned(), 1_800_000_000));
        let stored: String = conn
            .query_row("SELECT name FROM universe WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stored, "Aster");
    }

    #[test]
    fn a_new_file_is_version_3_with_the_calendar_and_anchor_tables() {
        assert_eq!(SCHEMA_VERSION, 3);
        assert_eq!(
            tables(&current("Aster")),
            [
                "anchor",
                "calendar",
                "era",
                "meta",
                "rule",
                "unit",
                "unit_member",
                "universe"
            ]
        );
    }

    #[test]
    fn a_version_1_file_migrates_to_the_current_version_keeping_its_name() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_v0(&conn, "0.0.1", 0).unwrap();
        conn.execute_batch(MIGRATIONS[0]).unwrap();
        conn.execute_batch(
            "UPDATE universe SET name = 'Old' WHERE id = 1; UPDATE meta SET schema_version = 1;",
        )
        .unwrap();
        migrate(&mut conn, 1).unwrap();
        let (version, name): (u32, String) = conn
            .query_row(
                "SELECT schema_version, (SELECT name FROM universe) FROM meta",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((version, name.as_str()), (SCHEMA_VERSION, "Old"));
        assert_eq!(count(&conn, "calendar"), 0);
    }

    #[test]
    fn unit_rows_keep_each_kinds_columns_to_itself() {
        let conn = current("Aster");
        conn.execute(
            "INSERT INTO calendar (id, seq, name) VALUES (1, 1, 'Aster')",
            [],
        )
        .unwrap();
        let insert =
            |kind: &str, member_unit: Option<&str>, length: Option<&str>, parts: Option<i64>| {
                conn.execute(
                "INSERT INTO unit (calendar_id, seq, kind, name, member_unit_name, length, parts)
                     VALUES (1, (SELECT COUNT(*) FROM unit) + 1, ?1, 'u', ?2, ?3, ?4)",
                rusqlite::params![kind, member_unit, length, parts],
            )
            };
        assert!(insert("fundamental", None, Some("1"), None).is_ok());
        assert!(
            insert("fundamental", None, None, None).is_err(),
            "no length"
        );
        assert!(insert("grouping", None, Some("1"), None).is_err(), "length");
        assert!(insert("subdivision", None, None, Some(2)).is_ok());
        assert!(insert("subdivision", None, None, None).is_err(), "no parts");
        assert!(insert("grouping", None, None, Some(2)).is_err(), "parts");
        assert!(insert("grouping", Some("span"), None, None).is_ok());
        assert!(
            insert("subdivision", Some("span"), None, Some(2)).is_err(),
            "member unit"
        );
        assert!(insert("other", None, None, None).is_err(), "unknown kind");
    }

    #[test]
    fn deleting_a_calendar_cascades_with_foreign_keys_on() {
        let conn = current("Aster");
        conn.execute_batch(
            "INSERT INTO calendar (id, seq, name) VALUES (1, 1, 'Aster');
             INSERT INTO unit (id, calendar_id, seq, kind, name)
                 VALUES (1, 1, 1, 'grouping', 'orbit');
             INSERT INTO unit_member (unit_id, seq, count) VALUES (1, 1, 10);
             INSERT INTO rule (unit_id, seq, condition_src, member, change)
                 VALUES (1, 1, 'orbit % 4 == 0', '1', 1);
             INSERT INTO era (calendar_id, seq, name, unit_name) VALUES (1, 1, 'Era', 'orbit');
             DELETE FROM calendar;",
        )
        .unwrap();
        for table in ["unit", "unit_member", "rule", "era"] {
            assert_eq!(count(&conn, table), 0, "{table}");
        }
    }

    /// A version-2 file holding the calendars named, in that order, with `seq` 1, 2, ….
    fn version_2(names: &[&str]) -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        enable_foreign_keys(&conn).unwrap();
        create_v0(&conn, "0.0.1", 0).unwrap();
        let tx = conn.transaction().unwrap();
        for step in &MIGRATIONS[..2] {
            tx.execute_batch(step).unwrap();
        }
        tx.execute("UPDATE meta SET schema_version = 2", [])
            .unwrap();
        for (seq, name) in (1i64..).zip(names) {
            // Ids in a different order from seq, so the migration has to go by seq.
            tx.execute(
                "INSERT INTO calendar (id, seq, name) VALUES (?1, ?2, ?3)",
                params![100 - seq, seq, name],
            )
            .unwrap();
        }
        tx.commit().unwrap();
        conn
    }

    type AnchorRow = (
        i64,
        String,
        Option<i64>,
        Option<String>,
        String,
        String,
        Option<i64>,
        Option<i64>,
    );

    fn anchor_rows(conn: &Connection) -> Vec<AnchorRow> {
        let mut stmt = conn
            .prepare(
                "SELECT calendar_id, kind, base_calendar_id, base_name, date, base_date,
                        at_secs, at_nanos
                 FROM anchor ORDER BY calendar_id DESC",
            )
            .unwrap();
        stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
    }

    #[test]
    fn a_version_2_file_anchors_every_later_calendar_to_the_first() {
        let mut conn = version_2(&["Aster", "Tallow", "Wick"]);
        migrate(&mut conn, 2).unwrap();
        let fixed = |id| {
            (
                id,
                "fixed".to_owned(),
                Some(99),
                None,
                "[1]".to_owned(),
                "[1]".to_owned(),
                Some(0),
                Some(0),
            )
        };
        // Aster (id 99) is first by seq and has no row: the reference calendar.
        assert_eq!(anchor_rows(&conn), [fixed(98), fixed(97)]);
    }

    #[test]
    fn a_version_2_file_with_no_calendars_migrates_with_no_anchors() {
        let mut conn = version_2(&[]);
        migrate(&mut conn, 2).unwrap();
        assert!(anchor_rows(&conn).is_empty());
        let version: u32 = conn
            .query_row("SELECT schema_version FROM meta", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 3);
    }

    #[test]
    fn anchor_rows_are_checked() {
        let conn = current("Aster");
        conn.execute_batch(
            "INSERT INTO calendar (id, seq, name) VALUES (1, 1, 'Aster'), (2, 2, 'Tallow');",
        )
        .unwrap();
        let insert = |kind: &str, base: Option<i64>, secs: Option<i64>, nanos: Option<i64>| {
            conn.execute(
                "INSERT INTO anchor (calendar_id, kind, base_calendar_id, date, base_date,
                                     at_secs, at_nanos)
                 VALUES (2, ?1, ?2, '[1]', '[1]', ?3, ?4)",
                params![kind, base, secs, nanos],
            )
            .map(|_| conn.execute("DELETE FROM anchor", []).unwrap())
        };
        assert!(insert("fixed", Some(1), Some(0), Some(0)).is_ok());
        assert!(insert("equation", Some(1), None, None).is_ok());
        assert!(
            insert("fixed", Some(2), Some(0), Some(0)).is_err(),
            "itself"
        );
        assert!(insert("fixed", Some(1), Some(0), None).is_err(), "no nanos");
        assert!(insert("fixed", Some(1), None, Some(0)).is_err(), "no secs");
        assert!(
            insert("fixed", Some(1), Some(0), Some(1_000_000_000)).is_err(),
            "10⁹ nanos"
        );
        assert!(insert("equation", None, None, None).is_err(), "no base");
        assert!(
            insert("other", Some(1), None, None).is_err(),
            "unknown kind"
        );
    }

    #[test]
    fn deleting_calendars_keeps_anchor_rows_right() {
        let conn = current("Aster");
        conn.execute_batch(
            "INSERT INTO calendar (id, seq, name) VALUES (1, 1, 'Aster'), (2, 2, 'Tallow'),
                                                         (3, 3, 'Wick');
             INSERT INTO anchor (calendar_id, kind, base_calendar_id, date, base_date)
                 VALUES (2, 'fixed', 1, '[1]', '[1]'), (3, 'fixed', 2, '[1]', '[1]');
             DELETE FROM calendar WHERE id = 1;",
        )
        .unwrap();
        let base: Option<i64> = conn
            .query_row(
                "SELECT base_calendar_id FROM anchor WHERE calendar_id = 2",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(base, None, "the base is set to NULL");
        conn.execute("DELETE FROM calendar WHERE id = 3", [])
            .unwrap();
        assert_eq!(
            count(&conn, "anchor"),
            1,
            "the anchored calendar's row goes"
        );
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
