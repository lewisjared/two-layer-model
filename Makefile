.DEFAULT_GOAL := all
sources = python/two_layer_model tests

# using pip install cargo (via maturin via pip) doesn't get the tty handle
# so doesn't render color without some help
export CARGO_TERM_COLOR=$(shell (test -t 0 && echo "always") || echo "auto")
# maturin develop only makes sense inside a virtual env, is otherwise
# more or less equivalent to pip install -e just a little nicer
USE_MATURIN = $(shell [ "$$VIRTUAL_ENV" != "" ] && (which maturin))

.PHONY: install
install:
	uv venv
	pre-commit install

.PHONY: build-dev
build-dev:
	@rm -f python/two_layer_model/*.so
ifneq ($(USE_MATURIN),)
	maturin develop
else
	pip install -v -e . --config-settings=build-args='--profile dev'
endif

.PHONY: build-prod
build-prod:
	@rm -f python/two_layer_model/*.so
ifneq ($(USE_MATURIN),)
	maturin develop --release
else
	pip install -v -e .
endif

.PHONY: format
format:
	ruff check --fix $(sources)
	ruff format $(sources)
	cargo fmt

.PHONY: lint-python
lint-python:
	ruff check $(sources)
	ruff format --check $(sources)

.PHONY: lint-rust
lint-rust:
	cargo fmt --version
	cargo fmt --all -- --check
	cargo clippy --version
	cargo clippy --tests -- -D warnings

.PHONY: lint
lint: lint-python lint-rust

.PHONY: test
test:
	pytest

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
