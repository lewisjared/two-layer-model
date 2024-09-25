# Rust Simple Climate Model (RSCM)

This is intended as a PoC of the use of Rust for simple climate models.
The aim is to provide a framework for building components in both Rust and Python.

## Design goals

* Fast
* Easy to extend
* Understandable by non-Rust developers
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


### Documentation

The documentation for the Rust codebase can be built using:

```
make docs-rust
```
