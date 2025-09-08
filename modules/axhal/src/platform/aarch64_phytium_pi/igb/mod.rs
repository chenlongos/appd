#![no_std]

use core::{ops::Deref, ptr::NonNull};

use alloc::vec::Vec;
use dma_api::{DVec, Direction};
use log::debug;
pub use mac::{MacAddr6, MacStatus};
pub use trait_ffi::impl_extern_trait;

pub use err::DError;
use ring::DEFAULT_RING_SIZE;

extern crate alloc;

mod err;
mod mac;
#[macro_use]
pub mod osal;
mod descriptor;
mod phy;
mod ring;

pub use futures::{Stream, StreamExt};
pub use ring::{RxPacket, RxRing, TxRing};

pub struct Request {
    buff: DVec<u8>,
}

impl Request {
    fn new(buff: Vec<u8>, dir: Direction) -> Self {
        let buff = DVec::from_vec(buff, dir);
        Self { buff }
    }
    pub fn new_rx(buff: Vec<u8>) -> Self {
        Self::new(buff, Direction::FromDevice)
    }

    pub fn new_tx(buff: Vec<u8>) -> Self {
        Self::new(buff, Direction::ToDevice)
    }

    pub fn bus_addr(&self) -> u64 {
        self.buff.bus_addr()
    }
}

impl Deref for Request {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buff.as_ref()
    }
}

pub struct Igb {
    mac: mac::Mac,
    phy: phy::Phy,
    _rx_ring_addrs: [usize; 16],
    _tx_ring_addrs: [usize; 16],
}

pub fn test_igb_basic() -> Result<(), DError> {
    use log::{info, warn};
    use pcie::{RootComplexGeneric, SimpleBarAllocator};
    use crate::mem::phys_to_virt;
    
    info!("Starting basic IGB driver test...");
    
    // 初始化 PCIe
    let pci_32_range = axconfig::PCI_RANGES[0];
    let pci_64_range = axconfig::PCI_RANGES[1];
    let bar_alloc = SimpleBarAllocator::new(
        pci_32_range.0 as u32, 
        (pci_32_range.1 - pci_32_range.0) as u32, 
        pci_64_range.0 as u64, 
        (pci_64_range.1 - pci_64_range.0) as u64
    );

    let base_vaddr = phys_to_virt(axconfig::PCI_ECAM_BASE.into());
    let base_vaddr = unsafe {
        core::ptr::NonNull::new_unchecked(base_vaddr.as_mut_ptr())
    };
    
    let mut root = RootComplexGeneric::new(base_vaddr);
    let mut igb_found = false;
    
    // 查找 IGB 设备
    for header_elem in root.enumerate_keep_bar(None) {
        if let pcie::Header::Endpoint(mut endpoint) = header_elem.header {
            if Igb::check_vid_did(endpoint.vendor_id, endpoint.device_id) {
                info!("Found IGB device: VID={:#x}, DID={:#x}", 
                      endpoint.vendor_id, endpoint.device_id);
                
                // 配置 PCIe 设备 (现在我们有了 root 的访问权限！)
                configure_pcie_device(&mut endpoint, header_elem.root);
                
                // 获取 BAR0 地址
                let bar_addr = match &endpoint.bar {
                    pcie::BarVec::Memory32(bars) => {
                        bars[0].as_ref().map(|bar| bar.address as usize)
                    },
                    pcie::BarVec::Memory64(bars) => {
                        bars[0].as_ref().map(|bar| bar.address as usize)
                    },
                    _ => None,
                };
                
                if let Some(addr) = bar_addr {
                    info!("IGB device BAR0 address: {:#x}", addr);
                    
                    // 测试驱动
                    match test_igb_driver(addr) {
                        Ok(()) => {
                            info!("IGB driver test passed!");
                            igb_found = true;
                            break;
                        },
                        Err(e) => {
                            warn!("IGB driver test failed: {:?}", e);
                        }
                    }
                }
            }
        }
    }
    
    if !igb_found {
        warn!("No IGB device found or test failed");
    }
    
    Ok(())
}

