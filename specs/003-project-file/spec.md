# Feature Specification: Project File

**Feature Branch**: `design/003-project-file`

**Created**: 2026-11-04

**Status**: Draft

**Input**: User description: "Project file (issue #11, Phase 0 spec 003). From the app, the user can create a new empty universe, save it as a file, close it and open it again. Out of scope: any content in the universe (systems, calendars), JSON export, caches."

**Issue**: #11 · **Phase**: 0. Foundations · **Knowledge**: [/storage/schema.md](../../knowledge/storage/schema.md), [/architecture/crate-layout.md](../../knowledge/architecture/crate-layout.md), [/storage/export.md](../../knowledge/storage/export.md)

## Clarifications

### Session 2026-11-04

- Q: How should saving work? → A: Like a document. Changes stay in the app until the owner saves (Save, Ctrl+S); Save As writes to a new place. A new universe is "Untitled" and not on disk until first saved. Closing with unsaved changes asks: save, don't save, or cancel.
- Q: Where should New, Open, Save and Close live in the app? → A: A standard File menu in the window's menu bar (New Ctrl+N, Open Ctrl+O, Save Ctrl+S, Save As Ctrl+Shift+S, Close Ctrl+W, Exit), plus New and Open buttons on the empty "No universe open" screen.
- Q: Should this spec include a recent-files list? → A: Not yet. It's a separate small feature for later; spec 003 covers only New, Open, Save, Save As and Close.

## User Scenarios & Testing *(mandatory)*

The **owner** uses the app. A universe has no content yet beyond its name; later phases add systems, calendars and events to the same file.

### User Story 1 - Create and save a new universe (Priority: P1)

The owner starts a new universe, gives it a name, and saves it as a file they can find again.

**Why this priority**: every later feature stores its work in this file. It completes Phase 0's usable outcome: "app opens, creates and saves an empty universe".

**Independent Test**: start the app, create a universe named "Aster", save it, and confirm a `.universe` file exists where the owner chose, and that the window shows the universe's name instead of "No universe open".

**Acceptance Scenarios**:

1. **Given** no universe is open, **When** the owner chooses New and enters a name, **Then** the window shows that universe's name.
2. **Given** a new universe, **When** the owner saves it, **Then** they choose a folder and file name, and a file ending `.universe` is written there.
3. **Given** a name that's empty or only spaces, **When** the owner tries to create the universe, **Then** it's refused with a message saying a name is needed.

---

### User Story 2 - Open an existing universe (Priority: P1)

The owner reopens a universe they saved earlier and sees it exactly as it was.

**Why this priority**: saving is pointless without opening; together they're the round trip Phase 0 promises.

**Independent Test**: save a universe, close the app, start it again, open the file, and check the name matches.

**Acceptance Scenarios**:

1. **Given** a saved `.universe` file, **When** the owner opens it, **Then** the window shows its name.
2. **Given** a file that isn't a Universe project (another SQLite database, a text file renamed to `.universe`, or a damaged file), **When** the owner opens it, **Then** the app refuses with a message saying it isn't a Universe project file, and nothing is changed on disk.
3. **Given** a file saved by a newer version of the app, **When** it's opened, **Then** the app refuses with a message naming both versions, and doesn't touch the file.
4. **Given** a file saved by an older version of the app, **When** it's opened, **Then** it's brought up to date automatically, after keeping a copy of the original beside it.

---

### User Story 3 - Close a universe (Priority: P2)

The owner closes the open universe and returns to the "No universe open" state, without losing work by accident.

**Why this priority**: needed for a clean round trip, but simpler than create and open.

**Independent Test**: open a universe, close it, and confirm the window says "No universe open" and the file is unchanged.

**Acceptance Scenarios**:

1. **Given** an open universe with no unsaved changes, **When** the owner closes it, **Then** the window returns to "No universe open".
2. **Given** an open universe with unsaved changes, **When** the owner closes it or the app, **Then** they're asked whether to save, discard or cancel.

### Edge Cases

- **Opening the same file twice**: only one universe is open at a time, so opening another file first closes the current one, following the unsaved-changes rule.
- **Saving where the owner can't write** (read-only folder, missing drive): the save fails with a message naming the folder and the reason; the universe stays open and nothing is lost.
- **File moved or deleted while open**: the next save reports it and offers Save As.
- **File locked by another program or a second copy of the app**: opening reports it's in use, rather than risking two writers.
- **Very long or unusual names**: names up to 200 characters of any language are accepted and kept exactly; the file name the owner picks is separate from the universe's name.
- **Crash or power cut during a save**: the file on disk is either the old version or the new one, never a broken mix.

