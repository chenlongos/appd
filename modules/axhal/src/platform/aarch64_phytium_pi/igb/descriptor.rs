#![allow(dead_code)]

use tock_registers::register_bitfields;

pub trait Descriptor {}

register_bitfields! [
    u64,

    // Advanced Receive Descriptor Read Format
    RX_DESC_READ_PKT_ADDR [
        ADDR OFFSET(1) NUMBITS(63)[],  // Address bits [63:1]
        NSE OFFSET(0) NUMBITS(1)[],    // No-Snoop Enable bit [0]
    ],

    RX_DESC_READ_HDR_ADDR [
        ADDR OFFSET(1) NUMBITS(63)[],  // Address bits [63:1]
        DD OFFSET(0) NUMBITS(1)[],     // Descriptor Done bit [0]
    ],
];

register_bitfields! [
    u32,

    // Advanced Receive Descriptor Write-back Format Low DWORD
    RX_DESC_WB_LO_RSS_HASH [
        RSS_HASH OFFSET(0) NUMBITS(32)[],  // RSS Hash [31:0]
    ],

    RX_DESC_WB_LO_FRAG_CSUM [
        FRAG_CSUM OFFSET(16) NUMBITS(16)[],  // Fragment Checksum [31:16]
        IP_ID OFFSET(0) NUMBITS(16)[],       // IP identification [15:0]
    ],

    RX_DESC_WB_LO_HDR_STATUS [
        HDR_LEN OFFSET(22) NUMBITS(10)[],    // Header Length [31:22]
        SPH OFFSET(21) NUMBITS(1)[],         // Split Header [21]
        EXT_STATUS_LO OFFSET(0) NUMBITS(21)[], // Extended Status [20:0]
    ],

    // Advanced Receive Descriptor Write-back Format High DWORD
    RX_DESC_WB_HI_VLAN_LEN [
        VLAN_TAG OFFSET(16) NUMBITS(16)[],   // VLAN Tag [31:16]
        PKT_LEN OFFSET(0) NUMBITS(16)[],     // Packet Length [15:0]
    ],

    RX_DESC_WB_HI_ERROR_STATUS [
        EXT_ERROR OFFSET(20) NUMBITS(12)[],  // Extended Error [31:20]
        RSS_TYPE OFFSET(17) NUMBITS(3)[],    // RSS Type [19:17]
        PKT_TYPE OFFSET(4) NUMBITS(13)[],    // Packet Type [16:4]
        EXT_STATUS_HI OFFSET(0) NUMBITS(4)[], // Extended Status [3:0]
    ],

    // Extended Status bits
    RX_DESC_EXT_STATUS [
        DD OFFSET(0) NUMBITS(1)[],      // Descriptor Done
        EOP OFFSET(1) NUMBITS(1)[],     // End of Packet
        VP OFFSET(3) NUMBITS(1)[],      // VLAN Packet
        UDPCS OFFSET(4) NUMBITS(1)[],   // UDP Checksum
        L4I OFFSET(5) NUMBITS(1)[],     // L4 Integrity check
        IPCS OFFSET(6) NUMBITS(1)[],    // IP Checksum
        PIF OFFSET(7) NUMBITS(1)[],     // Passed In-exact Filter
        VEXT OFFSET(9) NUMBITS(1)[],    // First VLAN on double VLAN
        UDPV OFFSET(10) NUMBITS(1)[],   // UDP Valid
        LLINT OFFSET(11) NUMBITS(1)[],  // Low Latency Interrupt
        TS OFFSET(16) NUMBITS(1)[],     // Time Stamped
        SECP OFFSET(17) NUMBITS(1)[],   // Security Processing
        LB OFFSET(18) NUMBITS(1)[],     // Loopback
    ],

    // Extended Error bits
    RX_DESC_EXT_ERROR [
        HBO OFFSET(3) NUMBITS(1)[],     // Header Buffer Overflow
        SECERR OFFSET(7) NUMBITS(2)[],  // Security Error [8:7]
        L4E OFFSET(9) NUMBITS(1)[],     // L4 Error
        IPE OFFSET(10) NUMBITS(1)[],    // IP Error
        RXE OFFSET(11) NUMBITS(1)[],    // RX Error
    ],

    // Advanced Transmit Descriptor CMD_TYPE_LEN field
    pub TX_DESC_CMD_TYPE_LEN [
        LEN OFFSET(0) NUMBITS(20)[],        // Packet Length [19:0]
        DTYPE OFFSET(20) NUMBITS(4)[
            Data = 0b11,                    // Data descriptor
            Context = 0b10,                 // Context descriptor
        ],
        CMD_EOP OFFSET(24) NUMBITS(1)[],    // End of Packet
        CMD_IFCS OFFSET(25) NUMBITS(1)[],   // Insert FCS
        CMD_IC OFFSET(26) NUMBITS(1)[],     // Insert Checksum
        CMD_RS OFFSET(27) NUMBITS(1)[],     // Report Status
        CMD_DEXT OFFSET(29) NUMBITS(1)[],   // Descriptor Extension
        CMD_VLE OFFSET(30) NUMBITS(1)[],    // VLAN Packet Enable
        CMD_IDE OFFSET(31) NUMBITS(1)[],    // Interrupt Delay Enable
    ],

    // Advanced Transmit Descriptor Status field (write-back format)
    pub TX_DESC_STATUS [
        DD OFFSET(0) NUMBITS(1)[],          // Descriptor Done
    ],
];