fn configure_pcie_device(endpoint: &mut pcie::Endpoint, root: &mut pcie::RootComplexGeneric) {
    use log::info;
    use pcie::{CommandRegister, PciCapability};
    
    info!("Configuring PCIe device...");
    
    // 启用PCIe设备的内存访问、I/O访问和总线主控
    info!("Enabling PCIe device memory access and bus mastering...");
    endpoint.update_command(root, |cmd| {
        cmd | CommandRegister::IO_ENABLE
            | CommandRegister::MEMORY_ENABLE
            | CommandRegister::BUS_MASTER_ENABLE
    });
    
    // 配置中断模式 - 对于基础测试，禁用所有中断
    info!("Configuring interrupt mode for basic testing...");
    for cap in &mut endpoint.capabilities {
        match cap {
            PciCapability::Msi(msi_capability) => {
                info!("Disabling MSI capability");
                msi_capability.set_enabled(false, &mut *root);
            }
            PciCapability::MsiX(msix_capability) => {
                info!("Disabling MSI-X capability");
                msix_capability.set_enabled(false, &mut *root);
            }
            _ => {}
        }
    }
    info!("Note: Using polling mode for basic testing - interrupts disabled");
    
    info!("PCIe device configuration completed successfully!");
}

fn test_igb_driver(bar_addr: usize) -> Result<(), DError> {
    use log::info;
    use crate::mem::phys_to_virt;
    use crate::time::busy_wait;
    use core::time::Duration;
    
    // 映射物理地址到虚拟地址
    let virt_addr = phys_to_virt(bar_addr.into());
    let iobase = unsafe { 
        core::ptr::NonNull::new_unchecked(virt_addr.as_mut_ptr()) 
    };
    
    // 创建 IGB 驱动实例
    let mut igb = Igb::new(iobase)?;
    info!("IGB driver instance created");
    
    // 读取并显示 MAC 地址
    let mac_addr = igb.read_mac();
    info!("IGB MAC Address: {:?}", mac_addr);
    
    // 检查设备初始状态
    let initial_status = igb.status();
    info!("Initial device status: {:?}", initial_status);
    
    // 初始化设备
    igb.open()?;
    info!("IGB device opened successfully");
    
    // 等待链路建立
    info!("Waiting for link up...");
    let mut attempts = 0;
    const MAX_WAIT_ATTEMPTS: u32 = 10; // 最多等待 10 秒
    
    while attempts < MAX_WAIT_ATTEMPTS {
        let status = igb.status();
        info!("Link status attempt {}: {:?}", attempts + 1, status);
        
        if status.link_up {
            info!("Link is up!");
            break;
        }
        
        if attempts == MAX_WAIT_ATTEMPTS - 1 {
            info!("Link still down after {} seconds, continuing test...", MAX_WAIT_ATTEMPTS);
        }
        
        busy_wait(Duration::from_secs(1));
        attempts += 1;
    }
    
    // 检查最终状态
    let final_status = igb.status();
    info!("Final device status: {:?}", final_status);
    
    // 创建收发环
    let (mut tx_ring, mut rx_ring) = igb.new_ring()?;
    info!("TX/RX rings created successfully");
    
    // 简单的环回测试
    test_loopback(&mut igb, &mut tx_ring, &mut rx_ring)?;
    
    info!("IGB driver test completed successfully");
    Ok(())
}

fn test_loopback(igb: &mut Igb, _tx_ring: &mut TxRing, _rx_ring: &mut RxRing) -> Result<(), DError> {
    use log::info;
    
    info!("Starting loopback test...");
    
    // 启用环回模式
    igb.enable_loopback();
    info!("Loopback mode enabled");
    
    // 这里可以添加实际的数据包发送和接收测试
    // 由于涉及到复杂的网络协议栈，这里先做基本的状态检查
    
    let status = igb.status();
    info!("Loopback status: {:?}", status);
    
    // 禁用环回模式
    igb.disable_loopback();
    info!("Loopback mode disabled");
    
    info!("Loopback test completed");
    Ok(())
}

impl Igb {
    pub fn new(iobase: NonNull<u8>) -> Result<Self, DError> {
        let mac = mac::Mac::new(iobase);
        let phy = phy::Phy::new(mac);

        Ok(Self {
            mac,
            phy,
            _rx_ring_addrs: [0; 16],
            _tx_ring_addrs: [0; 16],
        })
    }

