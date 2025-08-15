use core::{fmt::Debug, ptr::NonNull, time::Duration};

use log::error;
use mbarrier::mb;
use tock_registers::{interfaces::*, register_bitfields, register_structs, registers::*};

use crate::{DError, Speed, osal::wait_for};

register_structs! {
    pub MacRegister {
        (0x0 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x4 => _rsv1),
        (0x8 => status: ReadOnly<u32, STATUS::Register>),
        (0xC => _rsv2),
        (0x18 => ctrl_ext: ReadWrite<u32, CTRL_EXT::Register>),
        (0x1c => _rsv3),
        (0x20 => mdic: ReadWrite<u32, MDIC::Register>),
        (0x24 => _rsv4),
        (0xc0 => icr: ReadWrite<u32, ICR::Register>),
        (0xc4 => _rsv13),
        (0xd0 => ims: ReadWrite<u32, IMS::Register>),
        (0xd4 => _rsv14),
        (0xd8 => imc: ReadWrite<u32, IMC::Register>),
        (0xdc => _rsv15),
        (0x100 => pub rctl: ReadWrite<u32, RCTL::Register>),
        (0x104 => _rsv7),
        (0x400 => tctl: ReadWrite<u32, TCTL::Register>),
        (0x404 => _rsv12),
        (0x1514 => gpie: ReadWrite<u32, GPIE::Register>),
        (0x1518 => _rsv16),
        (0x1524 => eims: ReadWrite<u32>),
        (0x1528 => eimc: ReadWrite<u32>),
        (0x152c => eiac: ReadWrite<u32>),
        (0x1530 => eiam: ReadWrite<u32>),
        (0x1534 => _rsv5),
        (0x1580 => eicr: ReadWrite<u32>),
        (0x1584 => _rsv6),
        (0x5400 => ralh_0_15: [ReadWrite<u32>; 32]),
        (0x5480 => _rsv8),
        (0x54e0 => ralh_16_23: [ReadWrite<u32>;32]),
        (0x5560 => _rsv9),
        (0x5B50 => swsm: ReadWrite<u32, SWSM::Register>),
        (0x5B54 => fwsm: ReadWrite<u32>),
        (0x5B58 => _rsv10),
        (0x5B5C => sw_fw_sync: ReadWrite<u32>),
        (0x5B60 => _rsv11),

        // The end of the struct is marked as follows.
        (0xEFFF => @END),
    }
}

