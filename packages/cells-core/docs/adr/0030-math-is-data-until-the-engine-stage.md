# Math is data until the engine stage, and the engine may arrive late

Status: proposed 2026-11-01. Amends ADR 0005 (the numeric or symbolic
decision moves to the compile), ADR 0011 (the engine splits from the
parser), ADR 0015 (a build makes no handle) and ADR 0028 (constant
expressions are made in the engine stage, not by the first build).

The core holds math in two forms: as **data**, the parsed expressions
the compile makes, and as **handles**, positions in the session's math
engine that cells hold. This ADR puts a hard line between the two. The
core parses math; the engine computes with it. The compile and the probe
handle only data and never call the engine. A load hands the data to
the engine once, in an **engine stage** after the probe, and from then
on only the engine stage, ticks and requests make handles: a build
makes none. The compile declares, from the source alone, whether a
document needs the engine at all, early enough that the engine can be a
separate download fetched while the compile runs.

## Context

### The download

The compile runs on page load, in the browser, as the first thing a
load does. Everything the compile calls is in the first download. ADR
0011 measured the wasm at 1.06 MB before math-expressions-rs and 2.61 MB
after; today every document pays for the whole engine before its first
paint, including the many that have no symbolic math (points, sliders,
arithmetic), whose maths lower to operators (ADR 0005) and never reach
it. Rust wasm cannot link a module at run time, so a part of the engine
that loads later is a separate wasm module the host fetches, and the
core must know, before it needs it, that it will.

### Where the engine is called today

| Stage | What calls the engine | Where |
|---|---|---|
| Compile | a `mathInput`'s prefill, parsed with a throwaway engine | `components/inputs/math_input.rs` |
| Build | a symbolic math's text, parsed (cached in `Compiled::sym_trees` only with repeats) | `build/expand/math.rs`, `sym_source` |
| Build | a constant expression imported, simplified or expanded (cached in `Session::math_constants`) | `build/expand/math.rs` |
| Build | a `GivenMath` default imported, every build | `build/expand/mod.rs`, `Source::GivenMath` |
| Build | every `Instantiate` template imported, once per instance | `build/emit.rs`, the `PlannedOp::Template` arm |
| Build | a constant shape exported, its derivative taken | `build/emit.rs`, `constant_shape` |
| Build | a numeric seed of a math cell imported | `build/emit.rs` |
| Build | the input signature prints expressions | `build/signature.rs` |
| Tick | instantiate, simplify, expand, derivative, evaluate, equals, sample | `program/mod.rs`, `run_sym` |
| Tick | the `Evaluate` inverse makes a number's expression | `tick/invert.rs` |
| Request | a student's text parsed; saved math restored | `Document::parse_math`, `document/state.rs` |

So the engine is reached from every stage, and the build both parses
and computes. Two parsers read math text: the core's own numeric parser
(`compile/expr.rs`, the parse arena of ADR 0005) and math-expressions'
(ADR 0011 names this as the remaining parser to unify).

### A template's identity follows the build's layout

`emit` rebinds a template's leaves from slots to cells and imports the
result, so a leaf is the symbol `cellN<cell index>` and the stored
expression names the cells this build happened to place. Found by
reading the code, not yet reproduced by a test:

- a repeat of 1,000 `<math>$a x</math>` imports 1,000 distinct
  templates per build, since each instance reads other cells, and
  hash-consing (ADR 0011) cannot merge them;
- a rebuild that moves cell indices (a repeat grown before a math with
  leaves) imports every such template again, and the old ones, with
  their `Engine::leaves` entries, are never reclaimed.

`tests/symbolic.rs`'s `rebuilding_back_and_forth_does_not_grow_the_engine`
toggles between two layouts of a math without leaves, so it would not
see either.

## Decision

### 1. The core parses; the engine computes

The core links math-expressions' parser and its `Expr` data type, and
nothing else of it. Everything that stores or computes with an
expression (the store and its hash-consing, printing, substitute,
evaluate, simplify, expand, derivative, equals) is the **engine**,
which may be a separate wasm module.

There is one parser. The compile parses each `<math>`'s text once (the
parse stage, ADR 0029) into an `Expr` whose `$ref`s are positional
placeholders; the settled type (point, tuple), the numeric lowering and
the symbolic expression are all read from that one parse, and
`compile/expr.rs`'s parser goes. A `mathInput`'s prefill is parsed the
same way. `Expr` replaces `Tree` in the compile, since `Tree` cannot
say a relation, a tuple or a function of several arguments, which is
why `Engine::parse_constant` exists.

