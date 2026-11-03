# Feature Specification: Exact Time Core

**Feature Branch**: `design/002-exact-time-core`

**Created**: 2026-11-02

**Status**: Approved by the owner, 2026-11-02

**Input**: User description: "Exact time core (issue #10, Phase 0 spec 002). The app has exact, ordered time values and exact fractions that later calendar and physics work can build on, with no rounding error. Out of scope: reference frames beyond plain absolute time, clocks, calendars."

**Issue**: #10 · **Phase**: 0. Foundations · **Knowledge**: [/time/timeline-and-frames.md](../../knowledge/time/timeline-and-frames.md), [/architecture/crate-layout.md](../../knowledge/architecture/crate-layout.md), [/glossary.md](../../knowledge/glossary.md)

## Clarifications

### Session 2026-11-02

- Q: How big should exact fractions be allowed to get? → A: Fixed 128-bit numerator and denominator, with overflow reported as an error, as the verified design says.
- Q: Should this spec include converting moments to and from floating-point seconds for physics? → A: No. That boundary is built with the physics work in Phase 2; the time core stays entirely free of floating point.
- Q: Should uni-time join the 'exact' crates, so the architecture check forbids floating point in it permanently? → A: Yes. This design PR adds `uni-time` to the `exact` list in `/architecture/crate-layout.md` (owner re-verifies), and the Phase 2 floating-point boundary will live in `uni-physics`.

## User Scenarios & Testing *(mandatory)*

This feature has no screen. Its users are the **features built on it**: the calendar engine (Phase 1), physics (Phases 2–3) and storage (spec 003). The **owner** sees its effect indirectly: dates that are always exact and comparable, and errors instead of silently wrong answers.

### User Story 1 - Exact points in time (Priority: P1)

Later features need to name any moment on the absolute timeline exactly, put moments in order, and measure the exact time between two moments.

**Why this priority**: every calendar date, every event and every physics result is ultimately one of these moments. Nothing in Phase 1 can start without them.

**Independent Test**: create moments from whole seconds and fractions of a second, including far in the past and future, compare them, subtract them, and add time spans to them. Check that every result is exact.

**Acceptance Scenarios**:

1. **Given** two moments, **When** they're compared, **Then** they're ordered correctly, including moments before the timeline's zero and moments one nanosecond apart.
2. **Given** a moment and a time span, **When** the span is added and then subtracted, **Then** the original moment comes back exactly.
3. **Given** two moments, **When** one is subtracted from the other, **Then** the result is the exact signed span between them, to the nanosecond.
4. **Given** a moment 100,000 years from the zero in either direction, **When** a one-nanosecond span is added, **Then** the result differs from the original by exactly one nanosecond.
5. **Given** an operation whose result would fall outside the supported range, **When** it's attempted, **Then** it reports that the result is out of range, and never wraps round or clips silently.

---

### User Story 2 - Exact fractions (Priority: P1)

Calendar rules and unit lengths need numbers like "365 and 97/400 days" or "a 13th of a cycle" with no rounding error, and arithmetic on them that stays exact.

**Why this priority**: constitution principle III forbids floating point in the calendar engine. Without exact fractions, Phase 1's leap rules, mean periods and subdivisions can't be written.

**Independent Test**: add, subtract, multiply, divide and compare fractions, including ones that would be inexact as decimals, and check every result against known exact answers.

**Acceptance Scenarios**:

1. **Given** fractions such as 1/3 and 1/6, **When** they're added, **Then** the result is exactly 1/2.
2. **Given** any fraction, **When** it's created or produced by arithmetic, **Then** it's always in lowest terms with a positive denominator, so equal values are always identical.
3. **Given** a fraction, **When** it's split into a whole part and a remainder (for example, to turn a count of days into whole days plus a fraction of a day), **Then** both parts are exact, and the rounding direction for negative values is defined and consistent (towards minus infinity).
4. **Given** a division by zero, **When** it's attempted, **Then** it reports an error.
5. **Given** an operation whose exact result can't be represented, **When** it's attempted, **Then** it reports that, and never returns an approximate value.

---

### User Story 3 - Moving between moments and fractions (Priority: P2)

Calendars count time in their own units ("412.37 rotations"), so later features need to turn a span of time into an exact fraction of seconds and back again, and scale spans by exact fractions.

**Why this priority**: it joins Stories 1 and 2. Phase 1 needs it, but each story is useful and testable on its own.

**Independent Test**: convert spans to fractions of seconds and back, and multiply spans by fractions, checking exact results and how sub-nanosecond results are handled.

**Acceptance Scenarios**:

