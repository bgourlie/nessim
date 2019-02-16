pub const SPRITE_RAM_SIZE: usize = 0x120;
pub const PALETTE_RAM_SIZE: usize = 0x20;
pub const NUM_NODES: usize = 33001;
pub const EMPTYNODE: u16 = 65535;
pub const NODE_GND: u16 = 2;
pub const NODE_PWR: u16 = 1;
pub const NODE_CLK0: u16 = 772;
pub const NODE_RESET: u16 = 1934;
pub const NODE_IO_CE: u16 = 5;
pub const NODE_INT: u16 = 1031;
pub const NODE_ALE: u16 = 1611;
pub const NODE_RD: u16 = 2428;
pub const NODE_WR: u16 = 2087;
pub const NODE_CPU_SO: u16 = 24246;
pub const NODE_CPU_IRQ: u16 = 23488;
pub const NODE_CPU_NMI: u16 = 1031;
pub const NODE_CPU_CLK0: u16 = 24235;
pub const NODE_AB0: u16 = 1991;
pub const NODE_AB1: u16 = 2370;
pub const NODE_AB2: u16 = 2650;
pub const NODE_AB3: u16 = 2776;
pub const NODE_AB4: u16 = 2775;
pub const NODE_AB5: u16 = 2774;
pub const NODE_AB6: u16 = 2773;
pub const NODE_AB7: u16 = 2772;
pub const NODE_AB8: u16 = 2771;
pub const NODE_AB9: u16 = 2770;
pub const NODE_AB10: u16 = 2769;
pub const NODE_AB11: u16 = 2768;
pub const NODE_AB12: u16 = 2767;
pub const NODE_AB13: u16 = 2649;
pub const NODE_CPU_AB0: u16 = 23020;
pub const NODE_CPU_AB1: u16 = 23019;
pub const NODE_CPU_AB2: u16 = 23030;
pub const NODE_CPU_AB3: u16 = 23091;
pub const NODE_CPU_AB4: u16 = 23335;
pub const NODE_CPU_AB5: u16 = 23489;
pub const NODE_CPU_AB6: u16 = 23727;
pub const NODE_CPU_AB7: u16 = 24521;
pub const NODE_CPU_AB8: u16 = 24628;
pub const NODE_CPU_AB9: u16 = 24817;
pub const NODE_CPU_AB10: u16 = 24965;
pub const NODE_CPU_AB11: u16 = 25055;
pub const NODE_CPU_AB12: u16 = 25084;
pub const NODE_CPU_AB13: u16 = 25083;
pub const NODE_CPU_AB14: u16 = 25085;
pub const NODE_CPU_AB15: u16 = 25086;
pub const NODE_CPU_DB0: u16 = 24819;
pub const NODE_CPU_DB1: u16 = 24966;
pub const NODE_CPU_DB2: u16 = 25056;
pub const NODE_CPU_DB3: u16 = 25091;
pub const NODE_CPU_DB4: u16 = 25090;
pub const NODE_CPU_DB5: u16 = 25089;
pub const NODE_CPU_DB6: u16 = 25088;
pub const NODE_CPU_DB7: u16 = 25087;
pub const NODE_DB0: u16 = 1991;
pub const NODE_DB1: u16 = 2370;
pub const NODE_DB2: u16 = 2650;
pub const NODE_DB3: u16 = 2776;
pub const NODE_DB4: u16 = 2775;
pub const NODE_DB5: u16 = 2774;
pub const NODE_DB6: u16 = 2773;
pub const NODE_DB7: u16 = 2772;
pub const NODE_CPU_RW: u16 = 1224;
pub const NODE_PAL_D0_OUT: u16 = 1215;
pub const NODE_PAL_D1_OUT: u16 = 6565;
pub const NODE_PAL_D2_OUT: u16 = 6566;
pub const NODE_PAL_D3_OUT: u16 = 6567;
pub const NODE_PAL_D4_OUT: u16 = 6564;
pub const NODE_PAL_D5_OUT: u16 = 6568;
pub const NODE_PCLK1: u16 = 58;
pub const NODE_HPOS0: u16 = 209;
pub const NODE_HPOS1: u16 = 260;
pub const NODE_HPOS2: u16 = 310;
pub const NODE_HPOS3: u16 = 376;
pub const NODE_HPOS4: u16 = 428;
pub const NODE_HPOS5: u16 = 495;
pub const NODE_HPOS6: u16 = 544;
pub const NODE_HPOS7: u16 = 584;
pub const NODE_HPOS8: u16 = 631;
pub const NODE_VPOS0: u16 = 210;
pub const NODE_VPOS1: u16 = 259;
pub const NODE_VPOS2: u16 = 311;
pub const NODE_VPOS3: u16 = 377;
pub const NODE_VPOS4: u16 = 429;
pub const NODE_VPOS5: u16 = 496;
pub const NODE_VPOS6: u16 = 543;
pub const NODE_VPOS7: u16 = 588;
pub const NODE_VPOS8: u16 = 632;

#[allow(clippy::unreadable_literal)]
pub const PALETTE_ARGB: [u32; 64] = [
    0xFF666666, 0xFF002A88, 0xFF1412A7, 0xFF3B00A4, 0xFF5C007E, 0xFF6E0040, 0xFF6C0600, 0xFF561D00,
    0xFF333500, 0xFF0B4800, 0xFF005200, 0xFF004F08, 0xFF00404D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFADADAD, 0xFF155FD9, 0xFF4240FF, 0xFF7527FE, 0xFFA01ACC, 0xFFB71E7B, 0xFFB53120, 0xFF994E00,
    0xFF6B6D00, 0xFF388700, 0xFF0C9300, 0xFF008F32, 0xFF007C8D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFF64B0FF, 0xFF9290FF, 0xFFC676FF, 0xFFF36AFF, 0xFFFE6ECC, 0xFFFE8170, 0xFFEA9E22,
    0xFFBCBE00, 0xFF88D800, 0xFF5CE430, 0xFF45E082, 0xFF48CDDE, 0xFF4F4F4F, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFFC0DFFF, 0xFFD3D2FF, 0xFFE8C8FF, 0xFFFBC2FF, 0xFFFEC4EA, 0xFFFECCC5, 0xFFF7D8A5,
    0xFFE4E594, 0xFFCFEF96, 0xFFBDF4AB, 0xFFB3F3CC, 0xFFB5EBF2, 0xFFB8B8B8, 0xFF000000, 0xFF000000,
];
