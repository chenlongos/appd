use log::debug;
use tock_registers::register_bitfields;

use crate::{err::DError, mac::Mac, osal::wait_for};

const PHY_CONTROL: u32 = 0;
const PHY_STATUS: u32 = 1;

register_bitfields! {
    u16,

    /// PHY Control Register (PCTRL) - Register 0x00
    /// This register controls various PHY operations including power management,
    /// speed selection, duplex mode, and auto-negotiation.
    PCTRL [
        /// PHY Reset
        /// 1b = PHY reset
        /// 0b = Normal operation
        /// Note: When using PHY Reset, the PHY default configuration is not loaded from the EEPROM.
        /// The preferred way to reset the 82576 PHY is using the CTRL.PHY_RST field.
        RESET OFFSET(15) NUMBITS(1) [
            Normal = 0,
            Reset = 1
        ],

        /// Loopback
        /// 1b = Enable loopback
        /// 0b = Disable loopback
        LOOPBACK OFFSET(14) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],

        /// Speed Selection (LSB)
        /// Combined with bit 6 (MSB) to determine speed:
        /// 11b = Reserved
        /// 10b = 1000 Mb/s
        /// 01b = 100 Mb/s
        /// 00b = 10 Mb/s
        /// Note: If auto-negotiation is enabled, this bit is ignored.
        SPEED_SELECTION_LSB OFFSET(13) NUMBITS(1) [],

        /// Auto-Negotiation Enable
        /// 1b = Enable Auto-Negotiation Process
        /// 0b = Disable Auto-Negotiation Process
        /// This bit must be enabled for 1000BASE-T operation.
        AUTO_NEGOTIATION_ENABLE OFFSET(12) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],

        /// Power Down
        /// 1b = Power down
        /// 0b = Normal operation
        /// When using this bit, PHY default configuration is lost and is not loaded from
        /// the EEPROM after de-asserting the Power Down bit.
        /// Note: After this bit is set, all indications from PHY including link status are invalid.
        POWER_DOWN OFFSET(11) NUMBITS(1) [
            Normal = 0,
            PowerDown = 1
        ],

        /// Isolate
        /// This bit has no effect on PHY functionality. Program to 0b for future compatibility.
        ISOLATE OFFSET(10) NUMBITS(1) [
            Normal = 0,
            Isolate = 1
        ],

        /// Restart Auto-Negotiation
        /// 1b = Restart Auto-Negotiation Process
        /// 0b = Normal operation
        /// Auto-Negotiation automatically restarts after hardware or software reset
        /// regardless of whether or not the restart bit is set.
        RESTART_AUTO_NEGOTIATION OFFSET(9) NUMBITS(1) [
            Normal = 0,
            Restart = 1
        ],

        /// Duplex Mode
        /// 1b = Full Duplex
        /// 0b = Half Duplex
        /// Note: If auto-negotiation is enabled, this bit is ignored.
        DUPLEX_MODE OFFSET(8) NUMBITS(1) [
            Half = 0,
            Full = 1
        ],

        /// Collision Test
        /// 1b = Enable COL signal test
        /// 0b = Disable COL signal test
        /// Note: This bit is ignored unless loopback is enabled (bit 14 = 1b).
        COLLISION_TEST OFFSET(7) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],

        /// Speed Selection (MSB)
        /// Combined with bit 13 (LSB) to determine speed:
        /// 11b = Reserved
        /// 10b = 1000 Mb/s
        /// 01b = 100 Mb/s
        /// 00b = 10 Mb/s
        /// A write to these bits do not take effect until a software reset is asserted,
        /// Restart Auto-Negotiation is asserted, or Power Down transitions from power down
        /// to normal operation.
        /// Note: If auto-negotiation is enabled, this bit is ignored.
        SPEED_SELECTION_MSB OFFSET(6) NUMBITS(1) [],

        /// Reserved bits 5:0
        /// Always read as 0b. Write to 0b for normal operation
        RESERVED OFFSET(0) NUMBITS(6) []
    ]
}

