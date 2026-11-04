# Quickstart: validating the Project File

1. `cargo test -p uni-store` runs the round trip, bad files, migration, interrupted save and in-use tests (SC-003, SC-004, SC-005).
2. `cargo test -p uni-store --release -- --ignored` checks that open plus save take under 0.5 s (SC-002).
3. `npm test` runs the UI flows: empty screen, the name form, the title marker, and the unsaved-changes prompt.
4. **Owner trial** (`npm run dev`), timed from step a to step f (SC-001: under 1 minute):
   a. The empty screen shows **New universe** and **Open universe…**. The File menu shows Save, Save As and Close greyed out.
   b. Ctrl+N, type `Aster ✶ Ñandú`, then Create. The title becomes `Aster ✶ Ñandú • — Universe`.
   c. Ctrl+S opens a Save dialog suggesting `Aster ✶ Ñandú.universe`; save it to Documents. The `•` disappears.
   d. Ctrl+W returns to "No universe open", with no prompt.
   e. Ctrl+O and choose the file. The name appears exactly as typed.
   f. Ctrl+N, then close the window with the ✕. You're asked Save / Don't save / Cancel. Choose Cancel: nothing happens. Then Don't save: the app exits.
5. **Bad files**: rename a `.txt` file to `.universe` and open it. You get "This isn't a Universe project file", and the file is unchanged.
