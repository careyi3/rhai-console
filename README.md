# rhai-console

Operational REPL and script runner for embedding Rhai in Rust applications.

Heavily inspired by the Rails Console and Django Shell; all you need to configure is your application state and a few module registrations, and you get back a CLI with an interactive REPL and script runner all of which run within your applications context.

You can find an example here [rhai-ops](https://github.com/careyi3/rhai-ops): a Rocket + Diesel + React app that exposes the same service layer over HTTP and through a Rhai REPL.

You can find the [crates.io listing here](https://crates.io/crates/rhai-console).

![CI](https://github.com/careyi3/rhai-console/actions/workflows/test.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/rhai-console.svg)](https://crates.io/crates/rhai-console)
[![Crates.io](https://img.shields.io/crates/d/rhai-console.svg)](https://crates.io/crates/rhai-console)

## Features

- Interactive REPL with multi-line input and history
- Tab completion for module namespaces, registered functions (with signatures), and in-scope variables
- Run `.rhai` scripts from disk with positional args injected as an `args` array
- Back traces pointing into the source for runtime error
- Colored output that auto-detects the terminal and honors `NO_COLOR`
- Available-modules listing generated automatically from your registrations
- Simple function registration using the `reg!` macro that wraps each call with position capture, error mapping, and serde result conversion

## Setup

Add the crate:

```bash
$ cargo add rhai-console
 
```

A minimal `main.rs`:

```rust
use rhai_console::Console;

mod modules;

fn main() -> anyhow::Result<()> {
    let services = bootstrap()?;

    Console::new(services)
        .intro("my-app REPL")
        .module("products", modules::products::register)
        .module("orders",   modules::orders::register)
        .run()?;

    Ok(())
}
```

A module-builder fn:

```rust
use rhai_console::prelude::*;

pub fn register(m: &mut Module, svc: Services) {
    reg!(m, svc, "list",        |s|                       s.products().list());
    reg!(m, svc, "find",        |s, id: i64|              s.products().find(id.try_into()?));
    reg!(m, svc, "find_by_sku", |s, sku: ImmutableString| s.products().find_by_sku(&sku));
}
```

## Running

Without arguments, you get an interactive REPL:

```bash
$ my-app-repl
 
```

Pass a path to run a script instead:

```bash
$ my-app-repl scripts/seed.rhai
 
```

Note: extra positional args go into the script as an `args` array of strings.

```bash
$ my-app-repl scripts/place_order.rhai ANV-001 5
 
```

The script sees `args[0] == "ANV-001"` and `args[1] == "5"`.

By convention, commit `.rhai` files to a `scripts/` dir in your repo. They will have access to the same modules as the REPL.

## Color

Output (prompt, result values, errors, and tracebacks) is colored by default, with sensible auto-detection:

- color is enabled only when the relevant stream is a terminal, so piping or redirecting produces clean, plain text;
- the [`NO_COLOR`](https://no-color.org/) standard is honored — set `NO_COLOR=1` to disable color, or `CLICOLOR_FORCE=1` to force it on for a non-terminal.

The embedding application can override this explicitly, which takes precedence over the environment. Wire it up to your own flags or config:

```rust
use rhai_console::{Console, ColorChoice};

Console::new(services)
    .color(ColorChoice::Never) // Auto (default) | Always | Never
    .run()?;
```

## Development

This repo uses [`just`](https://github.com/casey/just) as a task runner. Run `just` with no arguments to list the available recipes:

```bash
$ just
```

Common recipes:

| Recipe       | What it does                                  |
| ------------ | --------------------------------------------- |
| `just build` | Build the library and examples                |
| `just lint`  | Format code, then run clippy (warnings denied) |
| `just test`  | Run the test suite (`cargo test`)             |
| `just demo`  | Run the interactive demo REPL                 |

You don't need `just` installed; every recipe is a thin wrapper over a `cargo` command you can run directly.

### Trying it out locally

A runnable example lives in [`examples/demo.rs`](examples/demo.rs): a tiny app exposing a handful of trivial functions across a few modules. It's the quickest way to play with the REPL, history, and tab completion:

```bash
$ just demo
# or:
$ cargo run --example demo
```

Examples are dev-only — they are never compiled into the published crate or into downstream consumers' builds.