register_bitfields! [
    // First parameter is the register width. Can be u8, u16, u32, or u64.
    u32,

    CTRL [
        FD OFFSET(0) NUMBITS(1)[
            HalfDuplex = 0,
            FullDuplex = 1,
        ],
        SLU OFFSET(6) NUMBITS(1)[],
        SPEED OFFSET(8) NUMBITS(2)[
            Speed10 = 0,
            Speed100 = 1,
            Speed1000 = 0b10,
        ],
        FRCSPD OFFSET(11) NUMBITS(1)[],
        FRCDPLX OFFSET(12) NUMBITS(1)[],
        RST OFFSET(26) NUMBITS(1)[
            Normal = 0,
            Reset = 1,
        ],
        PHY_RST OFFSET(31) NUMBITS(1)[],
    ],
    STATUS [
        FD OFFSET(0) NUMBITS(1)[
            HalfDuplex = 0,
            FullDuplex = 1,
        ],
        LU OFFSET(1) NUMBITS(1)[],
        SPEED OFFSET(6) NUMBITS(2)[
            Speed10 = 0,
            Speed100 = 1,
            Speed1000 = 0b10,
        ],
         PHYRA OFFSET(10) NUMBITS(1)[],
    ],
    pub CTRL_EXT [
        LINK_MODE OFFSET(22) NUMBITS(2)[
            DircetCooper = 0,
            SGMII = 0b10,
            InternalSerdes = 0b11,
        ],
    ],
    MDIC [
        DATA OFFSET(0) NUMBITS(16)[],
        REGADDR OFFSET(16) NUMBITS(5)[],
        PHY_ADDR OFFSET(21) NUMBITS(5)[],
        OP OFFSET(26) NUMBITS(2)[
            Write = 0b1,
            Read = 0b10,
        ],
        READY OFFSET(28) NUMBITS(1)[],
        I OFFSET(29) NUMBITS(1)[],
        E OFFSET(30) NUMBITS(1)[
            NoError = 0,
            Error = 1,
        ],
        Destination OFFSET(31) NUMBITS(1)[
            Internal = 0,
            External = 1,
        ]
    ],

    SWSM [
        SMBI OFFSET(0) NUMBITS(1)[],
        SWESMBI OFFSET(1) NUMBITS(1)[],
        WMNG OFFSET(2) NUMBITS(1)[],
        EEUR OFFSET(3) NUMBITS(1)[],
    ],

    SW_FW_SYNC [
        SW_EEP_SM OFFSET(0) NUMBITS(1)[],
        SW_PHY_SM0 OFFSET(1) NUMBITS(1)[],
        SW_PHY_SM1 OFFSET(2) NUMBITS(1)[],
        SW_MAC_CSR_SM OFFSET(3) NUMBITS(1)[],
        SW_FLASH_SM OFFSET(4) NUMBITS(1)[],

        FW_EEP_SM OFFSET(16) NUMBITS(1)[],
        FW_PHY_SM0 OFFSET(17) NUMBITS(1)[],
        FW_PHY_SM1 OFFSET(18) NUMBITS(1)[],
        FW_MAC_CSR_SM OFFSET(19) NUMBITS(1)[],
        FW_FLASH_SM OFFSET(20) NUMBITS(1)[],
    ],

    pub RCTL [
        RXEN OFFSET(1) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        SBP OFFSET(2) NUMBITS(1)[
            DoNotStore = 0,
            Store = 1,
        ],
        UPE OFFSET(3) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        MPE OFFSET(4) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        LPE OFFSET(5) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        LBM OFFSET(6) NUMBITS(2)[
            Normal = 0b00,
            MacLoopback = 0b01,
            Reserved = 0b11,
        ],
        MO OFFSET(12) NUMBITS(2)[
            Bits47_36 = 0b00,
            Bits46_35 = 0b01,
            Bits45_34 = 0b10,
            Bits43_32 = 0b11,
        ],
        BAM OFFSET(15) NUMBITS(1)[
            Ignore = 0,
            Accept = 1,
        ],
        BSIZE OFFSET(16) NUMBITS(2)[
            Bytes2048 = 0b00,
            Bytes1024 = 0b01,
            Bytes512 = 0b10,
            Bytes256 = 0b11,
        ],
        VFE OFFSET(18) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        CFIEN OFFSET(19) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        CFI OFFSET(20) NUMBITS(1)[
            Accept = 0,
            Discard = 1,
        ],
        PSP OFFSET(21) NUMBITS(1)[],
        DPF OFFSET(22) NUMBITS(1)[
            Forward = 0,
            Discard = 1,
        ],
        PMCF OFFSET(23) NUMBITS(1)[
            Pass = 0,
            Filter = 1,
        ],
        SECRC OFFSET(26) NUMBITS(1)[
            DoNotStrip = 0,
            Strip = 1,
        ],
    ],

    // Transmit Control Register - TCTL (0x400)
    TCTL [
        EN OFFSET(1) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        PSP OFFSET(3) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        CT OFFSET(4) NUMBITS(8)[],
        COLD OFFSET(12) NUMBITS(10)[],
        SWXOFF OFFSET(22) NUMBITS(1)[],
        RTLC OFFSET(24) NUMBITS(1)[],
        NRTU OFFSET(25) NUMBITS(1)[],
        MULR OFFSET(28) NUMBITS(1)[],
    ],

    // Extended Interrupt Cause Register - EICR (0x01580)
    EICR [
        // Non MSI-X mode (GPIE.Multiple_MSIX = 0)
        RxTxQ OFFSET(0) NUMBITS(16)[],
        Reserved1 OFFSET(16) NUMBITS(14)[],
        TCP_Timer OFFSET(30) NUMBITS(1)[],
        Other_Cause OFFSET(31) NUMBITS(1)[],
    ],

    // Extended Interrupt Cause Register - EICR MSI-X mode
    EICR_MSIX [
        // MSI-X mode (GPIE.Multiple_MSIX = 1)
        MSIX OFFSET(0) NUMBITS(25)[],
        Reserved OFFSET(25) NUMBITS(7)[],
    ],

    // Extended Interrupt Mask Set/Read - EIMS (0x01524)
    EIMS [
        // Non MSI-X mode (GPIE.Multiple_MSIX = 0)
        RxTxQ OFFSET(0) NUMBITS(16)[],
        Reserved1 OFFSET(16) NUMBITS(14)[],
        TCP_Timer OFFSET(30) NUMBITS(1)[],
        Other_Cause OFFSET(31) NUMBITS(1)[],
    ],

    // Extended Interrupt Mask Set/Read - EIMS MSI-X mode
    EIMS_MSIX [
        // MSI-X mode (GPIE.Multiple_MSIX = 1)
        MSIX OFFSET(0) NUMBITS(25)[],
        Reserved OFFSET(25) NUMBITS(7)[],
    ],

    // Legacy Interrupt Cause Register - ICR (0x000C0)
    ICR [
        TXDW OFFSET(0) NUMBITS(1)[],   // Transmit Descriptor Written Back
        TXQE OFFSET(1) NUMBITS(1)[],   // Transmit Queue Empty
        LSC OFFSET(2) NUMBITS(1)[],    // Link Status Change
        RXSEQ OFFSET(3) NUMBITS(1)[],  // Receive Sequence Error
        RXDMT0 OFFSET(4) NUMBITS(1)[], // Receive Descriptor Minimum Threshold Reached
        RXO OFFSET(6) NUMBITS(1)[],    // Receiver Overrun
        RXT0 OFFSET(7) NUMBITS(1)[],   // Receiver Timer Interrupt
        MDAC OFFSET(9) NUMBITS(1)[],   // MDI/O Access Complete
        RXCFG OFFSET(10) NUMBITS(1)[], // Receiving /C/ ordered sets
        GPI_EN0 OFFSET(11) NUMBITS(1)[], // General Purpose Interrupt Enable 0
        GPI_EN1 OFFSET(12) NUMBITS(1)[], // General Purpose Interrupt Enable 1
        GPI_EN2 OFFSET(13) NUMBITS(1)[], // General Purpose Interrupt Enable 2
        GPI_EN3 OFFSET(14) NUMBITS(1)[], // General Purpose Interrupt Enable 3
        TXD_LOW OFFSET(15) NUMBITS(1)[], // Transmit Descriptor Low Threshold Hit
        SRPD OFFSET(16) NUMBITS(1)[],  // Small Receive Packet Detected
        ACK OFFSET(17) NUMBITS(1)[],   // Receive ACK Frame
        MNG OFFSET(18) NUMBITS(1)[],   // Management Bus Interrupt
        DOCK OFFSET(19) NUMBITS(1)[],  // Dock/Undock Status Change
        INT_ASSERTED OFFSET(31) NUMBITS(1)[], // Interrupt Asserted
    ],

    // Legacy Interrupt Mask Set/Read - IMS (0x000D0)
    IMS [
        TXDW OFFSET(0) NUMBITS(1)[],   // Transmit Descriptor Written Back
        TXQE OFFSET(1) NUMBITS(1)[],   // Transmit Queue Empty
        LSC OFFSET(2) NUMBITS(1)[],    // Link Status Change
        RXSEQ OFFSET(3) NUMBITS(1)[],  // Receive Sequence Error
        RXDMT0 OFFSET(4) NUMBITS(1)[], // Receive Descriptor Minimum Threshold Reached
        RXO OFFSET(6) NUMBITS(1)[],    // Receiver Overrun
        RXT0 OFFSET(7) NUMBITS(1)[],   // Receiver Timer Interrupt
        MDAC OFFSET(9) NUMBITS(1)[],   // MDI/O Access Complete
        RXCFG OFFSET(10) NUMBITS(1)[], // Receiving /C/ ordered sets
        GPI_EN0 OFFSET(11) NUMBITS(1)[], // General Purpose Interrupt Enable 0
        GPI_EN1 OFFSET(12) NUMBITS(1)[], // General Purpose Interrupt Enable 1
        GPI_EN2 OFFSET(13) NUMBITS(1)[], // General Purpose Interrupt Enable 2
        GPI_EN3 OFFSET(14) NUMBITS(1)[], // General Purpose Interrupt Enable 3
        TXD_LOW OFFSET(15) NUMBITS(1)[], // Transmit Descriptor Low Threshold Hit
        SRPD OFFSET(16) NUMBITS(1)[],  // Small Receive Packet Detected
        ACK OFFSET(17) NUMBITS(1)[],   // Receive ACK Frame
        MNG OFFSET(18) NUMBITS(1)[],   // Management Bus Interrupt
        DOCK OFFSET(19) NUMBITS(1)[],  // Dock/Undock Status Change
    ],

    // Legacy Interrupt Mask Clear - IMC (0x000D8)
    IMC [
        TXDW OFFSET(0) NUMBITS(1)[],   // Transmit Descriptor Written Back
        TXQE OFFSET(1) NUMBITS(1)[],   // Transmit Queue Empty
        LSC OFFSET(2) NUMBITS(1)[],    // Link Status Change
        RXSEQ OFFSET(3) NUMBITS(1)[],  // Receive Sequence Error
        RXDMT0 OFFSET(4) NUMBITS(1)[], // Receive Descriptor Minimum Threshold Reached
        RXO OFFSET(6) NUMBITS(1)[],    // Receiver Overrun
        RXT0 OFFSET(7) NUMBITS(1)[],   // Receiver Timer Interrupt
        MDAC OFFSET(9) NUMBITS(1)[],   // MDI/O Access Complete
        RXCFG OFFSET(10) NUMBITS(1)[], // Receiving /C/ ordered sets
        GPI_EN0 OFFSET(11) NUMBITS(1)[], // General Purpose Interrupt Enable 0
        GPI_EN1 OFFSET(12) NUMBITS(1)[], // General Purpose Interrupt Enable 1
        GPI_EN2 OFFSET(13) NUMBITS(1)[], // General Purpose Interrupt Enable 2
        GPI_EN3 OFFSET(14) NUMBITS(1)[], // General Purpose Interrupt Enable 3
        TXD_LOW OFFSET(15) NUMBITS(1)[], // Transmit Descriptor Low Threshold Hit
        SRPD OFFSET(16) NUMBITS(1)[],  // Small Receive Packet Detected
        ACK OFFSET(17) NUMBITS(1)[],   // Receive ACK Frame
        MNG OFFSET(18) NUMBITS(1)[],   // Management Bus Interrupt
        DOCK OFFSET(19) NUMBITS(1)[],  // Dock/Undock Status Change
    ],

    // General Purpose Interrupt Enable - GPIE (0x1514)
    GPIE [
        NSICR OFFSET(0) NUMBITS(1)[
            Normal = 0,
            ClearOnRead = 1,
        ],
        Multiple_MSIX OFFSET(4) NUMBITS(1)[
            SingleVector = 0,
            MultipleVectors = 1,
        ],
        LL_Interval OFFSET(7) NUMBITS(5)[],  // Low Latency Credits Increment Rate (bits 11:7)
        EIAME OFFSET(30) NUMBITS(1)[
            Disabled = 0,
            Enabled = 1,
        ],
        PBA_Support OFFSET(31) NUMBITS(1)[
            Legacy = 0,
            MSIX = 1,
        ],
    ],
];

