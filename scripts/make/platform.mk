# Architecture and platform resolving

resolve_config = \
  $(if $(wildcard $(PLAT_CONFIG)),\
    $(if $(filter "$(PLAT_PACKAGE)",$(shell axconfig-gen $(PLAT_CONFIG) -r package)),\
      $(PLAT_CONFIG),\
      $(error "PLAT_CONFIG=$(PLAT_CONFIG)" is not compatible with "PLAT_PACKAGE=$(PLAT_PACKAGE)")),\
    $(shell cargo axplat info -c $(PLAT_PACKAGE) 2>/dev/null))

ifeq ($(MYPLAT),)
  # `MYPLAT` is not specified, use the default platform for each architecture
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
  PLAT_CONFIG := $(resolve_config)  
  # We don't need to check the validity of `PLAT_CONFIG` here, as the `PLAT_PACKAGE`
  # is a valid pacakage. So if the `PLAT_CONFIG` is not compatible with the `PLAT_PACKAGE`,
  # it will be caught by the `resolve_config` function.
  
else
  # `MYPLAT` is specified, treat it as a package name
  PLAT_PACKAGE := $(MYPLAT)
  # We have checked the validity of `MYPLAT`, so the `PLAT_CONFIG` should be valid too.
  PLAT_CONFIG := $(resolve_config)
  ifeq ($(strip $(PLAT_CONFIG)),)
    $(error "MYPLAT=$(MYPLAT) is not a valid platform package name")
  endif

  # Read the architecture name from the configuration file
  _arch := $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r arch))
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "MYPLAT=$(MYPLAT)")
    endif
  endif
  ARCH := $(_arch)
endif

PLAT_NAME := $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r platform))
