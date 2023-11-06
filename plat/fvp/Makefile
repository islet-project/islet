KANI_FLAGS := \
	--enable-unstable \
	--ignore-global-asm \
	--restrict-vtable

.PHONY: verify
verify:
	cargo kani $(KANI_FLAGS)

.PHONY: clean
clean:
	cargo clean