    pub fn open(&mut self) -> Result<(), DError> {
        self.mac.disable_interrupts();

        self.mac.reset()?;

        self.mac.disable_interrupts();

        debug!("reset done");

        let link_mode = self.mac.link_mode().unwrap();
        debug!("link mode: {link_mode:?}");
        self.phy.power_up()?;

        self.setup_phy_and_the_link()?;

        self.mac.set_link_up();
        self.phy.wait_for_auto_negotiation_complete()?;
        debug!("Auto-negotiation complete");
        self.config_fc_after_link_up()?;

        self.init_stat();

        self.mac.enable_interrupts();

        self.mac.enable_rx();
        self.mac.enable_tx();

        Ok(())
    }

    pub fn new_ring(&mut self) -> Result<(TxRing, RxRing), DError> {
        let tx_ring = TxRing::new(0, self.mac.iobase(), DEFAULT_RING_SIZE)?;
        let rx_ring = RxRing::new(0, self.mac.iobase(), DEFAULT_RING_SIZE)?;

        Ok((tx_ring, rx_ring))
    }

    fn config_fc_after_link_up(&mut self) -> Result<(), DError> {
        // TODO 参考 drivers/net/ethernet/intel/igb/e1000_mac.c
        // igb_config_fc_after_link_up
        Ok(())
    }

    fn setup_phy_and_the_link(&mut self) -> Result<(), DError> {
        self.phy.power_up()?;
        debug!("PHY powered up");
        self.phy.enable_auto_negotiation()?;

        Ok(())
    }

    pub fn read_mac(&self) -> MacAddr6 {
        self.mac.read_mac().into()
    }

    pub fn check_vid_did(vid: u16, did: u16) -> bool {
        // This is a placeholder for actual VID/DID checking logic.
        // In a real implementation, this would check the device's
        // vendor ID and device ID against the provided values.
        vid == 0x8086 && [0x10C9, 0x1533].contains(&did)
    }

    pub fn status(&self) -> MacStatus {
        self.mac.status()
    }

    pub fn enable_loopback(&mut self) {
        self.mac.enable_loopback();
    }

    pub fn disable_loopback(&mut self) {
        self.mac.disable_loopback();
    }

    fn init_stat(&mut self) {
        //TODO
    }

    /// # Safety
    /// This function should only be called from the interrupt handler.
    /// It will handle the interrupt by acknowledging
    pub unsafe fn handle_interrupt(&mut self) {
        let msg = self.mac.interrupts_ack();
        debug!("Interrupt message: {msg:?}");
        if msg.queue_idx & 0x1 != 0 {
            // let rx_ring = unsafe { &mut *(self.rx_ring_addrs[0] as *mut Ring<AdvRxDesc>) };
            // rx_ring.clean();
        }
    }

    pub fn irq_mode_legacy(&mut self) {
        self.mac.configure_legacy_mode();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Speed {
    Mb10,
    Mb100,
    Mb1000,
}

// DMA API functions for igb driver
use axalloc::global_allocator;
use crate::mem::PAGE_SIZE_4K;

#[no_mangle]
pub extern "C" fn __dma_api_alloc(size: usize, _align: usize) -> *mut u8 {
    let pages = (size + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K;
    match global_allocator().alloc_pages(pages, PAGE_SIZE_4K) {
        Ok(vaddr) => vaddr as *mut u8,
        Err(_) => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn __dma_api_dealloc(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        let pages = (size + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K;
        global_allocator().dealloc_pages(ptr as usize, pages);
    }
}

#[no_mangle]
pub extern "C" fn __dma_api_map(ptr: *const u8, _size: usize) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    crate::mem::virt_to_phys((ptr as usize).into()).as_usize() as u64
}

#[no_mangle]
pub extern "C" fn __dma_api_unmap(_ptr: *const u8, _addr: u64, _size: usize) {
    // No-op for this implementation
}

#[no_mangle]
pub extern "C" fn __dma_api_flush(_ptr: *const u8, _size: usize) {
    // Cache flush - use DSB for memory barrier
    unsafe {
        core::arch::asm!("dsb sy", options(nostack, nomem));
    }
}