This is conditional on step 1 of the migration: if the parser and
`Expr` are most of the engine's size, the split buys little and this
section is revisited.

### 2. Math is data until the engine stage

A load runs:

1. **Compile** (outline, parse, plan, link, seal; ADR 0027, ADR 0029),
   making `Expr` data. No handle exists.
2. **Probe** (ADR 0012), a build through the probe view. Its
   instructions never run, and a build makes no handle (rule 3), so it
   needs no engine.
3. **Wait for the engine**, when the document needs one (rule 5) and it
   has not arrived. This is the only point at which a load waits.
4. **Engine stage.** The compile's expressions enter the engine: each
   constant expression with its `simplify` or `expand` applied, each
   template (rule 4), each math's written form (rule 7), each given
   math default, the derivative of each constant shape. The handles go
   into `Session::math_constants` and the constant pool (ADR 0028).
5. **Builds** until the structure settles, then **ticks**.

The compile and the probe are most of a load's time (ADR 0027 names
the probe the expensive stage), so a fetch started early (rule 5) has
them to arrive in. Since no handle exists before step 4, the engine
starts empty and nothing migrates into it.

The engine stage runs once per load. A rebuild does not repeat it: it
reads what the engine stage made.

### 3. Who makes handles

| Stage | Makes handles | Calls the engine for |
|---|---|---|
| Compile, probe | never | nothing |
| Engine stage | every handle that depends on the source alone | import, simplify, expand, derivative, print |
| Build | never | nothing: it copies handles the engine stage made into cells |
| Tick | what the program computes | instantiate, evaluate, simplify, expand, derivative, equals, sample |
| Request, restore | a student's typed math, a saved value | parse |

So a rebuild cannot grow the engine by construction, not by
hash-consing. The input signature (ADR 0017) prints from the compile's
`Expr` with the core's printer of data, not from handles. A curve's
tape (`sym/tape.rs`) compiles from the compile's `Expr`, not from an
expression exported from the engine.

### 4. A template names its leaves by position

A template's leaves are `arg0`, `arg1`, … in the order of the
instruction's inputs, which already carry the cells. `instantiate`
substitutes the input values by position. A template then depends on
the source text, its post-processing and its leaf kinds alone: a
document constant made once, in the engine stage, however many
instances read it and however many builds run. No stored expression
names a cell, a slot, a scope or an instance.

### 5. The compile declares the math a document needs

Whether a document needs the engine depends on the source alone, as a
diagnostic does (ADR 0014). It needs it when it has math that does not
lower to operators: a math with a free symbol or a reference to an
expression, a `mathInput`, a `<function>` or `<derivative>`, `simplify`
or `expand`, a math answer or comparison.

The compile reports this twice:

- **after outlining**, a superset, from tags, attributes and each
  math's tokens (a free symbol is a token). The host starts fetching
  the engine on it;
- **after linking**, the exact answer: a math whose references all name
  numbers lowers (ADR 0005) and needs nothing. The load waits (step 3)
  only on this one.

A fetch the superset started and the exact answer did not need costs
bandwidth; a need the superset missed would cost a wait mid-session,
which rule 6 forbids, so the superset is never smaller than the exact
answer (tested). A host may also guess from the DAST before the core
runs, to start the fetch sooner; the guess is only a hint.

### 6. A tick never waits

A load completes only when every module the document declared is
present. Ticks and requests stay synchronous; no cell is ever pending
on a download, and credit is computable at every submit.

### 7. The numeric or symbolic decision is the compile's