#[derive(Clone, Copy)]
pub struct Mac {
    reg: NonNull<MacRegister>,
}

impl Mac {
    pub fn new(iobase: NonNull<u8>) -> Self {
        Self { reg: iobase.cast() }
    }

    pub fn iobase<T>(&self) -> NonNull<T> {
        self.reg.cast()
    }

    pub fn write_mdic(&self, phys_addr: u32, offset: u32, data: u16) -> Result<(), DError> {
        self.reg().mdic.write(
            MDIC::REGADDR.val(offset)
                + MDIC::PHY_ADDR.val(phys_addr)
                + MDIC::DATA.val(data as _)
                + MDIC::OP::Write,
        );
        mb();

        loop {
            let mdic = self.reg().mdic.extract();

            if mdic.is_set(MDIC::READY) {
                break;
            }
            if mdic.is_set(MDIC::E) {
                error!("MDIC read error");
                return Err(DError::Unknown("MDIC read error"));
            }
        }

        Ok(())
    }

    pub fn read_mdic(&self, phys_addr: u32, offset: u32) -> Result<u16, DError> {
        self.reg()
            .mdic
            .write(MDIC::REGADDR.val(offset) + MDIC::PHY_ADDR.val(phys_addr) + MDIC::OP::Read);
        mb();
        loop {
            let mdic = self.reg().mdic.extract();
            if mdic.is_set(MDIC::READY) {
                return Ok(mdic.read(MDIC::DATA) as _);
            }
            if mdic.is_set(MDIC::E) {
                error!("MDIC read error");
                return Err(DError::Unknown("MDIC read error"));
            }
        }
    }

