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


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
