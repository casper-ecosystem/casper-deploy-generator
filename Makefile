# This supports environments where $HOME/.cargo/env has not been sourced (CI, CLion Makefile runner)
CARGO  = $(or $(shell which cargo),  $(HOME)/.cargo/bin/cargo)

CARGO_OPTS := --locked
CARGO := $(CARGO) $(CARGO_OPTS) --quiet

# Do not echo commands
$(V).SILENT:

# !!!!!!! DO NOT CHANGE THE TEST SEED UNLESS YOU KNOW WHAT YOU'RE DOING !!!!!!!
# The test seed below is used to feed the PRNG that later is responsible for generating random data for the test vectors.
# Since we're reusing it, and it's D=Deterministic, we are guaranteed to always generate the same "random" data for the vectors,
# meaning, no mather how many times we re-generate it we will keep getting the same data in `output.txt` == no diff.
test-vectors:
	cp manual.json old_manual.json && \
	CL_TEST_SEED=c954046e102bdfb7c954046e102bdfb7 $(CARGO) run > manual.json

# To check whether any of the old entries have changed.
# If we see any difference in previously-generated entries it might mean we're breaking backwards compatibility.
# ANALYZE WITH CAUTION
check-against-old: test-vectors
	diff old_manual.json manual.json > test_vectors_diff.txt && \
	RESULT=$(![ -s test_vectors_diff.txt ]) || echo "WARNING: diff file is non-empty. Check test_vectors_diff.txt file." && \
	rm old_manual.json

check:
	$(CARGO) check

format:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy

clean:
	$(CARGO) clean
