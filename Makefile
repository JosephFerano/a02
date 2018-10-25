all: build

dist: build
	@rm -f assig-02-mem-mng.tar.gz
	@cp target/release/optimal optimal
	@cp target/release/second second
	@cp target/release/wsclock wsclock
	@tar -czf assig-02-mem-mng.tar.gz src/ README.md optimal second wsclock input1.txt input2.txt
	@rm optimal second wsclock

build:
	cargo build --release
