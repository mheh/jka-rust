#![allow(non_camel_case_types, non_snake_case)]

/// Raven `outPacket_t` — bookkeeping for a sent client packet.
///
/// Raven: `p_cmdNumber` = cl.cmdNumber when packet was sent; `p_serverTime` =
/// usercmd->serverTime when packet was sent; `p_realtime` = cls.realtime when
/// packet was sent.
/// Type definition source: `oracle/oracle/codemp/client/client.h:58-62`
#[repr(C)]
pub struct outPacket_t {
    pub p_cmdNumber: i32,
    pub p_serverTime: i32,
    pub p_realtime: i32,
}

const _: () = assert!(core::mem::size_of::<outPacket_t>() == 12);
const _: () = assert!(core::mem::offset_of!(outPacket_t, p_cmdNumber) == 0);
const _: () = assert!(core::mem::offset_of!(outPacket_t, p_serverTime) == 4);
const _: () = assert!(core::mem::offset_of!(outPacket_t, p_realtime) == 8);
