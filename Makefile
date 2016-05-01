run:
	./target/debug/val

build:
	cargo build

git1:
	git add -n .
	git add .

git2:
	git diff
	git status

git3:
	git commit --dry-run
