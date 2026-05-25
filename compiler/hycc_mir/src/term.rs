use std::fmt::Display;

use hycc_span::Span;

use crate::{
    basic_block::MirBasicBlockId,
    stmt::{Location, Operand},
};

#[derive(Debug, Clone)]
pub enum MirTerminatorKind {
    Goto(MirBasicBlockId),

    Ret,
    Unreachable,

    Call {
        func: Operand,
        args: Vec<Operand>,
        dest: Location,
        // target: Option<BasicBlockId>,
        // unwind: UnwindAction,
    },

    SwitchInt {
        discr: Operand,
        targets: Vec<MirBasicBlockId>,
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

impl Display for MirTerminator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            MirTerminatorKind::Goto(id) => write!(f, "goto bb{}", id.0),

            MirTerminatorKind::Ret => write!(f, "ret"),

            MirTerminatorKind::SwitchInt { discr, targets } => write!(
                f,
                "switch_int ({}) [{}]",
                discr,
                targets
                    .iter()
                    .enumerate()
                    .map(|(i, target)| format!("{}: bb{}", i, target.0))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            _ => write!(f, "{:?}", &self.kind),
        }
    }
}
