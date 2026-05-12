# ChainT

**Composable parsing combinators with backtracking and memoization.**

ChainT is a Rust library for building incremental parsers using composable combinator patterns. It provides an expressive API for defining grammars with alternatives, sequences, repetitions, and optional rules — backed by automatic memoization and rich outcome tracking.

## Installation

```toml
[dependencies]
chaint = "0.1.0"
```

## Quick Start

```rust
use chaint::formation::{Formation, Former};

// Match the literal "hello"
let pattern = Formation::literal("hello");

// Match a sequence of two literals
let greeting = Formation::sequence([
    Formation::literal("hello"),
    Formation::literal("world"),
]);

// Match one or more digits with a predicate
let digits = Formation::predicate(|c: &char| c.is_ascii_digit())
    .into_persistence(1, None);

let mut source = /* your Peekable source */;
let mut former = Former::new(&mut source);
let result = former.form(greeting);
```

## Combinators

| Combinator | Method | Description |
|-----------|--------|-------------|
| `sequence` | `Formation::sequence([...])` | Match all patterns in order |
| `alternative` | `Formation::alternative([...])` | Match the best of several patterns |
| `optional` | `pattern.into_optional()` | Match zero or one occurrence |
| `repetition` | `Formation::repetition(p, min, max)` | Match repeated with halt/keep controls |
| `persistence` | `Formation::persistence(p, min, max)` | Greedy repetition with backtracking |
| `anything` | `Formation::anything()` | Match any single item |
| `nothing` | `Formation::nothing()` | Match nothing (always fails) |

## Outcome States

Each parse attempt yields an outcome that propagates through combinators:

- **Aligned** — Successful match, position advanced
- **Failed** — Match failed but recoverable
- **Panicked** — Critical failure, stops alternatives
- **Blank** — Empty/uninitialized state
- **Ignored** — Successfully matched but discarded

## Post-Processing

Attach actions that run after a successful match:

```rust
let pattern = Formation::literal("hello")
    .with_transform(|joint| {
        // Transform matched output
        Ok(())
    })
    .with_fail(|joint| "expected greeting")
    .with_recover(
        |input| input == &'\n',
        |joint| "recovered at newline",
    );
```

## Memoization

Wrap any combinator with `Memoize` to cache results at each position:

```rust
use chaint::Memoize;

let pattern = Memoize::new(my_expensive_combinator);
```

## Peekable Trait

Implement `Peekable` for your input source to enable incremental parsing with lookahead:

```rust
trait Peekable<'a, Item> {
    fn peek_ahead(&self, n: Offset) -> Option<&Item>;
    fn advance(&mut self) -> Option<Item>;
    fn length(&self) -> Scale;
    // ...
}
```