# Architecture and platform resolving

ifeq ($(MYPLAT),)
  # `PLATFORM` is not specified, use the default platform for each architecture
  ifeq ($(ARCH), x86_64)
    PLAT_PACKAGE := axplat-x86-pc
  else ifeq ($(ARCH), aarch64)
    PLAT_PACKAGE := axplat-aarch64-qemu-virt
  else ifeq ($(ARCH), riscv64)
    PLAT_PACKAGE := axplat-riscv64-qemu-virt
  else ifeq ($(ARCH), loongarch64)
    PLAT_PACKAGE := axplat-loongarch64-qemu-virt
  else
    $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
  endif
  PLAT_CONFIG := $(shell cargo axplat info -c $(PLAT_PACKAGE) 2>/dev/null)
else
  # `PLATFORM` is specified, override the `ARCH` variables
  ifneq ($(wildcard $(MYPLAT)),)
    # custom platform, read the "plat-name" fields from the toml file
    PLAT_CONFIG := $(MYPLAT)
    PLAT_PACKAGE := $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r package))
  else 
    # treat `MYPLAT` as a package name
    PLAT_PACKAGE := $(MYPLAT)
    PLAT_CONFIG := $(shell cargo axplat info -c $(PLAT_PACKAGE) 2>/dev/null)
  endif
  ifeq ($(PLAT_CONFIG),)
    $(error "MYPLAT" must be a valid configuration file path or a valid package name)
  endif
  # Read the architecture name from the configuration file
  _arch :=  $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r arch))
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "MYPLAT=$(MYPLAT)")
    endif
  endif
  ARCH := $(_arch)
endif

PLAT_NAME := $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r platform))

default_package := axplat-x86-pc axplat-riscv64-qemu-virt axplat-aarch64-qemu-virt axplat-loongarch64-qemu-virt
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