#[derive(Debug, Clone, Copy)]
pub enum TxAdvDescType {
    Data,
    Context,
}

#[allow(clippy::upper_case_acronyms)]
pub enum TxAdvDescCmd {
    EOP,
    IFCS,
    IC,
    RS,
    DEXT,
    VLE,
    IDE,
}

/// RSS类型枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RssType {
    None = 0x0,
    HashTcpIpv4 = 0x1,
    HashIpv4 = 0x2,
    HashTcpIpv6 = 0x3,
    HashIpv6Ex = 0x4,
    HashIpv6 = 0x5,
    HashTcpIpv6Ex = 0x6,
    HashUdpIpv4 = 0x7,
    HashUdpIpv6 = 0x8,
    HashUdpIpv6Ex = 0x9,
    Reserved(u8),
}

impl From<u8> for RssType {
    fn from(val: u8) -> Self {
        match val {
            0x0 => RssType::None,
            0x1 => RssType::HashTcpIpv4,
            0x2 => RssType::HashIpv4,
            0x3 => RssType::HashTcpIpv6,
            0x4 => RssType::HashIpv6Ex,
            0x5 => RssType::HashIpv6,
            0x6 => RssType::HashTcpIpv6Ex,
            0x7 => RssType::HashUdpIpv4,
            0x8 => RssType::HashUdpIpv6,
            0x9 => RssType::HashUdpIpv6Ex,
            _ => RssType::Reserved(val),
        }
    }
}

/// 安全错误类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityError {
    None = 0b00,
    NoSaMatch = 0b01,
    ReplayError = 0b10,
    BadSignature = 0b11,
}

