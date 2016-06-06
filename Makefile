all: run

EXE := ./target/debug/val
SRC := $(wildcard src/*.rs)
DATA := $(wildcard sample/*.bv)

run: $(EXE)
	$(EXE)

test: $(EXE)
	cargo test

testn: $(EXE)
	cargo test -- --nocapture # don't hide stdout

testv: $(EXE)
	cargo test --verbose

build: $(EXE)

$(EXE): $(SRC) Makefile Cargo.toml
	cargo build

clean:
	cargo clean

help:
	echo "Targets include: run test build echo clean"

echo:
	echo "EXE: $(EXE)"
	echo "SRC: $(SRC)"

g1:
	git status

g2:
	git diff

g3:
	git add . # -n for dryrun

g4:
	git status

g5:
	git commit # --dry-run

g6:
	git push # -n for dryrun
