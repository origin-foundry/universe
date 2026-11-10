# Data Model: Calendar Engine Basics

All types live in `uni-calendar`. No floating point anywhere. Time amounts are `uni_time::Ratio` seconds or `uni_time::Instant`s. Nothing here is stored; storage comes with a later feature.

## Definition (input)

### CalendarDef

| Field | Type | Rule |
|---|---|---|
| `name` | `String` | Not empty |
| `clock` | `Clock` | Only `Clock::Gct` for now (FR-007) |
| `groupings` | `Vec<GroupingDef>` | Top to bottom; may be empty |
| `fundamental` | `FundamentalDef` | Exactly one |
| `subdivisions` | `Vec<SubdivisionDef>` | Top to bottom; the first splits the fundamental unit; may be empty |
| `era` | `EraDef` | Exactly one |

### FundamentalDef

| Field | Type | Rule |
|---|---|---|
| `name` | `String` | Unique among unit names |
| `length` | `Ratio` (seconds) | `> 0`; any length the definer chooses, never checked against physics (FR-002a); within the limits below |
| `first_number` | `Option<i64>` | Default 1. Refused if the fundamental unit is the era's unit |

### GroupingDef

| Field | Type | Rule |
|---|---|---|
| `name` | `String` | Unique among unit names |
| `member_unit` | `Option<MemberUnitDef>` | Required when there's more than one member |
| `members` | `Vec<MemberDef>` | At least one; each a run of the unit directly below this grouping |
| `first_number` | `Option<i64>` | Default 1. Refused if this grouping is the era's unit or above it |

### MemberUnitDef

| Field | Type | Rule |
|---|---|---|
| `name` | `String` | Unique among unit names (e.g. "month") |
| `first_number` | `Option<i64>` | Default 1. Refused if the member unit is the era's unit or above it |

### MemberDef

| Field | Type | Rule |
|---|---|---|
| `name` | `Option<String>` | Unique within its grouping when given (e.g. "Prime") |
| `count` | `i64` | `> 0`: how many of the unit directly below the grouping |

### SubdivisionDef

| Field | Type | Rule |
|---|---|---|
| `name` | `String` | Unique among unit names (e.g. "hour") |
| `parts` | `i64` | `≥ 1`: equal children of the unit above |
| `first_number` | `Option<i64>` | Default 0 |

### EraDef

| Field | Type | Rule |
|---|---|---|
| `name` | `String` | Not empty (e.g. "After Landing") |
| `unit` | `String` | Names a grouping, member unit or the fundamental unit; never a subdivision |
| `epoch` | `Instant` | Where fixed number 0 starts on the hidden timeline (FR-008) |

## The unit chain

Checking a definition flattens it into one chain of **units**, top to bottom. For Aster:

| # | Unit | Kind | Length | Numbered |
|---|---|---|---|---|
| 0 | age | grouping (1 member: 10 decades) | 66,800 sols | by count from the epoch |
| 1 | decade | grouping (1 member: 10 years) | 6,680 sols | by count from the epoch |
| 2 | year | grouping over sols, member unit month | 668 sols | **era's unit**: by count from the epoch |
| 3 | month | member unit (12 members) | 56 or 55 sols, by position | position + 1 |
| 4 | sol | fundamental | 88,775.244 s | position in month + 1 |
| 5 | hour | subdivision (20 parts) | 4,438.7622 s | position + 0 |
| 6 | minute | subdivision (100 parts) | 44.387622 s | position + 0 |

## Checked calendar (`Calendar`)

The checked, ready-to-use form. Immutable. Holds the chain above, plus precomputed values: each grouping's length in fundamental units (`i64`), each member-unit grouping's prefix sums of member counts, the fundamental length in nanoseconds as `p/q`, and the subdivision part counts. Created only by `Calendar::new(CalendarDef)`.

### Refused definitions (`DefinitionError`, FR-006)

| Variant | When |
|---|---|
| `EmptyName { what }` | The calendar or era has no name |
| `NonPositiveLength { unit }` | Fundamental length ≤ 0 (FR-002) |
| `NoMembers { unit }` | A grouping lists no members (FR-003) |
| `NonPositiveCount { unit, member }` | A member's count ≤ 0; `member` is its name or its number (FR-003) |
| `MembersNeedUnit { unit }` | More than one member and no member unit (research R1) |
| `TooFewParts { unit }` | A subdivision into fewer than 1 part (FR-004) |
| `DuplicateUnit { name }` | Two units share a name (FR-005) |
| `DuplicateMember { unit, name }` | Two members of one grouping share a name (FR-005) |
| `EraUnitUnknown { unit }` | The era names a unit not in the calendar (FR-006) |
| `EraUnitBelowFundamental { unit }` | The era names a subdivision (FR-006) |
| `FirstNumberNotAllowed { unit }` | A first number is given for the era's unit or a unit above it (research R5) |
| `BeyondLimits { unit, limit }` | One of the limits in research R3; `limit` says which |

## Dates (conversion values)

### Date (input to `date_to_instant`)

| Field | Type | Rule |
|---|---|---|
| `values` | `Vec<i64>` | From the era's unit downwards; at least one value, at most one per unit from the era's unit to the smallest unit. Missing smaller units take their first value (FR-011). Units above the era's unit are never entered (FR-010) |

### InstantDate (result of `instant_to_date`)

| Field | Type | Meaning |
|---|---|---|
| `above` | `Vec<i64>` | Numbers of the units above the era's unit, top to bottom (Aster: age, decade) |
| `date` | `Date` | Every value from the era's unit to the smallest unit |
| `remainder` | `Ratio` (seconds) | Exact time from the start of the smallest unit; `0 ≤ remainder <` that unit's length |

### DateStart (result of `date_to_instant`)

| Field | Type | Meaning |
|---|---|---|
| `instant` | `Instant` | The first whole nanosecond at or after the date's exact start |
| `rounded_by` | `Option<Ratio>` (seconds) | `None` if the exact start is a whole nanosecond; otherwise the exact amount added, `0 < rounded_by < 1 ns` (FR-014) |

### Refused conversions (`CalendarError`, FR-013, FR-015, FR-019)

| Variant | When | Example message |
|---|---|---|
| `InvalidValue { unit, value, first, last, within }` | A value is outside its unit's range; `within` names the member when the range depends on it | "sol 56 is outside 1 to 55 in Third" |
| `OutOfRange` | A result needs a number outside `i64`, or an instant outside the `Instant` range | "date is out of the supported range" |
| `EmptyDate` | A date with no values | "a date needs at least the era's unit" |
| `TooManyValues { given, max }` | More values than units from the era's unit down | "a date has at most 5 values here" |
| `UnknownUnit { name }` | A query names a unit not in the calendar | "no unit named week" |
| `NotFixed { unit, of }` | `units_per` asked for a count that varies (months in sols) | "the number of sols in a month varies" |

## Relationships

- A `Calendar` has one fundamental unit, an ordered chain of groupings above it and subdivisions below it, and one era.
- The era's unit splits the chain: units above it are numbered by count from the epoch; units below it by position within their parent plus their first number.
- **Fixed number**: the whole count of fundamental units from the epoch (`i64`, negative before it). Every date and instant maps through it (FR-012).
