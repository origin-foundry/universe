# Quickstart: Date Formats

How to check that the feature works. Shapes are in [data-model.md](data-model.md) and [contracts/formats.md](contracts/formats.md).

## Automated checks

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test -p uni-calendar -p uni-format -p uni-store -p uni-app
npm test
python scripts/check_architecture.py
python scripts/check_earth_terms.py
python scripts/check_privacy.py
python scripts/check_okf.py knowledge
```

Expected: all pass. `uni-format`'s property tests write and read at least 1,000 dates per test format (SC-002); each pattern refusal, read error, disagreement and ambiguity has a test (SC-003); `uni-store` migrates a version-3 file (SC-004).

## By hand in the app (`npm run dev`)

Start from a universe holding Aster and Tallow, Tallow anchored to Aster at cycle 1, round 1, watch 1 = orbit 412, span 3, sol 9 (spec 008's quickstart, steps 1–3), saved.

1. **Nothing changed yet (SC-004)**: open the converter, choose Tallow, enter cycle 1, round 1, watch 1. Aster shows "age 5, cohort 42, orbit 412, span 3 (Third), sol 9, bell 0, pulse 0", as before this feature.
2. **A format (Story 1, scenario 1)**: open Aster. In **Formats**, **Add format**: name "Short", pattern `{orbit}.{span:02}.{sol:02}`. The sample shows a date like `1.01.01` (or the anchor's date, if Aster has one). Add "Long": `{sol} {span:name}, orbit {orbit} {era}`. Its sample shows the span's member name and the era's name.
3. **Refusals (Story 1, scenario 2)**: change "Short" to `{orbit}.{tide}`: `"tide" isn't a unit of this calendar` appears at the pattern. Try `{orbit`, `{sol:name}`, `just text`: each shows its reason. Press **Keep**: the calendar isn't kept, and the fault is shown at the format. Put `{orbit}.{span:02}.{sol:02}` back.
4. **Write-only (FR-004)**: add "Day" with `{sol} {span:name}`: the sample shows, with "It doesn't show the orbit, so text in it doesn't say which orbit."
5. **Main format (Story 3, scenario 1)**: choose "Long" as the main format and keep. In the converter, enter Tallow cycle 1, round 1, watch 1: Aster shows "9 Third, orbit 412 After Landing".
6. **Partway (Story 3, scenario 2)**: enter Tallow cycle 1, round 1, watch 2. Aster's result ends "partway through the sol"; switch to decimal and exact: the fraction is of a sol, because "Long" stops at the sol.
7. **Read-back (Story 3, scenario 3)**: open Tallow's **Place in time**: the sentence reads "Tallow's cycle 1, round 1, watch 1 began at the start of Aster's 9 Third, orbit 412 After Landing."
8. **Deleting the main format**: open Aster, delete "Long", keep. The converter shows Aster in the generated format again.
9. **Save and reopen (Story 1, scenario 3)**: save, close, reopen. Aster's formats and main choice are as kept.
10. **Old file (FR-018)**: open a universe saved by spec 008's app. It opens with `<name>.v3.universe` beside it, and its calendars show dates as before.

## Timing (SC-005), by hand

`cargo test -p uni-format --release -- --ignored write_read_timing --nocapture` writes and reads one date in a calendar with a 1,000-member named grouping. Expected: each under 10 ms. Record the result in `tasks.md`.
