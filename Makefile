# This supports environments where $HOME/.cargo/env has not been sourced (CI, CLion Makefile runner)
CARGO  = $(or $(shell which cargo),  $(HOME)/.cargo/bin/cargo)

PINNED_NIGHTLY := $(shell cat rust-toolchain)

CARGO_OPTS := --locked
CARGO_PINNED_NIGHTLY := $(CARGO) +$(PINNED_NIGHTLY) $(CARGO_OPTS)
CARGO := $(CARGO) $(CARGO_OPTS)

# !!!!!!! DO NOT CHANGE THE TEST SEED UNLESS YOU KNOW WHAT YOU'RE DOING !!!!!!!
# The test seed below is used to feed the PRNG that later is responsible for generating random data for the test vectors.
# Since we're reusing it, and it's D=Deterministic, we are guaranteed to always generate the same "random" data for the vectors,
# meaning, no mather how many times we re-generate it we will keep getting the same data in `output.txt` == no diff.
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