## Requirements *(mandatory)*

### Functional Requirements

**Creating and naming**

- **FR-001**: The owner MUST be able to create a new universe from the app, giving it a name of 1 to 200 characters (after trimming spaces at the ends).
- **FR-002**: Once a universe is open, the window MUST show its name in place of "No universe open", and the window title MUST include it, marked when there are unsaved changes.
- **FR-002a**: A new universe MUST exist only in the app until first saved, and counts as having unsaved changes.

**The file**

- **FR-003**: A universe MUST be saved as a single file with the extension `.universe`, in the SQLite format and layout recorded in `/storage/schema.md`, marked so the app can recognise its own files (the file's application identifier).
- **FR-004**: The file MUST record the schema version and the app version that last wrote it, and when it was created.
- **FR-005**: This feature MUST create only the tables it uses (the file's metadata and the universe's name). Later features add their tables through versioned migrations.
- **FR-006**: Saving MUST be crash-safe: an interrupted save leaves either the previous file or the complete new one.
- **FR-006a**: The owner MUST be able to Save (Ctrl+S; the first save of a new universe asks where) and Save As (Ctrl+Shift+S; always asks where, then continues in the new file). Changes reach the file only when saved.

**Controls**

- **FR-006b**: The window MUST have a File menu with New (Ctrl+N), Open (Ctrl+O), Save (Ctrl+S), Save As (Ctrl+Shift+S), Close (Ctrl+W) and Exit, and the "No universe open" screen MUST offer New and Open buttons. Save, Save As and Close MUST be unavailable when no universe is open.

**Opening**

- **FR-007**: The owner MUST be able to open a `.universe` file chosen from a file dialog.
- **FR-008**: The app MUST refuse files that aren't Universe project files, and files from a newer schema version, with a message saying which, without changing them.
- **FR-009**: Files from an older schema version MUST be migrated to the current version on open, after the original is copied beside it with the old version in its name.

**Closing and unsaved changes**

- **FR-010**: Only one universe MUST be open at a time.
- **FR-011**: The owner MUST be able to close the open universe. If there are unsaved changes, closing the universe or the app MUST ask: save, discard or cancel.

**Errors**

- **FR-012**: Every failure (can't read, can't write, wrong kind of file, newer version, file in use) MUST be shown to the owner with what happened and what they can do, and MUST NOT lose the open universe.

**Layering**

- **FR-013**: File handling MUST live in the storage component (`uni-store`), which MUST NOT depend on the app or the UI, so it can be tested on its own.

### Key Entities

- **Project file**: one `.universe` file holding one universe; recognisable as Universe's by its application identifier.
- **Universe**: the top-level thing the owner works on. For now just a name; later it gains systems, calendars, formats and events.
- **File metadata**: schema version, app version that last wrote it, creation time.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Create, save, close and reopen a universe in under 1 minute of the owner's time, with the name preserved exactly (including non-English characters).
- **SC-002**: Opening or saving an empty universe takes under half a second.
- **SC-003**: 100% of the bad-file cases in the tests (not SQLite, someone else's SQLite file, truncated file, newer version) are refused with the right message and leave the file byte-for-byte unchanged.
- **SC-004**: In a test that interrupts a save part-way, the file afterwards is always either the complete old or the complete new version.
- **SC-005**: An older-version file opens successfully after migration, and its untouched copy remains beside it.

## Assumptions

- **Single user, single machine**: the file isn't shared live between people or machines; "file in use" only guards against a second app window or another program.
- **No content yet**: systems, calendars and events, and their tables, come in later phases; JSON export is Phase 6; cache tables come with the physics.
- **Native dialogs**: the operating system's own Open and Save dialogs are used, filtered to `.universe` files.
- **Migrations framework now, first real migration later**: this feature ships schema version 1 and the mechanism for later versions. The older-version behaviour (FR-009) is tested with a synthetic version-0 file.
- **No recent-files list or autosave**: both ruled out for this spec (see Clarifications).
- **Default file name**: the Save dialog suggests the universe's name as the file name; the owner can change it, and the two can differ afterwards.
