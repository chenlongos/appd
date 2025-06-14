# Architecture and platform resolving

# Builtin platforms in [axhal_crates](https://github.com/arceos-org/axplat_crates/tree/main/platforms)
builtin_platforms := x86_64-qemu-q35 \
    riscv64-qemu-virt \
    aarch64-qemu-virt \
    aarch64-raspi4 \
    aarch64-bsta1000b \
    aarch64-phytium-pi \
    loongarch64-qemu-virt

# Resolve the path of platform configuration file
define resolve_plat_config
  $(shell cargo axplat info axplat-$(PLAT_PACKAGE)  | grep '^config_path:' | cut -d ' ' -f2-)
endef
ifeq ($(MYPLAT),)
  # `PLATFORM` is not specified, use the default platform for each architecture
  ifeq ($(ARCH), x86_64)
    PLAT_NAME := x86_64-qemu-q35
    PLAT_PACKAGE := x86-pc
  else ifeq ($(ARCH), aarch64)
    PLAT_NAME := aarch64-qemu-virt
    PLAT_PACKAGE := aarch64-qemu-virt
  else ifeq ($(ARCH), riscv64)
    PLAT_NAME := riscv64-qemu-virt
    PLAT_PACKAGE := riscv64-qemu-virt
  else ifeq ($(ARCH), loongarch64)
    PLAT_NAME := loongarch64-qemu-virt
    PLAT_PACKAGE := loongarch64-qemu-virt
  else
    $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
  endif
  PLAT_CONFIG := $(call resolve_plat_config)
else
  # `PLATFORM` is specified, override the `ARCH` variables
  ifneq ($(filter $(MYPLAT), $(builtin_platforms)),)
    # builtin platform
    ifeq ($(MYPLAT), x86_64-qemu-q35)
      PLAT_PACKAGE := x86-pc
    else ifeq ($(MYPLAT), aarch64-raspi4)
      PLAT_PACKAGE := aarch64-raspi
    else
      PLAT_PACKAGE := $(MYPLAT)
    endif
    PLAT_NAME := $(MYPLAT)
    PLAT_CONFIG := $(call resolve_plat_config)
    _arch := $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r arch))
  else ifneq ($(wildcard $(MYPLAT)),)
    # custom platform, read the "arch" and "plat-name" fields from the toml file
    _arch :=  $(patsubst "%",%,$(shell axconfig-gen $(MYPLAT) -r arch))
    PLAT_NAME := $(patsubst "%",%,$(shell axconfig-gen $(MYPLAT) -r platform))
    # 需要去除前后的 " 和 "，因为 axconfig-gen 返回的值可能包含引号
    PLAT_PACKAGE := $(patsubst "%",%,$(shell axconfig-gen $(MYPLAT) -r package))
    PLAT_CONFIG := $(MYPLAT)
  else
    builtin_platforms := $(foreach pair,$(builtin_platforms_map),$(firstword $(subst :, ,$(pair))))
    $(error "MYPLAT" must be a valid path to a toml file)
  endif
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "PLAT_NAME=$(PLAT_NAME)")
    endif
  endif
  ARCH := $(_arch)
endif

default_package := x86-pc riscv64-qemu-virt aarch64-qemu-virt loongarch64-qemu-virt
ifeq ($(filter $(PLAT_PACKAGE),$(default_package)),)
  # If `PLAT_PACKAGE` is not one of the default packages, then it must be a custom package.
  # so disable `defplat` feature and enable `myplat` feature
  FEATURES := $(filter-out defplat,$(FEATURES))
  override FEATURES += myplat
else
  # If `PLAT_PACKAGE` is one of the default packages, then enable `defplat` feature
  FEATURES := $(filter-out myplat,$(FEATURES))
  override FEATURES += defplat
endif