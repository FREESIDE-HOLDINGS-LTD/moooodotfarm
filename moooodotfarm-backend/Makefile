.PHONY: ci
ci: test-verbose fix clippy fmt check-repository-unchanged

.PHONY: cleanup
cleanup: fix clippy fmt

.PHONY: test
test:
	cargo test

.PHONY: test-verbose
test-verbose:
	RUST_BACKTRACE=1 cargo test -- --nocapture

.PHONY: clippy
clippy:
	cargo clippy --all-targets -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: fix
fix:
	cargo fix --allow-staged --allow-dirty --all-targets
	cargo clippy --fix --allow-staged --allow-dirty --all-targets
	cargo fmt

.PHONY: check-repository-unchanged
check-repository-unchanged:
	../check_repository_unchanged.sh
