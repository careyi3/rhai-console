# rhai-console

Operational REPL and script runner for embedding Rhai in Rust applications.

Heavily inspired by the Rails Console and Django Shell; all you need to configure is your application state and a few module registrations, and you get back a CLI with an interactive REPL and script runner all of which run within your applications context.

You can find an example here [rhai-ops](https://github.com/careyi3/rhai-ops): a Rocket + Diesel + React app that exposes the same service layer over HTTP and through a Rhai REPL.

You can find the [crates.io listing here](https://crates.io/crates/rhai-console).

![Test](https://github.com/careyi3/rhai-console/actions/workflows/test.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/rhai-console.svg)](https://crates.io/crates/rhai-console)
[![Crates.io](https://img.shields.io/crates/d/rhai-console.svg)](https://crates.io/crates/rhai-console)

## Features

- Interactive REPL with multi-line input and history
- Run `.rhai` scripts from disk with positional args injected as an `args` array
- Back traces pointing into the source for runtime error
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

## Tests

```bash
$ cargo test
 
```
