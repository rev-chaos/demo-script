TARGET := riscv64-unknown-linux-gnu
CC := $(TARGET)-gcc
LD := $(TARGET)-gcc
OBJCOPY := $(TARGET)-objcopy
CFLAGS := -fPIC -O3 -fno-builtin-printf -fno-builtin-memcmp -nostdinc -nostdlib -nostartfiles -fvisibility=hidden -fdata-sections -ffunction-sections -I deps/secp256k1/src -I deps/secp256k1 -I deps/ckb-c-std-lib -I deps/ckb-c-std-lib/libc -I deps/ckb-c-std-lib/molecule -I c -I build -Wall -Werror -Wno-nonnull -Wno-nonnull-compare -Wno-unused-function -g
LDFLAGS := -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections
SECP256K1_SRC := deps/secp256k1/src/ecmult_static_pre_context.h
PROTOCOL_HEADER := c/blockchain.h
PROTOCOL_SCHEMA := c/blockchain.mol
PROTOCOL_VERSION := d75e4c56ffa40e17fd2fe477da3f98c5578edcd1
PROTOCOL_URL := https://raw.githubusercontent.com/nervosnetwork/ckb/${PROTOCOL_VERSION}/util/types/schemas/blockchain.mol

# docker pull nervos/ckb-riscv-gnu-toolchain:gnu-bionic-20191012
BUILDER_DOCKER := nervos/ckb-riscv-gnu-toolchain@sha256:aae8a3f79705f67d505d1f1d5ddc694a4fd537ed1c7e9622420a470d59ba2ec3

all: build/always_success build/demo

all-via-docker: ${PROTOCOL_HEADER}
	docker run --rm -v `pwd`:/code ${BUILDER_DOCKER} bash -c "cd /code && make"

build/always_success: c/always_success.c
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<
	$(OBJCOPY) --only-keep-debug $@ $@.debug
	$(OBJCOPY) --strip-debug --strip-all $@

build/demo: c/demo.c
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<
	$(OBJCOPY) --only-keep-debug $@ $@.debug
	$(OBJCOPY) --strip-debug --strip-all $@

$(SECP256K1_SRC):
	cd deps/secp256k1 && \
		./autogen.sh && \
		CC=$(CC) LD=$(LD) ./configure --with-bignum=no --enable-ecmult-static-precomputation --enable-endomorphism --enable-module-recovery --host=$(TARGET) && \
		make src/ecmult_static_pre_context.h src/ecmult_static_context.h

${PROTOCOL_SCHEMA}:
	curl -L -o $@ ${PROTOCOL_URL}

publish:
	git diff --exit-code Cargo.toml
	sed -i.bak 's/.*git =/# &/' Cargo.toml
	cargo publish --allow-dirty
	git checkout Cargo.toml Cargo.lock
	rm -f Cargo.toml.bak

package:
	git diff --exit-code Cargo.toml
	sed -i.bak 's/.*git =/# &/' Cargo.toml
	cargo package --allow-dirty
	git checkout Cargo.toml Cargo.lock
	rm -f Cargo.toml.bak

package-clean:
	git checkout Cargo.toml Cargo.lock
	rm -rf Cargo.toml.bak target/package/

clean:
	rm -rf build/*.debug
	cd deps/secp256k1 && [ -f "Makefile" ] && make clean
	cargo clean

dist: clean all

.PHONY: all all-via-docker dist clean package-clean package publish
