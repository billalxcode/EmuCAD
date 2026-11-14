use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Gate {
    Const(bool, NetId),
    Buf(NetId, NetId),
    Not(NetId, NetId),
    And(NetId, NetId, NetId),
    Or(NetId, NetId, NetId),
    Xor(NetId, NetId, NetId),
    Nand(NetId, NetId, NetId),
    Nor(NetId, NetId, NetId),
    Mux {
        first: NetId,
        second: NetId,
        select: NetId,
        output: NetId,
    },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Dff {
    pub(super) data: NetId,
    pub(super) output: NetId,
}
