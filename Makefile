all: run

EXE := ./target/debug/val
SRC := $(wildcard src/*.rs)
DATA := $(wildcard sample/*.bv)

run: $(EXE)
	$(EXE)

test: $(EXE)
	cargo test

build: $(EXE)

$(EXE): $(SRC) Makefile
	cargo build

help:
	echo "Targets include: run test build echo"

echo:
	echo "EXE: $(EXE)"
	echo "SRC: $(SRC)"

git1:
	git add -n .
	git add .

git2:
	git diff
	git status

git3:
	git commit --dry-run
	git commit

git4:
	git push -n
	git push
