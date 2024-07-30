# embassy-imxrt-examples

## Introduction

These examples illustrates how to use the embassy-imxrt HAL.

## Adding Examples

Add uniquely named example to `src/bin` like `hello-world.rs`

## Prerequisite tools

### probe-rs-tools

```shell
cargo install probe-rs-tools --git https://github.com/probe-rs/probe-rs --locked
```

Used to download bits to and debug the target device

### flip-link

```shell
cargo install flip-link --locked
```

Handle stack overflows better with a hardware exception by positioning the stack smartly.  Used by build process directly during linking.

cargo install cargo-bloat --locked

### bloat

```shell
cargo install cargo-bloat --locked
```

Run with `cargo bloat` to see the function memory usage for the built binary

## Build

`cd` to examples/rt685s-evk folder
`cargo build --bin <example_name>` for example, `cargo build --bin hello-world`

## Run

Assuming RT685 is powered and connected to Jlink debug probe and the latest probe-rs is installed:

- `cd` to examples/rt685s-evk folder
- `cargo run --bin <example_name>` for example, `cargo run --bin hello-world`
