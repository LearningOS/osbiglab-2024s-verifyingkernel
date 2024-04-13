RELEASE ?= n
RUSTFLAGS := -C link-arg=-lc -C panic=abort
ifeq ($(RELEASE), n)
	MODE := debug
	RUSTFLAGS += -C opt-level=0 -C debuginfo=full -C debug-assertions=true -C overflow-checks=true -C incremental=target/debug/incremental
else
	MODE := release
	RUSTFLAGS += -C opt-level=2 -C debuginfo=none -C debug-assertions=false -C overflow-checks=false -C incremental=false
endif
LIBFLAGS := -L crate=target/$(MODE)
RUSTFLAGS += $(LIBFLAGS)

SRC := $(wildcard src/*.rs)
DEPS = spin
LIBS := $(patsubst %, target/$(MODE)/lib%.rlib, $(DEPS))

main: $(LIBS) $(SRC)
	verus --compile --no-verify src/main.rs $(RUSTFLAGS)

verify: $(LIBS) $(SRC)
	verus src/main.rs --expand-errors $(LIBFLAGS)

target/$(MODE)/lib%.rlib: Cargo.lock
	cargo build $(if $(RELEASE:n=),--release,) --package $*

clean:
	rm -f main
	cargo clean

.PHONY: verify deps clean
