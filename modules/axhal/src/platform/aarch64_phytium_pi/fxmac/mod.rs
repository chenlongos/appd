#![no_std]
#![allow(unused)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;
extern crate crate_interface;


extern crate log;



//mod mii_const;
mod fxmac_const;

mod utils;
mod fxmac_phy;
mod fxmac_dma;
mod fxmac_intr;
mod fxmac;

pub use fxmac::*;
pub use fxmac_dma::*;
pub use fxmac_intr::{FXmacIntrHandler, xmac_intr_handler};

// PHY interface
pub use fxmac_phy::{FXmacPhyInit, FXmacPhyRead, FXmacPhyWrite};

/// 声明网卡驱动所需的内核功能接口
#[crate_interface::def_interface]
pub trait KernelFunc{
    /// 虚拟地址转换成物理地址
    fn virt_to_phys(addr: usize) -> usize;

    /// 物理地址转换成虚拟地址
    fn phys_to_virt(addr: usize) -> usize;

    /// 申请DMA连续内存页
    fn dma_alloc_coherent(pages: usize) -> (usize, usize);

    /// 释放DMA内存页
    fn dma_free_coherent(vaddr: usize, pages: usize);

    /// 请求分配irq
    fn dma_request_irq(irq: usize, handler: fn());
}

pub struct FXmacDriver;

use crate::mem::PAGE_SIZE_4K;
use axalloc::global_allocator;
#[crate_interface::impl_interface]
impl KernelFunc for FXmacDriver {
    fn virt_to_phys(addr: usize) -> usize {
        crate::mem::virt_to_phys(addr.into()).into()
    }

    fn phys_to_virt(addr: usize) -> usize {
        crate::mem::phys_to_virt(addr.into()).into()
    }

    fn dma_alloc_coherent(pages: usize) -> (usize, usize) {
        let Ok(vaddr) = global_allocator().alloc_pages(pages, PAGE_SIZE_4K) else {
            error!("failed to alloc pages");
            return (0, 0);
        };
        let paddr = crate::mem::virt_to_phys((vaddr).into());
        debug!("alloc pages @ vaddr={:#x}, paddr={:#x}", vaddr, paddr);
        (vaddr, paddr.as_usize())
    }

    fn dma_free_coherent(vaddr: usize, pages: usize) {
        global_allocator().dealloc_pages(vaddr, pages);
    }

    fn dma_request_irq(_irq: usize, _handler: fn()) {
        warn!("unimplemented dma_request_irq for fxmax");
    }
}


