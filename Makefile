# This supports environments where $HOME/.cargo/env has not been sourced (CI, CLion Makefile runner)
CARGO  = $(or $(shell which cargo),  $(HOME)/.cargo/bin/cargo)

PINNED_NIGHTLY := $(shell cat rust-toolchain)

CARGO_OPTS := --locked
CARGO_PINNED_NIGHTLY := $(CARGO) +$(PINNED_NIGHTLY) $(CARGO_OPTS)
CARGO := $(CARGO) $(CARGO_OPTS)

test-vectors:
	CL_TEST_SEED=c954046e102bdfb7c954046e102bdfb7 $(CARGO) run > output.txt

check: 
	$(CARGO) check

format:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy

clean:
	$(CARGO) clean