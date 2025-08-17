use tock_registers::{
    interfaces::{Readable, Writeable, ReadWriteable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

register_structs! {
    pub PwmRegisters {
        // Dead-time control registers (0x000~0x3FF)
        (0x0000 => dbctrl: ReadWrite<u32, DBCTRL::Register>),
        (0x0004 => dbdly: ReadWrite<u32, DBDLY::Register>),
        (0x0008 => _reserved_db: [u8; 0x3F8]),
        // PWM0 channel registers (0x400~0x7FF)
        (0x0400 => ch0_tim_cnt: ReadWrite<u32, TIM_CNT::Register>),
        (0x0404 => ch0_tim_ctrl: ReadWrite<u32, TIM_CTRL::Register>),
        (0x0408 => ch0_state: ReadWrite<u32, STATE::Register>),
        (0x040C => ch0_pwm_period: ReadWrite<u32, PWM_PERIOD::Register>),
        (0x0410 => ch0_pwm_ctrl: ReadWrite<u32, PWM_CTRL::Register>),
        (0x0414 => ch0_pwm_ccr: ReadWrite<u32, PWM_CCR::Register>),
        (0x0418 => _reserved_ch0: [u8; 0x3E8]),
        // PWM1 channel registers (0x800~0xBFF)
        (0x0800 => ch1_tim_cnt: ReadWrite<u32, TIM_CNT::Register>),
        (0x0804 => ch1_tim_ctrl: ReadWrite<u32, TIM_CTRL::Register>),
        (0x0808 => ch1_state: ReadWrite<u32, STATE::Register>),
        (0x080C => ch1_pwm_period: ReadWrite<u32, PWM_PERIOD::Register>),
        (0x0810 => ch1_pwm_ctrl: ReadWrite<u32, PWM_CTRL::Register>),
        (0x0814 => ch1_pwm_ccr: ReadWrite<u32, PWM_CCR::Register>),
        (0x0818 => @END),
    }
}

register_bitfields! {
    u32,
    pub DBCTRL [
        OUT_MODE OFFSET(4) NUMBITS(2) [
            Bypass = 0b00,
            FallEdgeOnly = 0b01,
            RiseEdgeOnly = 0b10,
            FullDeadband = 0b11
        ],
        POLSEL OFFSET(2) NUMBITS(2) [
            AH = 0b00,
            ALC = 0b01,
            AHC = 0b10,
            AL = 0b11
        ],
        IN_MODE OFFSET(1) NUMBITS(1) [
            PWM0 = 0,
            PWM1 = 1
        ],
        DB_SW_RST OFFSET(0) NUMBITS(1) [
            Normal = 0,
            ResetActive = 1
        ]
    ],
    pub DBDLY [
        DBFED OFFSET(10) NUMBITS(10) [],
        DBRED OFFSET(0) NUMBITS(10) []
    ],
    pub TIM_CNT [
        CNT OFFSET(0) NUMBITS(16) []
    ],
    pub TIM_CTRL [
        DIV OFFSET(16) NUMBITS(12) [],
        GIE OFFSET(5) NUMBITS(1) [],
        OVFIF_ENABLE OFFSET(4) NUMBITS(1) [],
        MODE OFFSET(2) NUMBITS(1) [
            Modulo = 0,
            UpAndDown = 1
        ],
        ENABLE OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        SW_RST OFFSET(0) NUMBITS(1) [
            Normal = 0,
            ResetActive = 1
        ]
    ],
    pub STATE [
        FIFO_FULL OFFSET(3) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(2) NUMBITS(1) [],
        OVFIF OFFSET(1) NUMBITS(1) [],
        CHIF OFFSET(0) NUMBITS(1) []
    ],
    pub PWM_PERIOD [
        CCR OFFSET(0) NUMBITS(16) []
    ],
    pub PWM_CTRL [
        FIFO_EMPTY_ENABLE OFFSET(9) NUMBITS(1) [],
        DUTY_SEL OFFSET(8) NUMBITS(1) [
            Register = 0,
            FIFO = 1
        ],
        ICOV OFFSET(7) NUMBITS(1) [],
        CMP OFFSET(4) NUMBITS(3) [
            SetOnMatch = 0b000,
            ClearOnMatch = 0b001,
            ToggleOnMatch = 0b010,
            SetOnUpClearOnDown = 0b011,
            ClearOnUpSetOnDown = 0b100,
            ClearOnCCRSetOnPeriod = 0b101,
            SetOnCCRClearOnPeriod = 0b110,
            Initialize = 0b111
        ],
        IE OFFSET(3) NUMBITS(1) [],
        MODE OFFSET(2) NUMBITS(1) [
            FreeRunning = 0,
            Compare = 1
        ]
    ],
    pub PWM_CCR [
        CCR OFFSET(0) NUMBITS(16) []
    ]
}