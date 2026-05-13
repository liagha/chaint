# ChainT

Composable parsing combinators with backtracking and memoization.

ChainT is a Rust library for building incremental parsers using composable combinator patterns. It provides an expressive API for defining grammars with alternatives, sequences, repetitions, and optional rules — backed by automatic memoization and rich outcome tracking.

## Installation

```toml
[dependencies]
chaint = "0.1.0"
```

## Quick Start

```rust
use chaint::formation::{Formation, Former};
use chaint::Peeker;

let source = Peeker::new(vec!["hello", "world"]);
let mut former = Former::new(&mut source);

let pattern = Formation::literal("hello");
let result = former.form(pattern);
assert!(result.is_input());
```

```rust
use chaint::formation::{Formation, Former};
use chaint::Peeker;

let source = Peeker::new(vec!['h', 'e', 'l', 'l', 'o']);
let mut former = Former::new(&mut source);

let pattern = Formation::sequence([
    Formation::literal('h'),
    Formation::literal('e'),
    Formation::predicate(|c: &char| c.is_ascii_lowercase()),
    Formation::predicate(|c: &char| c.is_ascii_lowercase()),
    Formation::predicate(|c: &char| c.is_ascii_lowercase()),
]);

let result = former.form(pattern);
assert!(result.is_multiple());
```

```rust
use chaint::formation::{Formation, Former};
use chaint::Peeker;

let source = Peeker::new(vec![1, 2, 3, 4, 5]);
let mut former = Former::new(&mut source);

let digit = Formation::predicate(|n: &i32| *n > 0);
let multiple = digit.into_persistence(1, None);

let result = former.form(multiple);
assert_eq!(result.collect_inputs(), vec![1, 2, 3, 4, 5]);
```

## Peeker

Wrap any `Vec<Item>` into a `Peekable` source:

```rust
use chaint::Peeker;

let mut source = Peeker::new(vec!["a", "b", "c"]);
assert_eq!(source.peek(), Some(&"a"));
source.advance();
assert_eq!(source.peek(), Some(&"b"));
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
use chaint::formation::Formation;
use chaint::Peeker;

let mut source = Peeker::new(vec!["hello"]);
let pattern = Formation::literal("hello")
    .with_fail(|joint| "expected greeting")
    .with_recover(
        |input| input == &"\n",
        |joint| "recovered at newline",
    );
```

## Memoization

Wrap any combinator with `Memoize` to cache results at each position:

```rust
use chaint::Memoize;
use chaint::formation::Formation;

let pattern = Memoize::new(Formation::literal("expensive"));
```