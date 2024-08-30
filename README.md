# Two Layer Model

An over-engineered approach to the two layer model implemented using Rust.
This is intended as a PoC of the use of Rust for simple climate models and general testing.

## Design goals

* Fast
* Easy to extend
* Understandable by non-rust developers
* Can be driven from Python

## Getting Started

In this example we are going to use the `pyo3` crate to create a Python extension module.
This provides a mechanism to interact with the rust codebase from Python.

### Dependencies

* Rust
* [uv](https://github.com/astral-sh/uv) (Python package management)

after these dependencies have been installed the local Python environment can be initialised using:

```
make virtual-environment
```

Since Rust is a compiled language,
the extension module must be recompiled after any changes to the Rust code.
This can be done using:

```
make build-dev
```

### Tests

Rust unit tests are embedded alongside the implementation files.
These tests can be built and run using the following (or using RustRover):

```
cargo test
```

### Stub generation

To make it easier to consume the extension module,

In our case the exposed interface is probably simple enough to write the `.pyi` file by hand,
but since we are playing around we can use [pyo3-stub-gen](https://github.com/Jij-Inc/pyo3-stub-gen) to automatically
generate the stubs.

The command to run is:

```
cargo run --bin stub_gen
```
