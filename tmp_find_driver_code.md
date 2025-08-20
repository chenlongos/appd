# 驱动开发指南

## 第四章：中断

### GIC驱动

原本就有

### i2c的中断实现

https://github.com/Jasonhonghh/arceos_experiment/tree/usb-camera-base/crates/driver_i2c
需要摘一下，但是并未实现完中断控制，只实现了阻塞接收和发送

### spi的中断实现

已修复

## 第五章：高速传输类驱动

### DMA驱动开发

包含在fxmac中，可否直接用？

### PCI控制器驱动

应该在axdriver中？

### PCIe互联驱动

在axdriver中

## 第六章：网络通信类驱动

### 单元测试与调试

### PCIe网卡驱动基础

### IGB网卡驱动实现

已经搬移并修复

### GMAC以太网基础

### YT8521驱动实现

fxmac 已经实现

### net_device实现

TODO 并未发现

## WARNING

pwm 的驱动为了兼容 tock_registers和safe_mmio 新版库，做了大量破坏性修改，很可能不对