register_bitfields! {
    u16,

    /// PHY Status Register (PSTATUS) - Register 0x01 (Read Only)
    /// This register provides status information about the PHY capabilities and current state.
    PSTATUS [
        /// 100BASE-T4 Capable
        /// 1b = PHY is able to perform 100BASE-T4
        /// 0b = PHY is not able to perform 100BASE-T4
        CAPABILITY_100BASE_T4 OFFSET(15) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// 100BASE-TX Full Duplex Capable
        /// 1b = PHY is able to perform 100BASE-TX in full duplex mode
        /// 0b = PHY is not able to perform 100BASE-TX in full duplex mode
        CAPABILITY_100BASE_TX_FD OFFSET(14) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// 100BASE-TX Half Duplex Capable
        /// 1b = PHY is able to perform 100BASE-TX in half duplex mode
        /// 0b = PHY is not able to perform 100BASE-TX in half duplex mode
        CAPABILITY_100BASE_TX_HD OFFSET(13) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// 10BASE-T Full Duplex Capable
        /// 1b = PHY is able to perform 10BASE-T in full duplex mode
        /// 0b = PHY is not able to perform 10BASE-T in full duplex mode
        CAPABILITY_10BASE_T_FD OFFSET(12) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// 10BASE-T Half Duplex Capable
        /// 1b = PHY is able to perform 10BASE-T in half duplex mode
        /// 0b = PHY is not able to perform 10BASE-T in half duplex mode
        CAPABILITY_10BASE_T_HD OFFSET(11) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// 100BASE-T2 Full Duplex Capable
        /// 1b = PHY is able to perform 100BASE-T2 in full duplex mode
        /// 0b = PHY is not able to perform 100BASE-T2 in full duplex mode
        CAPABILITY_100BASE_T2_FD OFFSET(10) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// 100BASE-T2 Half Duplex Capable
        /// 1b = PHY is able to perform 100BASE-T2 in half duplex mode
        /// 0b = PHY is not able to perform 100BASE-T2 in half duplex mode
        CAPABILITY_100BASE_T2_HD OFFSET(9) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// Extended Status Information
        /// 1b = Extended status information in Register 15
        /// 0b = No extended status information in Register 15
        EXTENDED_STATUS OFFSET(8) NUMBITS(1) [
            NoExtended = 0,
            Extended = 1
        ],

        /// Reserved bit 7
        RESERVED_7 OFFSET(7) NUMBITS(1) [],

        /// MF Preamble Suppression
        /// 1b = PHY will accept management frames with preamble suppressed
        /// 0b = PHY will not accept management frames with preamble suppressed
        MF_PREAMBLE_SUPPRESSION OFFSET(6) NUMBITS(1) [
            NotAccept = 0,
            Accept = 1
        ],

        /// Auto-Negotiation Complete
        /// 1b = Auto-Negotiation process completed
        /// 0b = Auto-Negotiation process not completed
        AUTO_NEGOTIATION_COMPLETE OFFSET(5) NUMBITS(1) [
            NotComplete = 0,
            Complete = 1
        ],

        /// Remote Fault
        /// 1b = Remote fault condition detected
        /// 0b = No remote fault condition detected
        REMOTE_FAULT OFFSET(4) NUMBITS(1) [
            NoFault = 0,
            Fault = 1
        ],

        /// Auto-Negotiation Ability
        /// 1b = PHY is able to perform Auto-Negotiation
        /// 0b = PHY is not able to perform Auto-Negotiation
        AUTO_NEGOTIATION_ABILITY OFFSET(3) NUMBITS(1) [
            NotCapable = 0,
            Capable = 1
        ],

        /// Link Status
        /// 1b = Valid link established
        /// 0b = Link not established
        /// Note: This is a latching low bit. Once it goes low, it remains low until read.
        LINK_STATUS OFFSET(2) NUMBITS(1) [
            Down = 0,
            Up = 1
        ],

        /// Jabber Detect
        /// 1b = Jabber condition detected
        /// 0b = No jabber condition detected
        JABBER_DETECT OFFSET(1) NUMBITS(1) [
            NoJabber = 0,
            Jabber = 1
        ],

        /// Extended Capability
        /// 1b = Extended register capabilities
        /// 0b = Basic register set capabilities only
        EXTENDED_CAPABILITY OFFSET(0) NUMBITS(1) [
            Basic = 0,
            Extended = 1
        ]
    ]
}

pub struct Phy {
    mac: Mac,
    addr: u32,
}

impl Phy {
    pub fn new(mac: Mac) -> Self {
        Self { mac, addr: 1 }
    }

    pub fn read_mdic(&mut self, offset: u32) -> Result<u16, DError> {
        self.mac.read_mdic(self.addr, offset)
    }

    pub fn write_mdic(&mut self, offset: u32, data: u16) -> Result<(), DError> {
        self.mac.write_mdic(self.addr, offset, data)
    }

    // pub fn aquire_sync(&self, flags: SyncFlags) -> Result<Synced, DError> {
    //     Synced::new(self.mac, flags)
    // }

    pub fn power_up(&mut self) -> Result<(), DError> {
        let mut mii_reg = self.read_mdic(PHY_CONTROL)?;
        mii_reg &= !PCTRL::POWER_DOWN::SET.value;
        self.write_mdic(PHY_CONTROL, mii_reg)
    }

    pub fn read_status(&mut self) -> Result<u16, DError> {
        self.read_mdic(PHY_STATUS)
    }

    pub fn wait_for_auto_negotiation_complete(&mut self) -> Result<(), DError> {
        let interval = core::time::Duration::from_millis(100);
        let try_count = 30; // Wait for up to 3 seconds

        wait_for(
            || self.is_auto_negotiation_complete().unwrap_or(false),
            interval,
            Some(try_count),
        )
    }

    pub fn is_auto_negotiation_complete(&mut self) -> Result<bool, DError> {
        let status = self.read_status()?;
        Ok(status & PSTATUS::AUTO_NEGOTIATION_COMPLETE::Complete.value != 0)
    }

    pub fn enable_auto_negotiation(&mut self) -> Result<(), DError> {
        debug!("Enabling auto-negotiation for PHY at address {}", self.addr);
        let mut control = self.read_mdic(PHY_CONTROL)?;
        control |= PCTRL::AUTO_NEGOTIATION_ENABLE::Enable.value
            | PCTRL::RESTART_AUTO_NEGOTIATION::Restart.value;
        self.write_mdic(PHY_CONTROL, control)
    }
}

// pub struct Synced {
//     mac: Mac,
//     mask: u32,
// }

// impl Synced {
//     pub fn new(mac: Mac, flags: SyncFlags) -> Result<Self, IgbError> {
//         let semaphore = Semaphore::new(mac)?;
//         let mask = mac.software_sync_aquire(flags)?;
//         drop(semaphore);
//         Ok(Self { mac, mask })
//     }
// }

// impl Drop for Synced {
//     fn drop(&mut self) {
//         let semaphore = Semaphore::new(self.mac).unwrap();
//         self.mac.software_sync_release(self.mask);
//         drop(semaphore);
//     }
// }

// pub struct Semaphore {
//     mac: Mac,
// }

// impl Semaphore {
//     pub fn new(mac: Mac) -> Result<Self, DError> {
//         mac.software_semaphore_aquire()?;
//         Ok(Self { mac })
//     }
// }

// impl Drop for Semaphore {
//     fn drop(&mut self) {
//         self.mac.software_semaphore_release();
//     }
// }
