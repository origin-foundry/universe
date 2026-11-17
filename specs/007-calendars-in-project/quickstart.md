# Quickstart: Calendars in the Project File

How to check that the feature works. Shapes are in [data-model.md](data-model.md) and [contracts/calendar-commands.md](contracts/calendar-commands.md).

## Automated checks

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test -p uni-time -p uni-store -p uni-app
npm test
python scripts/check_architecture.py
python scripts/check_earth_terms.py
python scripts/check_privacy.py
python scripts/check_okf.py knowledge
```

Expected: all pass. `uni-store`'s property test runs 1,000 random calendars through store, save and reopen (SC-002); `uni-app` has one location test per `DefinitionError` variant (SC-003).

## By hand in the app (`npm run dev`)

1. **New universe**: File → New, name it "Aster test". The main area shows an empty calendar list and **Add calendar**.
2. **Starting form (FR-002a)**: click **Add calendar**. The form shows a calendar named "Calendar", a fundamental unit "unit" whose length field is empty and labelled as SI seconds (physical), and one era "Era" numbering "unit". Nothing else.
3. **Refusal (Story 3)**: click **Keep changes** with the length empty. "The fundamental unit's length is missing" appears under the length field, and the form keeps everything.
4. **Aster (Story 1)**: enter Aster's definition (specs 004 and 006):
   - name "Aster";
   - groupings "age" (one member of 10) and "cohort" (one member of 10);
   - "orbit", with member unit "span" and twelve members, Prime to Twelfth, of 56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55 sols;
   - the fundamental unit "sol" of `88775.244`;
   - subdivisions "bell" (20 parts) and "pulse" (100 parts);
   - the era "After Landing" numbering "orbit";
   - the adjustment rule `orbit % 5 == 0 && (orbit % 100 != 0 || orbit % 500 == 0)`, changing Twelfth by 1.

   Click **Keep changes**. Aster is listed, and the window title shows the unsaved-changes mark.
5. **Rule error (Story 3, scenario 2)**: open Aster, change the condition to `orbit %` and keep. The expression error with its position appears under that rule's condition. Discard.
6. **Save and reopen (Story 1, scenario 2)**: save as `aster-test.universe`, close, reopen. Open Aster: every field is as entered, and the condition is exactly as typed.
7. **Pending edit (FR-012)**: open Aster, rename a member, then File → Close. You're asked to keep or discard the edit first, then about the unsaved universe.
8. **Delete (Story 2)**: open Aster, **Delete calendar**, confirm. The list is empty. Close without saving and reopen: Aster is back.
9. **Old file (SC-005)**: open a universe file saved by spec 003's app. It opens with no calendars, and `<name>.v1.universe` sits beside it.

## Timing (SC-004), by hand

`cargo test -p uni-store --release -- --ignored list_twenty_calendars_timing --nocapture` builds a universe with 20 calendars, one with a 1,000-member grouping, saves it, reopens it and lists the calendars, printing the time. Expected: under 1 s. Record the result in `tasks.md`.
