# Phytium Pi 网络驱动测试方法文档

## 概述

本文档描述了 ArceOS 中 Phytium Pi 平台上 pice, fxmac 和 igb 网络驱动的测试方法和测试流程。

使用命令
```
make A=examples/helloworld PLATFORM=aarch64-phytium-pi LOG=info build
```
进行测试，为了方便调试，使用了 copy_and_run.py 脚本（需要进行配置，详见脚本内注释），使用命令

```
make A=examples/helloworld PLATFORM=aarch64-phytium-pi LOG=info build && python3 copy_and_run.py 
```

## PCIe 测试方法

### 测试函数位置
- 文件：`modules/axhal/src/platform/aarch64_phytium_pi/igb/mod.rs`
- 主要测试函数：`test_igb_basic()` 中的PCIe相关部分
- PCIe驱动框架：使用 `pcie` crate 提供的通用PCIe驱动

可以枚举到 igb 网卡


## FXMAC 驱动测试方法

### 测试函数位置
- 文件：`modules/axhal/src/platform/aarch64_phytium_pi/fxmac/mod.rs`
- 主要测试函数：`test_fxmac_simple()`

### 测试流程

#### 1. 初始化测试
```rust
// 使用硬件MAC地址进行初始化
match init_fxmac_driver(None) {
    Ok(fxmac_instance) => { /* 继续测试 */ }
    Err(e) => { /* 初始化失败 */ }
}
```

#### 2. 链路状态检查
```rust
// 检查网络链路状态
let link_status = check_link_status(fxmac_instance);
if link_status == FXMAC_LINKUP {
    // 链路已建立，继续网络功能测试
}
```

#### 3. 数据包发送测试
- **测试类型**：ARP广播包发送测试
- **测试包数量**：10个测试包
- **包格式**：标准ARP请求包
  - 目标MAC：广播地址 (FF:FF:FF:FF:FF:FF)
  - 源MAC：动态获取的硬件MAC地址
  - 协议类型：ARP (0x0806)
  - 发送方IP：192.168.1.100
  - 目标IP：192.168.1.1 (网关)

目前发送的时候接收方可以收到包，但是会出现重复byte，有可能是速率没有协商好

#### 4. 数据包接收测试
- **测试轮数**：500轮接收尝试
- **接收方式**：轮询模式，调用 `FXmacRecvHandler`
- **延时控制**：每轮之间有延时，避免CPU占用过高

可以收到包

```rust
// 接收测试循环
for round in 0..500 {
    match receive_packets(fxmac_instance) {
        Some(packets) => total_received += packets.len(),
        None => /* 无数据包 */
    }
}
```


## IGB 驱动测试方法

### 测试函数位置
- 文件：`modules/axhal/src/platform/aarch64_phytium_pi/igb/mod.rs`
- 主要测试函数：`test_igb_basic()`

### 测试流程

#### 1. PCIe 设备枚举和配置 
请注意，phytium 似乎会在最开始配置好pcie 的 bar 空间，所以测试的时候没有配置bar
```rust
// PCIe 根复合体初始化
let mut root = RootComplexGeneric::new(base_vaddr);

// 枚举并查找 IGB 设备
for header_elem in root.enumerate_keep_bar(None) {
    if Igb::check_vid_did(endpoint.vendor_id, endpoint.device_id) {
        // 配置PCIe设备
        configure_pcie_device(&mut endpoint, header_elem.root);
    }
}
```

#### 2. 设备初始化测试
- **VID/DID检查**：验证设备ID (VID=0x8086, DID=0x10C9/0x1533)
- **BAR映射**：获取BAR0地址并映射到虚拟地址空间
- **设备配置**：启用内存访问、I/O访问和总线主控模式
- **中断配置**：基础测试中禁用MSI/MSI-X，使用轮询模式

#### 3. MAC地址读取和状态检查
```rust
// 读取MAC地址
let mac_addr = igb.read_mac();

// 检查设备状态
let initial_status = igb.status();
let final_status = igb.status();
```

#### 4. 链路状态监控
- **等待时间**：最多等待10秒链路建立
- **状态轮询**：每秒检查一次链路状态
- **容错设计**：链路未建立也继续测试，适应不同网络环境

#### 5. 环回测试
```rust
fn test_loopback(igb: &mut Igb, _tx_ring: &mut TxRing, _rx_ring: &mut RxRing) {
    // 启用硬件环回模式
    igb.enable_loopback();
    
    // 进行基本状态检查
    let status = igb.status();
    
    // 禁用环回模式
    igb.disable_loopback();
}
```

## 预期输出
在minipcie 插上 igb 网卡的情况下
见同文件夹下 test.log
虽然log中fxmac并未接收到包，但是多循环几轮是可以接收到包的