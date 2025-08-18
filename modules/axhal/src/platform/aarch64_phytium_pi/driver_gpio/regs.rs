use safe_mmio::fields::ReadWrite;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};
use bitflags::bitflags;

#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C, align(4))]
pub struct PhitiumGpio {
    pub data: ReadWrite<GpioPins>,
    pub resv: ReadWrite<u16>,
    pub dir: ReadWrite<GpioPins>,
    pub resv2: ReadWrite<u16>,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct GpioPins(u16);

bitflags! {
    impl GpioPins: u16 {
        const p0 = 1 << 0;
        const p1 = 1 << 1;
        const p2 = 1 << 2;
        const p3 = 1 << 3;
        const p4 = 1 << 4;
        const p5 = 1 << 5;
        const p6 = 1 << 6;
        const p7 = 1 << 7;
        const p8 = 1 << 8;
        const p9 = 1 << 9;
        const p10 = 1 << 10;
        const p11 = 1 << 11;
        const p12 = 1 << 12;
        const p13 = 1 << 13;
        const p14 = 1 << 14;
        const p15 = 1 << 15;
    }
}