    pub fn disable_interrupts(&mut self) {
        self.reg_mut().eimc.set(u32::MAX);
        self.clear_interrupts();
    }

    pub fn enable_interrupts(&mut self) {
        self.reg_mut().eims.set(u32::MAX);
    }

    /// Enable legacy interrupts
    pub fn enable_legacy_interrupts(&mut self) {
        // Enable common legacy interrupts
        self.reg_mut().ims.write(
            IMS::TXDW::SET
                + IMS::TXQE::SET
                + IMS::LSC::SET
                + IMS::RXDMT0::SET
                + IMS::RXO::SET
                + IMS::RXT0::SET
                + IMS::MDAC::SET,
        );
    }

    /// Disable legacy interrupts
    pub fn disable_legacy_interrupts(&mut self) {
        // Disable all legacy interrupts
        self.reg_mut().imc.write(
            IMC::TXDW::SET
                + IMC::TXQE::SET
                + IMC::LSC::SET
                + IMC::RXSEQ::SET
                + IMC::RXDMT0::SET
                + IMC::RXO::SET
                + IMC::RXT0::SET
                + IMC::MDAC::SET
                + IMC::RXCFG::SET
                + IMC::GPI_EN0::SET
                + IMC::GPI_EN1::SET
                + IMC::GPI_EN2::SET
                + IMC::GPI_EN3::SET
                + IMC::TXD_LOW::SET
                + IMC::SRPD::SET
                + IMC::ACK::SET
                + IMC::MNG::SET
                + IMC::DOCK::SET,
        );
    }

