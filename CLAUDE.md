# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

ArceOS is an experimental modular operating system (or unikernel) written in Rust. It supports multiple architectures (x86_64, riscv64, aarch64) and provides a modular design where features can be selectively enabled.

## Build Commands

Build and run applications using the Makefile with various options:

```bash
# Basic build and run (uses examples/helloworld by default)
make A=path/to/app ARCH=<arch> run

# Common architectures: x86_64, riscv64, aarch64
# Example: Build httpserver for aarch64 with networking
make A=examples/httpserver ARCH=aarch64 LOG=info SMP=4 run NET=y

# Build for specific platforms
make PLATFORM=aarch64-raspi4 A=examples/helloworld
make PLATFORM=x86_64-pc-oslab A=examples/httpserver FEATURES=driver-ixgbe
make A=examples/helloworld PLATFORM=aarch64-phytium-pi build  # Phytium Pi platform

# Available LOG levels: off, error, warn, info, debug, trace
# QEMU options: BLK=y (storage), NET=y (network), GRAPHIC=y (display)
```

Development commands:
```bash
# Code formatting and linting
cargo fmt --all                    # Format Rust code
make fmt_c                         # Format C code
make clippy                        # Run clippy lints

# Testing
make unittest                      # Run unit tests
make unittest_no_fail_fast        # Run all tests without stopping on failure

# Documentation
make doc                          # Generate documentation

# Debugging
make debug                        # Start with GDB debugging

# Clean build artifacts
make clean
```

## High-Level Architecture

ArceOS follows a modular architecture with three main layers:

### Core Modules (`modules/`)
- **axhal**: Hardware abstraction layer providing platform-specific operations
- **axruntime**: Runtime library that initializes the system and calls application main
- **axalloc**: Global memory allocator
- **axtask**: Task/thread management and scheduling
- **axnet**: Network stack using smoltcp
- **axfs**: File system support
- **axmm**: Memory management and paging
- **axsync**: Synchronization primitives
- **axdriver**: Device driver framework

### APIs (`api/`)
- **arceos_api**: High-level ArceOS API
- **arceos_posix_api**: POSIX-compatible API layer
- **axfeat**: Feature configuration

### User Libraries (`ulib/`)
- **axstd**: ArceOS standard library (Rust std-like interface)
- **axlibc**: C library implementation for C applications

### Application Types
ArceOS supports two application types:
- **Rust apps**: Use `axstd` library with `no_std` + `no_main`
- **C apps**: Require `axbuild.mk` and `features.txt` files

### Platform Support
Platforms are defined in `platforms/` directory:
- QEMU: x86_64-qemu-q35, riscv64-qemu-virt, aarch64-qemu-virt
- Real hardware: aarch64-raspi4, aarch64-phytium-pi, x86_64-pc-oslab

### Feature System
ArceOS uses Cargo features for modular functionality. Key features include:
- `alloc`: Dynamic memory allocation
- `paging`: Page table manipulation
- `multitask`: Multi-threading support
- `smp`: Symmetric multiprocessing
- `fs`: File system support
- `net`: Networking support
- `irq`: Interrupt handling

### Boot Process
1. **axhal** handles platform-specific bootstrapping
2. **axruntime** performs system initialization:
   - Initialize logging and allocator
   - Set up memory management and paging
   - Initialize device drivers
   - Start scheduler and secondary CPUs
   - Call application `main()`

## Development Workflow

1. Applications can be built either inside the ArceOS tree (`examples/`) or externally
2. External Rust apps should depend on `axstd` and use `make -C /path/to/arceos A=$(pwd)` to build
3. C applications require `axbuild.mk` for build configuration and `features.txt` for feature selection
4. Use `make run` to build and execute in QEMU, or `make build` for build-only
5. Platform-specific features can be enabled via `FEATURES` variable

## Important Notes

- Applications must use `#[no_mangle]` for the main function when using axstd
- C applications are built using musl cross-compilation toolchains
- The build system automatically detects application type (Rust vs C) based on presence of Cargo.toml
- Memory regions and device initialization are logged during boot for debugging

## Phytium Pi Driver Development

当前项目正在整理和开发 Phytium Pi 平台的驱动程序，目标目录为 `modules/axhal/src/platform/aarch64_phytium_pi/`。

该平台已实现的驱动包括：
- **clock.rs**: 时钟管理
- **cru.rs**: 时钟复位单元，包含 reset 和 pinmux 功能
- **pinctrl.rs**: 引脚控制
- **uart.rs**: 串口通信
- **pwm.rs**: PWM 脉冲宽度调制
- **tacho.rs**: 转速计
- **mio.rs**: 多功能 IO
- **mem.rs**: 内存管理
- **fxmac/**: 网络控制器驱动
- **i2c/**: I2C 总线驱动

驱动开发注意事项：
- 所有平台特定驱动应放在 `modules/axhal/src/platform/aarch64_phytium_pi/` 目录下
- 驱动模块需要在 `mod.rs` 中正确声明和导出
- 遵循 ArceOS 的模块化设计原则，保持驱动的独立性和可复用性