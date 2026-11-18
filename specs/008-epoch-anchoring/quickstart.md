# Quickstart: Epoch Anchoring and the Converter

How to check that the feature works. Shapes are in [data-model.md](data-model.md) and [contracts/anchors-and-converter.md](contracts/anchors-and-converter.md).

## Automated checks

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test -p uni-time -p uni-calendar -p uni-store -p uni-app
npm test
python scripts/check_architecture.py
python scripts/check_earth_terms.py
python scripts/check_privacy.py
python scripts/check_okf.py knowledge
```

Expected: all pass. `uni-calendar`'s property test checks `epoch_for` on 1,000 random calendars; `uni-store`'s checks 1,000 random chains of anchors (SC-002); `uni-app` has a location test per anchor refusal (SC-003).

## By hand in the app (`npm run dev`)

Start from a universe holding Aster (spec 007's quickstart, step 4), saved.

1. **Not anchored (Story 1, scenario 1)**: open Aster. **Place in time** says "Not anchored.", with nothing to enter and no **Anchor to a base calendar** button (there's no other calendar yet). No timeline value appears anywhere.
2. **Tallow's starting anchor (FR-007)**: **Add calendar**. **Place in time** shows Aster as the base calendar, the anchor date and the base date with only their first value, `1`, and "Fix the moment" chosen.
3. **Tallow (Story 1, scenario 2)**: define Tallow: name "Tallow"; groupings "cycle" (one member of 52) and "round" (one member of 7); the fundamental unit "watch" of `30000`; the era "Count" numbering "cycle". Set its anchor date to cycle 1, round 1, watch 1 and its base date in Aster to orbit 412, span 3, sol 9, fixing the moment. Keep. Tallow is listed with no problem.
4. **Converter (Story 2, scenario 1)**: **Converter**, choose Tallow, enter cycle 1, round 1, watch 1. Aster shows age 5, cohort 42, orbit 412, span 3, sol 9, bell 0, pulse 0, with no partway mark.
5. **Partway and precision (Story 2, scenarios 2–3)**: choose Aster, enter orbit 412, span 3, sol 9, bell 7. Tallow shows a watch "partway through". Switch to decimal, then exact: the same result shows the fraction of the watch as `≈ 0.xyz` or exactly. Go **Back to calendars** and return: the switch is as you left it.
6. **Refusal (Story 2, scenario 4)**: enter Aster span 13. "span 13 is outside 1 to 12" appears at the span field, and no results are shown.
7. **Fixed moment (FR-005, FR-005a)**: open Aster, lengthen span Prime by one sol (57), keep. Open Tallow: its note says Aster has changed since, and gives the Aster date the moment now falls at. The converter still shows Tallow cycle 1, round 1, watch 1 at that moved Aster date.
8. **Keep the equation (Story 3, scenario 3)**: open Tallow, choose "Keep the equation", set the base date back to orbit 412, span 3, sol 9, keep. Lengthen span Second in Aster. The converter shows Tallow cycle 1, round 1, watch 1 at orbit 412, span 3, sol 9 again.
9. **Loop (Story 3, scenario 1)**: add Wick (any small calendar) with Tallow as its base, keeping the equation. Then try to give Tallow Wick as its base, with either kind: "Wick follows Tallow, so Tallow can't be anchored to it." appears at the base calendar, and nothing is kept.
10. **Delete (Story 3, scenario 4)**: in the converter, note Wick's date for Aster orbit 500. Delete Tallow: the confirmation names Wick. Confirm. Wick's editor says "Its base calendar, Tallow, has since been deleted.", and the converter gives the same Wick date for Aster orbit 500.
11. **Save and reopen (Story 1, scenario 3)**: save, close, reopen. Every anchor shows as entered.
12. **Anchoring the first calendar (Story 1, scenario 6, FR-001a)**: open Aster: **Place in time** now offers **Anchor to a base calendar**. Press it, choose Wick as the base calendar, set the anchor date to orbit 1 and the base date to Wick's first value `1`, fixing the moment, and keep. In the converter, Wick's first date now shows Aster orbit 1 at its very start, and Wick's own editor is unchanged. Aster's editor shows its anchor, with no way to remove it.
13. **Old file (Story 1, scenario 5, SC-004)**: open a universe saved by spec 007's app with two calendars. The second is anchored to the first at their first dates, fixing the moment, and `<name>.v2.universe` sits beside the file.

## Timing (SC-005), by hand

`cargo test -p uni-app --release -- --ignored convert_twenty_calendars_timing --nocapture` builds a universe with 20 calendars, one with a 1,000-member grouping, anchored in a chain of equations, and times one `convert` across them. Expected: under 1 s. Record the result in `tasks.md`.