    /// Read and clear legacy interrupt cause
    pub fn legacy_interrupts_ack(&mut self) -> LegacyIrqMsg {
        let icr = self.reg().icr.get();
        let ims = self.reg().ims.get();
        let status = icr & ims;

        LegacyIrqMsg {
            txdw: status & ICR::TXDW.mask != 0,
            txqe: status & ICR::TXQE.mask != 0,
            lsc: status & ICR::LSC.mask != 0,
            rxseq: status & ICR::RXSEQ.mask != 0,
            rxdmt0: status & ICR::RXDMT0.mask != 0,
            rxo: status & ICR::RXO.mask != 0,
            rxt0: status & ICR::RXT0.mask != 0,
            mdac: status & ICR::MDAC.mask != 0,
            rxcfg: status & ICR::RXCFG.mask != 0,
            asserted: status & ICR::INT_ASSERTED.mask != 0,
        }
    }

    pub fn interrupts_ack(&mut self) -> IrqMsg {
        let eicr = self.reg().eicr.get();
        let eims = self.reg().eims.get();
        let status = eicr & eims;
        let tcp_timer = status & EICR::TCP_Timer.mask != 0;
        let other = status & EICR::Other_Cause.mask != 0;
        let queue_idx = (status & EICR::RxTxQ.mask) as u16;
        IrqMsg {
            queue_idx,
            tcp_timer,
            other,
        }
    }

    pub fn link_mode(&self) -> Option<LinkMode> {
        Some(
            match self.reg().ctrl_ext.read_as_enum(CTRL_EXT::LINK_MODE) {
                Some(CTRL_EXT::LINK_MODE::Value::DircetCooper) => LinkMode::DirectCooper,
                Some(CTRL_EXT::LINK_MODE::Value::SGMII) => LinkMode::Sgmii,
                Some(CTRL_EXT::LINK_MODE::Value::InternalSerdes) => LinkMode::InternalSerdes,
                None => return None,
            },
        )
    }

    /// Clear all interrupt masks for all queues.
    pub fn clear_interrupts(&mut self) {
        // Clear interrupt mask
        self.reg_mut().eimc.set(u32::MAX);
        self.reg().eicr.get();
    }

    pub fn reset(&mut self) -> Result<(), DError> {
        self.reg_mut()
            .ctrl
            .modify(CTRL::RST::Reset + CTRL::PHY_RST::SET);
        wait_for(
            || self.reg().ctrl.matches_any(&[CTRL::RST::Normal]),
            Duration::from_millis(1),
            Some(1000),
        )
    }

    pub fn set_link_up(&mut self) {
        self.reg_mut().ctrl.modify(CTRL::SLU::SET + CTRL::FD::SET);
    }

    pub fn reg(&self) -> &MacRegister {
        unsafe { self.reg.as_ref() }
    }
    pub fn reg_mut(&mut self) -> &mut MacRegister {
        unsafe { self.reg.as_mut() }
    }

    pub fn read_mac(&self) -> [u8; 6] {
        let low = self.ral(0);
        let high = self.rah(0);

        [
            (low & 0xff) as u8,
            ((low >> 8) & 0xff) as u8,
            ((low >> 16) & 0xff) as u8,
            (low >> 24) as u8,
            (high & 0xff) as u8,
            ((high >> 8) & 0xff) as u8,
        ]
    }

    pub fn disable_rx(&mut self) {
        self.reg_mut().rctl.modify(RCTL::RXEN::Disabled);
    }

