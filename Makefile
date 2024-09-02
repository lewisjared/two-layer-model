.DEFAULT_GOAL := all
sources = python/two_layer_model tests

# using pip install cargo (via maturin via pip) doesn't get the tty handle
# so doesn't render color without some help
export CARGO_TERM_COLOR=$(shell (test -t 0 && echo "always") || echo "auto")

.PHONY: virtual-environment
virtual-environment:
	uv venv
	pre-commit install
	$(MAKE) build-dev


.PHONY: build-dev
build-dev:
	@rm -f python/two_layer_model/*.so
	uv run maturin develop

.PHONY: build-prod
build-prod:
	@rm -f python/two_layer_model/*.so
	uv run maturin develop --release

.PHONY: format
format:
	uv run ruff check --fix $(sources)
	uv run ruff format $(sources)
	cargo fmt

.PHONY: lint-python
lint-python:
	uv run ruff check $(sources)
	uv run ruff format --check $(sources)

.PHONY: lint-rust
lint-rust:
	cargo fmt --version
	cargo fmt --all -- --check
	cargo clippy --version
	cargo clippy --tests -- -D warnings

.PHONY: lint
lint: lint-python lint-rust

.PHONY: test-python
test-python: build-dev
	uv run pytest

.PHONY: test-rust
test-rust:
	cargo test --workspace

.PHONY: test
test: test-python test-rust

.PHONY: all
all: format build-dev lint test

.PHONY: clean
clean:
	rm -rf `find . -name __pycache__`
	rm -f `find . -type f -name '*.py[co]' `
	rm -f `find . -type f -name '*~' `
	rm -f `find . -type f -name '.*~' `
	rm -rf .cache
	rm -rf htmlcov
	rm -rf .pytest_cache
	rm -rf *.egg-info
	rm -f .coverage
	rm -f .coverage.*
	rm -rf build
	rm -rf perf.data*
	rm -rf python/two_layer_model/*.so
