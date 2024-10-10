MODEL_CHECKING_DIR := $(CURDIR)/src

KANI_FLAGS := \
	--enable-unstable \
	--ignore-global-asm \
	--restrict-vtable \
	--c-lib $(MODEL_CHECKING_DIR)/stub.c \
	-Z c-ffi

RMI_TARGET ?= rmi_features
ALLOWED_RMI_TARGET = \
	rmi_features \
	rmi_granule_delegate \
	rmi_granule_undelegate \
	rmi_realm_activate \
	rmi_rec_aux_count \
	rmi_rec_destroy \
	rmi_version

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
