# Research: Expression Language Basics

## R1. Shape of the API

- **Decision**: one checked type, `Expr`, made only by `Expr::check(source, vars, result)`. The caller declares its variables as `VarDecl { name, ty }` in an order of its choosing and evaluates with `expr.eval(&values)`, one `Value` per declared variable in that order. `expr.period(var, limit)` works out the period in one whole-number variable. `Expr` keeps its source text (FR-004).
- **Rationale**:
  - **Checked by construction**: an `Expr` that exists has passed every check in FR-003, so evaluation can only fail for the reasons in FR-006. The calendar (spec 006) checks once and evaluates many times.
  - **Values by position, not by name**: evaluation is the hot path (spec 006 evaluates a rule once per year of its period); variables are resolved to indices when checking, so evaluation does no string lookups. A caller passing the wrong number or types of values gets `EvalError::Inputs`, never a panic (FR-011).
- **Alternatives**: a separate `parse` then `type_check` in the public API; rejected because no caller needs an unchecked expression yet, and every extra public state is one more thing to keep stable.

## R2. Grammar and tokens

- **Decision**: a hand-written lexer and a recursive-descent parser over this grammar (loosest first; the full grammar is in the contract):

  ```text
  expr    = or
  or      = and { "||" and }
  and     = cmp { "&&" cmp }
  cmp     = sum [ cmpop sum ]            a second cmpop is ChainedComparison
  sum     = product { ("+" | "-") product }
  product = unary { ("*" | "/" | "%") unary }
  unary   = ("-" | "!") unary | atom
  atom    = INT | "true" | "false" | NAME | "(" expr ")" | "if" expr "then" expr "else" expr
  ```

  `if` is an atom whose `else` branch extends as far as possible, so `if c then 1 else 2 + 3` takes `2 + 3` as its else branch, which is what "loosest" means in the spec (Edge Cases).