impl From<u8> for SecurityError {
    fn from(val: u8) -> Self {
        match val & 0b11 {
            0b00 => SecurityError::None,
            0b01 => SecurityError::NoSaMatch,
            0b10 => SecurityError::ReplayError,
            0b11 => SecurityError::BadSignature,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub union AdvTxDesc {
    pub read: AdvTxDescRead,
    pub write: AdvTxDescWB,
}

impl Descriptor for AdvTxDesc {}

impl AdvTxDesc {
    /// 创建新的发送描述符
    pub fn new(
        buffer_addr: u64,
        buffer_len: usize,
        kind: TxAdvDescType,
        cmd_ls: &[TxAdvDescCmd],
    ) -> Self {
        let mut cmd_type_len = TX_DESC_CMD_TYPE_LEN::LEN.val(buffer_len as _);
        match kind {
            TxAdvDescType::Data => {
                cmd_type_len += TX_DESC_CMD_TYPE_LEN::DTYPE::Data;
            }
            TxAdvDescType::Context => {
                cmd_type_len += TX_DESC_CMD_TYPE_LEN::DTYPE::Context;
            }
        }

        for c in cmd_ls {
            match c {
                TxAdvDescCmd::EOP => cmd_type_len += TX_DESC_CMD_TYPE_LEN::CMD_EOP::SET,
                TxAdvDescCmd::IFCS => cmd_type_len += TX_DESC_CMD_TYPE_LEN::CMD_IFCS::SET,
                TxAdvDescCmd::IC => cmd_type_len += TX_DESC_CMD_TYPE_LEN::CMD_IC::SET,
                TxAdvDescCmd::RS => cmd_type_len += TX_DESC_CMD_TYPE_LEN::CMD_RS::SET,
                TxAdvDescCmd::DEXT => cmd_type_len += TX_DESC_CMD_TYPE_LEN::CMD_DEXT::SET,
                TxAdvDescCmd::VLE => cmd_type_len += TX_DESC_CMD_TYPE_LEN::CMD_VLE::SET,
                TxAdvDescCmd::IDE => cmd_type_len += TX_DESC_CMD_TYPE_LEN::CMD_IDE::SET,
            }
        }

        Self {
            read: AdvTxDescRead {
                buffer_addr,
                cmd_type_len: cmd_type_len.value,
                olinfo_status: 0,
            },
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescRead {
    pub buffer_addr: u64,
    pub cmd_type_len: u32,
    pub olinfo_status: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescWB {
    pub rsvd: u64,
    pub nxtseq_seed: u32,
    pub status: u32,
}

impl AdvTxDescWB {
    /// 检查描述符是否已完成 (DD bit)
    pub fn is_done(&self) -> bool {
        self.status & TX_DESC_STATUS::DD.mask != 0
    }
}

/// Advanced Receive Descriptor (82576EB)
///
/// 这个联合体表示Intel 82576EB控制器的高级接收描述符，
/// 支持软件写入格式（Read Format）和硬件写回格式（Write-back Format）
///
/// 根据Intel 82576EB GbE Controller文档第7.1.5节：
/// - Read Format: 软件写入描述符队列，硬件从主机内存读取
/// - Write-back Format: 硬件写回描述符到主机内存
#[derive(Clone, Copy)]
pub union AdvRxDesc {
    /// 软件写入格式 - 提供包和头部缓冲区地址
    pub read: AdvRxDescRead,
    /// 硬件写回格式 - 包含接收状态、长度、校验和等信息
    pub write: AdvRxDescWB,
}

impl Descriptor for AdvRxDesc {}

/// Advanced Receive Descriptor Read Format (软件写入格式)
///
/// 根据Intel 82576EB文档Table 7-10，这是软件写入描述符队列的格式：
///
/// ```text
/// 63                                                                0
/// +------------------------------------------------------------------+
/// |              Packet Buffer Address [63:1]                | A0/NSE|
/// +------------------------------------------------------------------+
/// |              Header Buffer Address [63:1]                |  DD   |
/// +------------------------------------------------------------------+
/// ```
///
/// - Packet Buffer Address: 包缓冲区的物理地址
/// - A0/NSE: 最低位是地址位A0或No-Snoop Enable位
/// - Header Buffer Address: 头部缓冲区的物理地址  
/// - DD: Descriptor Done位，硬件写回时设置
#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescRead {
    /// Packet Buffer Address [63:1] + A0/NSE [0]
    /// 物理地址的最低位是 A0 (地址的LSB) 或 NSE (No-Snoop Enable)
    pub pkt_addr: u64,
    /// Header Buffer Address [63:1] + DD [0]  
    /// 头部缓冲区物理地址，最低位是 DD (Descriptor Done)
    pub hdr_addr: u64,
}
/// Advanced Receive Descriptor Write-back Format (硬件写回格式)
///
/// 根据Intel 82576EB文档Table 7-11，这是硬件写回到主机内存的格式：
///
/// ```text
/// 127                                                               64
/// +------------------------------------------------------------------+
/// |    VLAN Tag   |   PKT_LEN   |Ext Err|RSS|Pkt Type|Ext Status    |
/// +------------------------------------------------------------------+
/// 63            48 47        32 31    21 20 19    17 16         0
/// +------------------------------------------------------------------+
/// |RSS Hash/Frag  |   IP ID     |HDR_LEN|S|  RSV   |Ext Status     |
/// |   Checksum     |             |       |P|        |               |
/// +------------------------------------------------------------------+
/// ```
///
/// 字段说明：
/// - RSS Hash/Fragment Checksum: RSS哈希值或片段校验和
/// - IP ID: IP标识符（用于片段重组）
/// - HDR_LEN: 头部长度
/// - SPH: Split Header位
/// - PKT_LEN: 包长度
/// - VLAN Tag: VLAN标签
/// - Ext Status: 扩展状态（包含DD、EOP等位）
/// - Ext Error: 扩展错误信息
/// - RSS Type: RSS类型
/// - Pkt Type: 包类型
#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescWB {
    /// Lower 64 bits of write-back descriptor
    /// 63:48 - RSS Hash Value/Fragment Checksum
    /// 47:32 - IP identification
    /// 31:21 - HDR_LEN (Header Length)
    /// 20 - SPH (Split Header)
    /// 19:0 - Extended Status
    pub lo_dword: LoDword,
    /// Upper 64 bits of write-back descriptor  
    /// 63:48 - VLAN Tag
    /// 47:32 - PKT_LEN (Packet Length)
    /// 31:20 - Extended Error
    /// 19:17 - RSS Type
    /// 16:4 - Packet Type
    /// 3:0 - Extended Status
    pub hi_dword: HiDword,
}

#[derive(Clone, Copy)]
pub union LoDword {
    /// 完整的64位数据
    pub data: u64,
    /// RSS Hash / Fragment Checksum + IP identification + HDR_LEN + SPH + Extended Status
    pub fields: LoFields,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LoFields {
    /// 31:0 - RSS Hash Value (32-bit) 或 Fragment Checksum (16-bit, 31:16) + IP identification (16-bit, 15:0)
    pub rss_hash_or_csum_ip: u32,
    /// 63:32 - HDR_LEN (31:22) + SPH (21) + RSV (20:17) + Extended Status (16:0)
    pub hdr_status: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union HiDword {
    /// 完整的64位数据
    pub data: u64,
    /// 分解的字段
    pub fields: HiFields,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct HiFields {
    /// 31:0 - Extended Error (31:20) + RSS Type (19:17) + Packet Type (16:4) + Extended Status (3:0)
    pub error_type_status: u32,
    /// 63:32 - VLAN Tag (31:16) + PKT_LEN (15:0)
    pub vlan_length: u32,
}

impl AdvRxDescRead {
    /// 创建新的接收描述符
    pub fn new(pkt_addr: u64, hdr_addr: u64, nse: bool) -> Self {
        let pkt_addr = if nse {
            pkt_addr | RX_DESC_READ_PKT_ADDR::NSE.mask
        } else {
            pkt_addr & !RX_DESC_READ_PKT_ADDR::NSE.mask
        };

        Self {
            pkt_addr,
            hdr_addr: hdr_addr & !RX_DESC_READ_HDR_ADDR::DD.mask, // 确保DD位为0
        }
    }
}

impl AdvRxDescWB {
    /// 检查描述符是否已完成 (DD bit)
    pub fn is_done(&self) -> bool {
        unsafe { self.hi_dword.fields.error_type_status & RX_DESC_EXT_STATUS::DD.mask != 0 }
    }

    /// 检查是否为包的最后一个描述符 (EOP bit)
    pub fn is_end_of_packet(&self) -> bool {
        unsafe { self.hi_dword.fields.error_type_status & RX_DESC_EXT_STATUS::EOP.mask != 0 }
    }

    /// 获取包长度
    pub fn packet_length(&self) -> u16 {
        unsafe { (self.hi_dword.fields.vlan_length & RX_DESC_WB_HI_VLAN_LEN::PKT_LEN.mask) as u16 }
    }

    /// 获取VLAN标签
    pub fn vlan_tag(&self) -> u16 {
        unsafe {
            ((self.hi_dword.fields.vlan_length & RX_DESC_WB_HI_VLAN_LEN::VLAN_TAG.mask)
                >> RX_DESC_WB_HI_VLAN_LEN::VLAN_TAG.shift) as u16
        }
    }

    /// 获取RSS哈希值
    pub fn rss_hash(&self) -> u32 {
        unsafe { self.lo_dword.fields.rss_hash_or_csum_ip }
    }

    /// 获取头部长度
    pub fn header_length(&self) -> u16 {
        unsafe {
            ((self.lo_dword.fields.hdr_status & RX_DESC_WB_LO_HDR_STATUS::HDR_LEN.mask)
                >> RX_DESC_WB_LO_HDR_STATUS::HDR_LEN.shift) as u16
        }
    }

    /// 检查是否分割头部 (SPH bit)
    pub fn is_split_header(&self) -> bool {
        unsafe { self.lo_dword.fields.hdr_status & RX_DESC_WB_LO_HDR_STATUS::SPH.mask != 0 }
    }

    /// 获取包类型
    pub fn packet_type(&self) -> u16 {
        unsafe {
            ((self.hi_dword.fields.error_type_status & RX_DESC_WB_HI_ERROR_STATUS::PKT_TYPE.mask)
                >> RX_DESC_WB_HI_ERROR_STATUS::PKT_TYPE.shift) as u16
        }
    }

    /// 获取RSS类型
    pub fn rss_type(&self) -> u8 {
        unsafe {
            ((self.hi_dword.fields.error_type_status & RX_DESC_WB_HI_ERROR_STATUS::RSS_TYPE.mask)
                >> RX_DESC_WB_HI_ERROR_STATUS::RSS_TYPE.shift) as u8
        }
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        unsafe {
            (self.hi_dword.fields.error_type_status & RX_DESC_WB_HI_ERROR_STATUS::EXT_ERROR.mask)
                != 0
                || (self.hi_dword.fields.error_type_status
                    & (RX_DESC_EXT_ERROR::L4E.mask
                        | RX_DESC_EXT_ERROR::IPE.mask
                        | RX_DESC_EXT_ERROR::RXE.mask))
                    != 0
        }
    }

    /// 检查IP校验和是否有效
    pub fn ip_checksum_valid(&self) -> bool {
        unsafe {
            (self.hi_dword.fields.error_type_status & RX_DESC_EXT_STATUS::IPCS.mask) != 0
                && (self.hi_dword.fields.error_type_status & RX_DESC_EXT_ERROR::IPE.mask) == 0
        }
    }

    /// 检查L4校验和是否有效
    pub fn l4_checksum_valid(&self) -> bool {
        unsafe {
            (self.hi_dword.fields.error_type_status & RX_DESC_EXT_STATUS::L4I.mask) != 0
                && (self.hi_dword.fields.error_type_status & RX_DESC_EXT_ERROR::L4E.mask) == 0
        }
    }

    /// 获取RSS类型枚举
    pub fn rss_type_enum(&self) -> RssType {
        RssType::from(self.rss_type())
    }

    /// 获取安全错误类型
    pub fn security_error(&self) -> SecurityError {
        unsafe {
            let error_bits = (self.hi_dword.fields.error_type_status
                & RX_DESC_EXT_ERROR::SECERR.mask)
                >> RX_DESC_EXT_ERROR::SECERR.shift;
            SecurityError::from(error_bits as u8)
        }
    }

    /// 检查是否有头部缓冲区溢出
    pub fn has_header_buffer_overflow(&self) -> bool {
        unsafe { (self.hi_dword.fields.error_type_status & RX_DESC_EXT_ERROR::HBO.mask) != 0 }
    }

    /// 检查是否为VLAN包
    pub fn is_vlan_packet(&self) -> bool {
        unsafe { (self.hi_dword.fields.error_type_status & RX_DESC_EXT_STATUS::VP.mask) != 0 }
    }

    /// 检查是否为回环包
    pub fn is_loopback_packet(&self) -> bool {
        unsafe { (self.hi_dword.fields.error_type_status & RX_DESC_EXT_STATUS::LB.mask) != 0 }
    }

    /// 检查是否为时间戳包
    pub fn is_timestamped(&self) -> bool {
        unsafe { (self.hi_dword.fields.error_type_status & RX_DESC_EXT_STATUS::TS.mask) != 0 }
    }

    /// 获取片段校验和（当不使用RSS时）
    pub fn fragment_checksum(&self) -> u16 {
        unsafe {
            ((self.lo_dword.fields.rss_hash_or_csum_ip & RX_DESC_WB_LO_FRAG_CSUM::FRAG_CSUM.mask)
                >> RX_DESC_WB_LO_FRAG_CSUM::FRAG_CSUM.shift) as u16
        }
    }

    /// 获取IP标识符（当不使用RSS时）
    pub fn ip_identification(&self) -> u16 {
        unsafe {
            (self.lo_dword.fields.rss_hash_or_csum_ip & RX_DESC_WB_LO_FRAG_CSUM::IP_ID.mask) as u16
        }
    }
}
