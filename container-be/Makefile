
HAMSLIB=libhams.dylib


target/lib/$(HAMSLIB):
	@mkdir -p target/lib
	echo "Copy ${HAMSLIB} to container-be/target/lib"
	@false

cargo-build: target/lib/${HAMSLIB}
	cargo build
