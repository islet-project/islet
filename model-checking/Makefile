KANI_FLAGS := \
	--enable-unstable \
	--ignore-global-asm \
	--restrict-vtable

RMI_TARGET ?= rmi_features
ALLOWED_RMI_TARGET = \
	rmi_features

ifeq ("$(filter $(RMI_TARGET), $(ALLOWED_RMI_TARGET))", "")
    $(error Invalid RMI_TARGET. Choose one of the following: $(ALLOWED_RMI_TARGET))
endif

FEATURE_FLAGS := \
	--features \
	mc_$(RMI_TARGET)

.PHONY: verify
verify:
	cargo kani $(KANI_FLAGS) $(FEATURE_FLAGS)

.PHONY: clean
clean:
	cargo clean