1. **Given** any time span, **When** it's converted to a fraction of seconds and back, **Then** the original span comes back exactly.
2. **Given** a fraction of seconds that is a whole number of nanoseconds, **When** it's converted to a span, **Then** the conversion is exact.
3. **Given** a fraction of seconds that falls between two nanoseconds, **When** it's converted to a span, **Then** the caller must say which way to round (down, up or nearest), and the result is never rounded without being asked.

### Edge Cases

- **Before the zero**: moments and spans before the timeline's zero (negative values) behave exactly like positive ones: ordering, arithmetic and splitting into whole and fractional parts all follow the same rules.
- **Range limits**: results that would exceed the supported range are errors, not wrapped or clipped values.
- **Very large fractions**: arithmetic whose exact result is too large to represent is an error, not an approximation.
- **Zero denominator**: creating a fraction with a zero denominator, or dividing by zero, is an error.
- **Equality**: two moments are equal only if they're the same to the nanosecond; two fractions are equal only if they're the same value (2/4 equals 1/2).

## Requirements *(mandatory)*

### Functional Requirements

**Moments and spans**

- **FR-001**: The system MUST represent a moment on the absolute timeline as a whole number of seconds from the timeline's zero plus a number of nanoseconds, as decided in `/time/timeline-and-frames.md`.
- **FR-002**: Moments MUST support at least ±100,000 years from the zero, at one-nanosecond resolution, with no loss of precision anywhere in that range.
- **FR-003**: The system MUST represent a signed span of time between two moments, with the same resolution.
- **FR-004**: Moments MUST be totally ordered, and spans MUST be ordered by length and sign.
- **FR-005**: The system MUST support: moment + span → moment; moment − span → moment; moment − moment → span; span + span, span − span, negating a span, and multiplying or dividing a span by a whole number.
- **FR-006**: Every operation whose result could leave the supported range MUST report an error instead of wrapping, clipping or crashing.

**Fractions**

- **FR-007**: The system MUST represent exact fractions (whole numbers are fractions with denominator 1), always stored in lowest terms with a positive denominator. Numerator and denominator are fixed 128-bit whole numbers; any result outside that size is an error (FR-009).
- **FR-008**: Fractions MUST support addition, subtraction, multiplication, division, negation, comparison, and splitting into a whole part and a remainder (floor and floored remainder) and rounding to a whole number in a caller-chosen direction.
- **FR-009**: Division by zero and results too large to represent exactly MUST be reported as errors, never approximated.

**Moving between them**

- **FR-010**: The system MUST convert a span to an exact fraction of seconds, and a fraction of seconds to a span with a caller-chosen rounding direction for anything finer than a nanosecond.
- **FR-011**: The system MUST multiply a span by a fraction, with the same caller-chosen rounding.

**Quality**

- **FR-012**: The time core MUST use whole numbers and exact fractions only; no floating point. It MUST be listed as an exact crate in `/architecture/crate-layout.md`, so the architecture check enforces this. (The conversion boundary for physics is out of scope and will live in `uni-physics`; see Clarifications.)
- **FR-013**: Every operation MUST be covered by tests that compare against independently known exact answers, plus property tests over random values (for example: add then subtract returns the original; fraction arithmetic matches cross-multiplication).
- **FR-014**: Errors MUST say what went wrong (out of range, division by zero, too large to represent) so later features can show a clear message.

### Key Entities

- **Moment** (`Instant` in the design): a point on the absolute timeline; whole seconds from the zero plus nanoseconds (0 to 999,999,999).
- **Span** (a duration): signed time between two moments, same resolution.
- **Fraction** (a rational number): an exact numerator over a positive denominator, in lowest terms.
- **Rounding direction**: down (towards minus infinity), up (towards plus infinity) or nearest (ties to even), always chosen by the caller.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of operations give exactly the right answer in the tests against known values, across the whole ±100,000-year range, including negative values.
- **SC-002**: Property tests run at least 10,000 random cases per property with no failures.
- **SC-003**: Every out-of-range, divide-by-zero and too-large case in the tests produces an error that names the problem; none produces a wrong value.
- **SC-004**: The time core contains no floating-point code, as confirmed by the architecture check.
- **SC-005**: Moment and span operations are fast enough not to matter later: each takes under a microsecond on the owner's machine.

## Assumptions

- **Scope**: only the timeline's own values. Reference frames (system time, proper time), clocks and calendars are later phases, as the issue says.
- **The physics boundary is out of scope**: converting moments to and from floating-point seconds for physics belongs with the physics work (Phase 2), in `uni-physics`, where its accuracy needs are known.
- **Size of fractions**: 128-bit, as `/time/timeline-and-frames.md` decided (confirmed in Clarifications). If later calendars hit that limit, a design PR can change it.
- **Rounding of "nearest"**: ties go to even, the usual convention for exact arithmetic, so repeated rounding doesn't drift in one direction.