Whether a math lowers is decided once, per template slot, when linking
(ADR 0005 says it depends on the leaf kinds alone), not per instance in
each build (`Builder::is_symbolic`, `MathMode`). A reference whose
referent's kind differs between instances (a choice's branches) is a
compile diagnostic. A math that lowers keeps a constant handle of its
written form beside its operators, made in the engine stage, which a
`symbolicEquality` comparison reads (ADR 0005's finding of 2026-10-14).

### 8. Handles stay in the session

A handle is a position in a store filled in the order the session ran,
so it means nothing outside that session. Saved state, the renderer and
the bridge receive text or LaTeX, never a handle, as today. Saved math
is restored by parsing; a document with saved math has an input, so it
already declared the engine.

### 9. The engine lives as long as the session

Essential math values, parked values and the constant pool hold
handles, so the engine outlives every build (ADR 0015). With rules 3
and 4 only ticks and requests grow it, by about one expression per
distinct value a student produces. That growth has no bound in a long
session (a drag through a symbolic chain), so reclamation is designed
here and built when measured: a rebuild rewrites every cell, so it is
the point where a compaction can remap handles cheaply. The roots are
the new build's math cells, the session's value store and parked
values, and `Session::math_constants`.

## Considered options

- **Compile ahead of time, on a server, and ship `Compiled`.** It would
  take the parser and the compile itself out of the browser. Rejected
  as an assumption: the compile runs on page load. Nothing here
  prevents it later; the engine stage would then be the first thing a
  page runs.
- **Wait inside a tick** for a module a symbolic instruction needs, its
  cells pending until it arrives. Rejected: it makes the tick
  asynchronous and leaves credit undefined at a submit.
- **Paint first, take input later**: load with what is present and hold
  requests until the engine arrives. Deferred, not rejected. It shortens
  first paint for symbolic documents but adds a state the bridge and
  renderer must show; rule 3 makes it cheap to add, since the core's
  printer of data can paint constant math without the engine.
- **Split the engine further**, a light module (store, printer,
  substitute, evaluate) and a heavy one (simplify, equals). Deferred to
  the measurement: a document that displays or evaluates symbolic math
  needs the light part anyway, and most answers need `equals`.
- **Keep the core's own small parser** for common math and fall back
  to math-expressions' for the rest. Smaller if the parser is large, but
  two parsers drift, the reason ADR 0011 removed one. Kept as the
  fallback if step 1 finds the parser heavy.
- **Ship a light engine and replace it with the full one**, copying its
  store with handles kept in place. Unneeded: no handle exists before
  the engine stage, so the load chooses its engine once.

## Consequences

- A document whose math all lowers never downloads the engine.
- A rebuild makes no handle; the engine's growth is the student's.
- The compile, the probe and the build depend on no engine, so
  `build/coverage.rs` and `build/probe.rs` lose their own engines.
- One parser reads math text, and the numeric lowering and the symbolic
  expression cannot disagree about it.
- The core's interface to the engine is handles and `f64`s after the
  engine stage, and `Expr` data once at it: a module boundary with one
  serialization per load.
- The load gains one possible wait, after the probe. A tick gains none.
- The engine's interface needs one thing upstream: serializing an
  `Expr` across the module boundary (`MATH_EXPRESSIONS_UPSTREAM_REQUESTS.md`).

## Migration

Each step lands alone, with the golden diff unchanged except where a
step says otherwise.

1. **Measure.** `twiggy top` and `twiggy dominators` on the release
   `cells-wasm`, by math-expressions module: parser and `Expr`,
   printer, substitute and evaluate, simplify, equals; and the share of
   `serde_json/arbitrary_precision`. Count, over the DoenetML test
   documents, how many need the engine at all, and how many need only
   printing and evaluation. Decide rule 1's condition here.
2. **Positional leaves.** A test that grows a repeat before and around
   `<math>$a x</math>` and counts `Engine::len` first, to confirm the
   finding above; then rule 4.
3. **The build makes no handle.** Move every build-time import, the
   derivative of constant shapes and the template imports into an
   engine stage run before the first build; the input signature and
   tapes read the compile's data.
4. **One parser.** The parse stage parses with math-expressions into
   `Expr`; lowering, the settled type and the prefill read it;
   `compile/expr.rs`'s parser and `Compiled::sym_trees` go.
5. **The decision in the compile.** `is_symbolic` and `MathMode` move to
   linking; written forms for `symbolicEquality`.
6. **The declaration.** The needs after outlining and after linking, a
   test that the first is never smaller, and the load's wait.
7. **The module boundary.** The engine as its own wasm module behind
   the handle interface; the host fetches it from the declaration.
8. **Reclamation**, when a long-session measurement (symchain-1000,
   10,000 drag ticks) says it is needed.

## Open questions

- Does `equals` belong with the light operations? Most answers use it,
  so if it is heavy, a split of the engine saves little.
- Should a reference whose kind differs between a choice's branches be
  refused, or should the math be symbolic in every branch?
- Should ADR 0008's deferred option, keeping cell leaves symbolic during
  simplify, return now that templates are document constants?
