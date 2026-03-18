use crate::vm::instr::InstructionOperand;

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: BinaryOpKind,
    pub left: InstructionOperand,
    pub right: InstructionOperand,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum BinaryOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[inline]
const fn post_dec(val: &mut u8) -> u8 {
    (*val, *val -= 1).0
}

impl BinaryOpKind {
    pub fn precedence(&self) -> u8 {
        let mut max = u8::MAX;
        match self {
            BinaryOpKind::Mul | BinaryOpKind::Div | BinaryOpKind::Mod => post_dec(&mut max),
            BinaryOpKind::Add | BinaryOpKind::Sub => post_dec(&mut max),

            #[allow(unreachable_patterns)]
            _ => post_dec(&mut max),
        }
    }
}
