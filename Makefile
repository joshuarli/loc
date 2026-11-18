PACKAGE    := laputa-loc
NAME       := loc
TEST_TARGETS := --lib --tests
HOST       := $(shell rustc -vV | awk '/^host:/ {print $$2}')
TARGET     ?= $(subst -unknown-linux-gnu,-unknown-linux-musl,$(HOST))

RUSTYBENCH_DEFAULT := $(CURDIR)/.work/tools/bin/rustybench
RUSTYBENCH ?= "$(RUSTYBENCH_DEFAULT)"

.PHONY: build test test-ci check lint bench rustybench release verify-release install

build:
	cargo build

test:
	@mkdir -p .work/tmp
	@if test -n "$$LOC_CI_IMAGE"; then \
	  docker run --rm --init --platform "$$LOC_CI_PLATFORM" -e CI \
	    -v "$(CURDIR):/loc" -w /loc "$$LOC_CI_IMAGE" \
	    sh -ec 'rustup component add clippy; cargo clippy --workspace --all-targets -- -D warnings; make test'; \
	elif test "$$LOC_CI_MODE" = macos; then \
	  TMPDIR="$(CURDIR)/.work/tmp" cargo test --workspace $(TEST_TARGETS) --no-fail-fast; \
	else \
	  TMPDIR="$(CURDIR)/.work/tmp" cargo test --quiet; \
	fi

test-ci:
	cargo test --quiet --profile release $(TEST_TARGETS)

check:
	@mkdir -p .work/tmp
	TMPDIR="$(CURDIR)/.work/tmp" cargo clippy --all-targets -- -D warnings
	TMPDIR="$(CURDIR)/.work/tmp" cargo test --quiet $(TEST_TARGETS)

lint:
	cargo clippy --all-targets -- -D warnings
	uvx ruff@0.17.3 check scripts

bench: $(if $(RUSTYBENCH),$(if $(subst "$(RUSTYBENCH_DEFAULT)",,$(RUSTYBENCH)),,rustybench))

rustybench:
	@git check-ignore -q .work/ || printf '\n.work/\n' >> "$$(git rev-parse --git-path info/exclude)"
	@mkdir -p "$(CURDIR)/.work/tmp" "$(CURDIR)/.work/tools"
	@if test ! -x "$(RUSTYBENCH_DEFAULT)"; then TMPDIR="$(CURDIR)/.work/tmp" cargo install rustybench --version 0.5.0 --locked --root "$(CURDIR)/.work/tools"; fi

bench:
	@$(RUSTYBENCH) baseline --root "$(CURDIR)" --baseline "$(CURDIR)/benches/baseline.json" -- cargo bench --bench bench

release:
	cargo clean -p $(PACKAGE) --profile dist --target $(TARGET)
	cargo build --profile dist --locked --target $(TARGET) \
	  -Z build-std=std \
	  -Z build-std-features=

verify-release:
	@test -f "target/$(TARGET)/dist/$(NAME)"
	@if echo "$(TARGET)" | grep -q -- '-linux-musl$$'; then \
		command -v readelf >/dev/null || { echo 'readelf is required for release verification'; exit 1; }; \
		file "target/$(TARGET)/dist/$(NAME)" | grep -Eq 'static-pie linked|statically linked' || { echo 'release is not statically linked'; exit 1; }; \
		file "target/$(TARGET)/dist/$(NAME)" | grep -q 'stripped' || { echo 'release is not stripped'; exit 1; }; \
		! readelf -l "target/$(TARGET)/dist/$(NAME)" | grep -q INTERP || { echo 'release has a dynamic ELF interpreter'; exit 1; }; \
		! readelf -d "target/$(TARGET)/dist/$(NAME)" | grep -q NEEDED || { echo 'release has dynamic dependencies'; exit 1; }; \
	else \
		echo "Skipping ELF checks for $(TARGET)"; \
	fi

install: release
	@set -eu; \
	  install_dir="$(HOME)/.local/bin"; \
	  mkdir -p "$$install_dir"; \
	  install -m 755 "target/$(TARGET)/dist/$(NAME)" "$$install_dir/$(NAME)"; \
	  if test "$$(uname -s)" = Darwin; then codesign -fs - "$$install_dir/$(NAME)"; fi

PYTHON ?= python3

.PHONY: mutants-diff
mutants-diff:
	$(PYTHON) -B tools/mutants_gate.py "$(or $(BASE),origin/master)"