    pub fn enable_rx(&mut self) {
        self.reg_mut().rctl.modify(RCTL::RXEN::Enabled);
    }

    pub fn enable_tx(&mut self) {
        self.reg_mut().tctl.modify(TCTL::EN::Enabled);
    }

    pub fn enable_loopback(&mut self) {
        self.reg_mut().rctl.modify(RCTL::LBM::MacLoopback);
    }

    pub fn disable_loopback(&mut self) {
        self.reg_mut().rctl.modify(RCTL::LBM::Normal);
    }

    /// Configure GPIE register for MSI-X mode
    pub fn configure_msix_mode(&mut self) {
        self.reg_mut().gpie.write(
            GPIE::Multiple_MSIX::MultipleVectors + GPIE::EIAME::Enabled + GPIE::PBA_Support::MSIX,
        );
    }

    /// Configure GPIE register for legacy/MSI mode
    pub fn configure_legacy_mode(&mut self) {
        self.reg_mut().gpie.write(
            GPIE::Multiple_MSIX::SingleVector + GPIE::EIAME::Disabled + GPIE::PBA_Support::Legacy,
        );
    }

    /// Set non-selective interrupt clear on read
    pub fn set_nsicr(&mut self, enable: bool) {
        if enable {
            self.reg_mut().gpie.modify(GPIE::NSICR::ClearOnRead);
        } else {
            self.reg_mut().gpie.modify(GPIE::NSICR::Normal);
        }
    }

    /// Set Low Latency Credits Increment Rate
    pub fn set_ll_interval(&mut self, interval: u8) {
        // interval is in 4μs increments, valid values 0-31
        let val = (interval as u32) & 0x1F;
        self.reg_mut().gpie.modify(GPIE::LL_Interval.val(val));
    }

    fn ral(&self, i: usize) -> u32 {
        if i <= 15 {
            self.reg().ralh_0_15[i * 2].get()
        } else {
            self.reg().ralh_16_23[i * 2].get()
        }
    }

    fn rah(&self, i: usize) -> u32 {
        if i <= 15 {
            self.reg().ralh_0_15[i * 2 + 1].get()
        } else {
            self.reg().ralh_16_23[i * 2 + 1].get()
        }
    }

    pub fn status(&self) -> MacStatus {
        let status = self.reg().status.extract();
        let speed = match status.read_as_enum(STATUS::SPEED) {
            Some(STATUS::SPEED::Value::Speed1000) => Speed::Mb1000,
            Some(STATUS::SPEED::Value::Speed100) => Speed::Mb100,
            _ => Speed::Mb10,
        };
        let full_duplex = status.is_set(STATUS::FD);
        let link_up = status.is_set(STATUS::LU);
        let phy_reset_asserted = status.is_set(STATUS::PHYRA);

        MacStatus {
            full_duplex,
            link_up,
            speed,
            phy_reset_asserted,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IrqMsg {
    pub queue_idx: u16,
    pub tcp_timer: bool,
    pub other: bool,
}

#[derive(Debug, Clone)]
pub struct LegacyIrqMsg {
    pub txdw: bool,     // Transmit Descriptor Written Back
    pub txqe: bool,     // Transmit Queue Empty
    pub lsc: bool,      // Link Status Change
    pub rxseq: bool,    // Receive Sequence Error
    pub rxdmt0: bool,   // Receive Descriptor Minimum Threshold Reached
    pub rxo: bool,      // Receiver Overrun
    pub rxt0: bool,     // Receiver Timer Interrupt
    pub mdac: bool,     // MDI/O Access Complete
    pub rxcfg: bool,    // Receiving /C/ ordered sets
    pub asserted: bool, // Interrupt Asserted
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MacAddr6([u8; 6]);

impl MacAddr6 {
    pub fn new(bytes: [u8; 6]) -> Self {
        MacAddr6(bytes)
    }

    pub fn bytes(&self) -> [u8; 6] {
        self.0
    }
}

impl Debug for MacAddr6 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl From<[u8; 6]> for MacAddr6 {
    fn from(addr: [u8; 6]) -> Self {
        MacAddr6(addr)
    }
}

impl From<MacAddr6> for [u8; 6] {
    fn from(addr: MacAddr6) -> Self {
        addr.0
    }
}

#[derive(Debug, Clone)]
pub struct MacStatus {
    pub full_duplex: bool,
    pub link_up: bool,
    pub speed: Speed,
    pub phy_reset_asserted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    DirectCooper,
    Sgmii,
    InternalSerdes,
}