- **Tokens**: decimal integer literals (digits only); names (a letter or `_`, then letters, digits or `_`, using Rust's `char::is_alphabetic` and `is_alphanumeric`, so any script works); the keywords `if then else true false`; the operators; parentheses; whitespace (any `char::is_whitespace`) between tokens. Anything else is `InvalidCharacter` with its position.
- **Positions**: every token carries its line and column, both from 1, columns counted in `char`s (spec, Edge Cases). Every AST node keeps the position of its first token and, for operators, of the operator.
- **Literals**: a literal that doesn't fit in `i64` is `LiteralOutOfRange` (Story 1 scenario 8). The one exception is `-9223372036854775808`: a unary minus directly in front of a literal is folded into it, so the smallest `i64` can be written.
- **Rationale**: the grammar is small enough that a parser generator or a combinator crate would be more code to review than the parser itself, and a hand-written parser gives exact positions and messages. No new runtime dependency (R7).

## R3. Type checking

- **Decision**: a separate pass over the parsed tree resolves names to variable indices and gives each node a type. Rules:
  - `+ - * / %` and unary `-`: whole numbers in, whole number out.
  - `< <= > >=`: whole numbers in, true/false out. `== !=`: both sides the same type, true/false out.
  - `&& || !`: true/false in and out.
  - `if c then a else b`: `c` true/false; `a` and `b` the same type, which is the result.
  - The whole expression must have the type the caller asked for.
- **Errors**: `TypeMismatch { context, expected, found, at }`, where `context` names the operator or keyword (`"+"`, `"!"`, `"if"`, `"then/else"`, or `"result"` for the whole expression) and `at` is the start of the operand with the wrong type. So `year + true` points at `true` (1:8), `!year` at `year` (1:2), and `year % 4` with a true/false result expected at 1:1. Unknown names are `UnknownVariable { name, at }`.
- **Declarations** are checked first: a declared name that isn't a valid name (R2) or is a keyword, or two declarations with the same name, is `InvalidVariable { name, reason }`.
- **Rationale**: keeping parsing and type checking apart keeps each simple to review and test, and matches `/language/expression-language.md` ("compiled to an AST", "type-checked when it's saved").

## R4. Depth limit

- **Decision**: an expression's tree may be at most **256** levels deep, counting every node. A deeper one is `TooDeep { limit: 256, at }`, pointing at the node where the limit was crossed. The parser also counts its own recursion against the same limit (one level for each nested parenthesis, unary operator and part of an `if`), so it can't overflow the stack before it gets there; that is what refuses 257 nested parentheses, which build no nodes of their own.
- **Rationale**: evaluation and period analysis walk the tree recursively. A chain like `1 + 1 + … + 1` is parsed by a loop but builds a tree as deep as it is long, so limiting only parser recursion wouldn't be enough. 256 is far beyond any real rule (the Gregorian rule is 6 deep) and far within the stack. The spec asks for at least 64.

## R5. Exact arithmetic

- **Decision**: `i64` values with checked operations. Floored division and remainder are computed as `q = a / b` (checked, which catches `b = 0` and `i64::MIN / −1`), then `q − 1` when the remainder is non-zero and `a` and `b` have opposite signs; `a % b = a − b × ⌊a / b⌋`, which takes the sign of `b`. Rust's `div_euclid`/`rem_euclid` are *not* used: they differ from floored division when the divisor is negative (`7.rem_euclid(-4)` is 3, floored is −1; Story 2 scenario 2).
- **Errors**: `DivisionByZero { at }` and `OutOfRange { at }`, with the operator's position (FR-006).
- **Short-circuit**: `&&`, `||` and `if` evaluate only what they need (FR-007; Story 2 scenario 6).

## R6. Working out the period (FR-008, FR-009)

- **Decision**: a single bottom-up pass over the checked tree gives every node one of four **forms** with respect to the chosen variable `y`:

  | Form | Meaning |
  |---|---|
  | `Const(c)` | doesn't depend on `y`, and evaluates without error to `c` (folded when the pass meets it) |
  | `Affine(a, b)` | equals `a·y + b` exactly, `a ≠ 0` (whole numbers only) |
  | `Periodic(P)` | its exact value, or division-by-zero error, at `y` and `y + P` is always the same |
  | `NotRepeating(at)` | none of the above can be shown; `at` is the node responsible |

- **Rules** (`m` a non-zero constant; `lcm` checked against the caller's limit):
  - `y` → `Affine(1, 0)`; other variables, `true`, `false` → `Periodic(1)` (they don't change with `y`); literals → `Const`.
  - `Const op Const` → folded to `Const` with the same checked arithmetic as evaluation; if folding fails (division by zero, overflow) the node is `Periodic(1)`, because it fails the same way for every `y`.
  - `Affine ± Const`, `Const ± Affine`, `Affine ± Affine`, `Affine × Const`, `Const × Affine`, `−Affine` → `Affine` with the combined coefficients (a result with `a = 0` becomes `Const`); if a coefficient overflows, `NotRepeating` at that operator.
  - `Affine % m` → `Periodic(|m|)`, because `a·(y + m) + b ≡ a·y + b (mod m)`. `Affine % 0` → `Periodic(1)` (it always fails).
  - Any operator, `if` or `!` whose operands are all `Const` or `Periodic` → `Periodic(lcm of their periods)` (a `Const` counts as 1).
  - Anything else involving an `Affine` operand → `NotRepeating` at the operator: `Affine / x` (`year / 7`), `Affine × Affine` (`year * year`), `Affine % Affine` or `% Periodic`, comparisons with an `Affine` side (`year > 100`), an `Affine` branch of `if`, `Affine ± Periodic`.
  - The result is the root's period (`Const` → 1); `NotRepeating(at)` → `PeriodError::NotRepeating { at }`; any period over the limit → `PeriodError::OverLimit { limit }` as soon as it appears (a parent's period is a multiple of each child's, so it can only grow).
- **Why this is safe**: each rule holds for exact whole numbers, so the root's exact value repeats with the period given. Division-by-zero errors repeat too, because a divisor is always `Const` or `Periodic`. Out-of-range errors may not: `year * 4611686018427387904 % 3` overflows at `y = 2` but has period 3. That is why the guarantee in FR-008 is about the exact value, and why spec 006 evaluates only one period near zero and maps every other year onto it (spec, Edge Cases "Repetition and range"). The spec's Story 3 scenario 7 and FR-008 are tightened to say this (value or division-by-zero error), a wording fix made in this design PR.
- **Not always the smallest**: `lcm` over the parts is a period, not necessarily the least one (spec, Edge Cases).
- **Worked examples** (spec Story 3): Aster's rule: `year % 5`, `% 100`, `% 500` give 5, 100, 500; comparisons with constants keep them; `&&`/`||` take the lcm: 500. Gregorian: lcm(4, 100, 400) = 400. Islamic tabular: `11·y + 14` is `Affine(11, 14)`, `% 30` gives 30, `< 11` keeps 30. `true`, `3 > 2`: 1. `year % 7 == 0 && year % 11 == 0 && year % 13 == 0`: 1,001, over a limit of 1,000.
- **Alternatives**: evaluating the expression for many years and searching for a cycle (can't prove anything, and costs time); a full symbolic analysis that finds the smallest period (much more code for no practical gain: callers can search inside the safe period).

## R7. Crate wiring

- **Decision**: `uni-expr` gets `publish = false` (consistent with the other workspace crates), keeps `#![deny(clippy::float_arithmetic)]`, and has **no runtime dependencies**. Dev-dependencies: `proptest` and `num-bigint` at exactly the versions `uni-calendar` uses. `uni-expr` is already in the `exact` list of `/architecture/crate-layout.md`; its allowed dependency on `uni-time` isn't needed yet. No new crate and no new dependency edge, so no architecture change.

## R8. Tests (constitution VI)

- **Hand-worked tests** for every acceptance scenario in Stories 1 to 3, each asserting the exact value, error variant and position.
- **Property tests** (`proptest`, 10,000 cases each; SC-003):
  1. **Evaluation against a reference**: a strategy generates random expression trees (literals, `year` and a second variable `n`, every operator, `if`), renders each as text twice (fully parenthesised, and with the fewest parentheses precedence allows, which also tests precedence), checks it, and evaluates it for random values. An independent evaluator in the test code walks the generated tree with `num-bigint`'s `BigInt` (unlimited size), applying floored division by its own definition; it reports out of range if any value it computes on the way doesn't fit `i64`, and division by zero where it meets one. Both must agree on the value or the kind of error.
  2. **Periods**: for random generated expressions whose period the language works out, evaluating at random `y` and `y + P` (both small enough that the reference finds no overflow) gives the same value or the same division-by-zero error.
  3. **Any text**: random strings, including long and deeply nested ones (`(((…)))` and `1+1+…`), are checked without panicking; each result is either an `Expr` or an error with a position inside the text.
- **Timing** (SC-004): an `#[ignore]` test evaluates the Gregorian rule 1,000,000 times and asserts under one second (under 1 µs each), run by hand in release mode, as for specs 002 and 004.

## R9. Knowledge changes in this design PR

- `/language/expression-language.md`: a section "First subset (spec 005)" listing what exists now (types, operators, floored `/` and `%`, precedence, no chained comparisons, positions as line and column in characters, the depth limit of 256) and a section "Repetition" describing the period and the forms in R6. The existing description of the full language stays. Back to `status: draft` without `verified` until the owner re-verifies it.
- `knowledge/log.md` entry.
- No change to `/architecture/crate-layout.md` (R7).
