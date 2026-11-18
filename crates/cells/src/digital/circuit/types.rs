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

impl Gate {
    pub(super) fn output(self) -> NetId {
        match self {
            Self::Const(_, output)
            | Self::Buf(_, output)
            | Self::Not(_, output)
            | Self::And(_, _, output)
            | Self::Or(_, _, output)
            | Self::Xor(_, _, output)
            | Self::Nand(_, _, output)
            | Self::Nor(_, _, output)
            | Self::Mux { output, .. } => output,
        }
    }

    pub(super) fn inputs(self) -> Vec<NetId> {
        match self {
            Self::Const(_, _) => Vec::new(),
            Self::Buf(first, _) | Self::Not(first, _) => vec![first],
            Self::And(first, second, _)
            | Self::Or(first, second, _)
            | Self::Xor(first, second, _)
            | Self::Nand(first, second, _)
            | Self::Nor(first, second, _) => vec![first, second],
            Self::Mux {
                first,
                second,
                select,
                ..
            } => vec![first, second, select],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Dff {
    pub(super) data: NetId,
    pub(super) output: NetId,
}
