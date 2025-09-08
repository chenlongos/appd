use pcie::{RootComplexGeneric, SimpleBarAllocator};
use crate::mem::phys_to_virt;
use core::ptr::NonNull;

pub fn init_pcie() {
    let pci_32_range = axconfig::PCI_RANGES[0];
    let pci_64_range = axconfig::PCI_RANGES[1];
    let mut bar_alloc = SimpleBarAllocator::new(pci_32_range.0 as u32, (pci_32_range.1 - pci_32_range.0)as u32, pci_64_range.0 as u64, (pci_64_range.1 - pci_64_range.0)as u64);

    let base_vaddr = phys_to_virt(axconfig::PCI_ECAM_BASE.into());

    let base_vaddr =  unsafe {
        NonNull::new_unchecked(base_vaddr.as_mut_ptr())
    };
    info!("Init PCIE @{base_vaddr:?}");

    let mut root = RootComplexGeneric::new(base_vaddr);

    for header in root.enumerate(None, Some(bar_alloc)) {
        info!("header is: {}", header);
    }
}