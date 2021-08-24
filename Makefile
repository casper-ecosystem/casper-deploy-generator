test-vectors:
	CL_TEST_SEED=c954046e102bdfb7c954046e102bdfb7 cargo +nightly run > output.txt

clean:
	cargo clean