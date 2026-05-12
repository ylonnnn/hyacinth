use hycc_span::Span;

use crate::{
    basic_block::MirBasicBlockId,
    stmt::{Location, Operand},
};

#[derive(Debug, Clone)]
pub enum MirTerminatorKind {
    Goto(MirBasicBlockId),
    Return,
    Unreachable,
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Location,
        // target: Option<BasicBlockId>,
        // unwind: UnwindAction,
    },
    SwitchInt {
        discr: Operand,
        // targets: SwitchTargets,
    },
    Drop {
        location: Location,
        // target: BasicBlockId,
        // unwind: UnwindAction,
    },
    Assert {
        cond: Operand,
        expected: bool,
        // msg: AssertMessage,
        // target: BasicBlockId,
        // unwind: UnwindAction,
    },
}

#[derive(Debug, Clone)]
pub struct MirTerminator {
    pub kind: MirTerminatorKind,
    pub span: Span,
}

impl MirTerminator {
    pub fn new(kind: MirTerminatorKind, span: Span) -> Self {
        Self { kind, span }
    }
}