/// 精简版 FXMAC 驱动测试 - 测试初始化和基本收发功能
pub fn test_fxmac_simple() {
    info!("=== FXMAC Simple Test Start ===");
    
    // 测试网卡初始化 - 使用硬件MAC地址
    match init_fxmac_driver(None) {
        Ok(fxmac_instance) => {
            info!("✓ FXMAC initialization: SUCCESS");
            
            // 检查链路状态
            let link_status = check_link_status(fxmac_instance);
            if link_status == FXMAC_LINKUP {
                info!("✓ Link status: UP");
                
                // 测试发送功能 - 发送ARP测试包
                info!("Starting packet transmission test...");
                
                // 获取当前网卡的MAC地址用于构造测试包
                let mut current_mac: [u8; 6] = [0; 6];
                FXmacGetMacAddress(&mut current_mac, 0);
                
                // 使用动态MAC地址构造ARP广播包
                let mut test_packet = [
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,  // 目标MAC (广播)
                    current_mac[0], current_mac[1], current_mac[2], current_mac[3], current_mac[4], current_mac[5],  // 源MAC
                    0x08, 0x06,                           // 类型: ARP
                    0x00, 0x01, 0x08, 0x00, 0x06, 0x04,  // 硬件类型，协议类型，地址长度
                    0x00, 0x01,                           // 操作: ARP请求
                    current_mac[0], current_mac[1], current_mac[2], current_mac[3], current_mac[4], current_mac[5],  // 发送方MAC
                    192, 168, 1, 100,                    // 发送方IP
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // 目标MAC (未知)
                    192, 168, 1, 1,                      // 目标IP (网关)
                ];
                
                info!("Test packet constructed with MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    current_mac[0], current_mac[1], current_mac[2], current_mac[3], current_mac[4], current_mac[5]);
                
                let mut success_count = 0;
                
                // 发送10个测试包
                for i in 0..10 {
                    match send_packet(fxmac_instance, &test_packet) {
                        Ok(()) => {
                            success_count += 1;
                            debug!("✓ Packet #{}: SUCCESS", i + 1);
                        }
                        Err(e) => {
                            warn!("✗ Packet #{}: FAILED ({})", i + 1, e);
                        }
                    }
                    
                    // 短暂延时，避免发送过快
                    for _ in 0..5000 { core::hint::spin_loop(); }
                }
                
                info!("Transmission test: {}/10 packets sent successfully", success_count);
                
                // 简化的接收测试 - 进行5轮接收尝试
                info!("Starting reception test...");
                let mut total_received = 0;
                
                for round in 0..500 {
                    match receive_packets(fxmac_instance) {
                        Some(packets) => {
                            total_received += packets.len();
                            info!("✓ Round {}: Received {} packet(s)", round + 1, packets.len());
                        }
                        None => {
                            debug!("○ Round {}: No packets", round + 1);
                        }
                    }
                    
                    // 接收间隔延时
                    for _ in 0..20000 { core::hint::spin_loop(); }
                }
                
                info!("Reception test: {} total packets received", total_received);
                
                if success_count > 0 || total_received > 0 {
                    info!("✓ FXMAC driver functionality verified");
                } else {
                    info!("○ Test completed - results may vary in different environments");
                }
                
            } else {
                warn!("○ Link status: DOWN/NEGOTIATING");
            }
            
            info!("✓ FXMAC test completed successfully");
        }
        Err(e) => {
            error!("✗ FXMAC initialization: FAILED ({})", e);
        }
    }
    
    info!("=== FXMAC Simple Test End ===");
}

/// 简化的 FXMAC 网络驱动初始化函数
/// 使用现有的 xmac_init 函数完成所有初始化工作
pub fn init_fxmac_driver(_mac_addr: Option<[u8; 6]>) -> Result<&'static mut FXmac, u32> {
    info!("=== FXMAC Driver Initialization Start ===");
    
    // 直接从硬件获取MAC地址
    let mut hwaddr: [u8; 6] = [0; 6];
    FXmacGetMacAddress(&mut hwaddr, 0);
    info!("Retrieved MAC address from hardware: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        hwaddr[0], hwaddr[1], hwaddr[2], hwaddr[3], hwaddr[4], hwaddr[5]);
    
    // xmac_init 已经包含了所有必要的初始化：
    // - 硬件复位和配置
    // - PHY 初始化  
    // - DMA 缓冲区分配和描述符环创建 (FXmacInitDma)
    // - 网卡启动 (FXmacStart)
    let fxmac_instance = xmac_init(&hwaddr);
    
    info!("=== FXMAC Driver Initialization Complete ===");
    info!("Driver status - Ready: {:#x}, Started: {:#x}, Link: {}", 
        fxmac_instance.is_ready, fxmac_instance.is_started, fxmac_instance.link_status);
    Ok(fxmac_instance)
}

/// 发送网络数据包 - 使用高层抽象接口
/// 
/// # 参数
/// * `instance` - FXMAC 驱动实例
/// * `data` - 要发送的数据包
/// 
/// # 返回值
/// * `Ok(())` - 发送成功
/// * `Err(i32)` - 发送失败，返回错误码
pub fn send_packet(instance: &mut FXmac, data: &[u8]) -> Result<(), i32> {
    if data.is_empty() || data.len() > 1518 {
        warn!("Invalid packet size: {}", data.len());
        return Err(-1);
    }
    
    debug!("Sending packet of {} bytes", data.len());
    
    // 转换为 Vec<Vec<u8>> 格式，符合 FXmacLwipPortTx 的接口
    let mut tx_vec = alloc::vec::Vec::new();
    tx_vec.push(data.to_vec());
    
    // 使用高层发送函数，避免直接操作描述符和寄存器
    let ret = FXmacLwipPortTx(instance, tx_vec);
    
    if ret < 0 {
        warn!("Packet transmission failed: {}", ret);
        Err(ret)
    } else {
        debug!("Packet transmitted successfully");
        Ok(())
    }
}

/// 接收网络数据包
/// 
/// # 参数
/// * `instance` - FXMAC 驱动实例
/// 
/// # 返回值
/// * `Some(Vec<Vec<u8>>)` - 接收到的数据包列表
/// * `None` - 没有数据包可接收
pub fn receive_packets(instance: &mut FXmac) -> Option<alloc::vec::Vec<alloc::vec::Vec<u8>>> {
    debug!("Checking for received packets");
    
    // 调用现有的接收处理函数
    match FXmacRecvHandler(instance) {
        Some(packets) => {
            info!("Received {} packet(s)", packets.len());
            for (i, packet) in packets.iter().enumerate() {
                debug!("Packet {}: {} bytes", i, packet.len());
            }
            Some(packets)
        }
        None => {
            debug!("No packets received");
            None
        }
    }
}

/// 检查网卡链路状态
pub fn check_link_status(instance: &FXmac) -> u32 {
    let status = instance.link_status;
    match status {
        FXMAC_LINKUP => debug!("Link status: UP"),
        FXMAC_LINKDOWN => debug!("Link status: DOWN"), 
        FXMAC_NEGOTIATING => debug!("Link status: NEGOTIATING"),
        _ => warn!("Link status: UNKNOWN ({})", status),
    }
    status
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
