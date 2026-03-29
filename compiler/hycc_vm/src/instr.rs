use std::fmt::Debug;

use crate::{label::LabelTable, vm::REGISTER_LIMIT};

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum OpCode {
    Push,  // push [reg]
    Pop,   // pop
    Load,  // load [reg], [stack-offset]
    Mov,   // mov [reg], [const]
    Add,   // add [reg], [reg], [reg]
    Sub,   // sub [reg], [reg], [reg]
    Mul,   // mul [reg], [reg], [reg]
    Div,   // div [reg], [reg], [reg]
    Not,   // not [reg]
    And,   // and [reg], [reg|const]
    Or,    // or [reg], [reg|const]
    Cmp,   // cmp [reg], [reg], [reg]
    Eq,    // eq [reg], [reg|const], [reg|const]
    Jmp,   // jmp [label|offset]
    JmpIf, // jmp [label|offset], [reg]
    Halt,  // halt
    COUNT,
}

impl OpCode {
    #[inline]
    pub const fn op_count(&self) -> usize {
        match self {
            Self::Pop | OpCode::Halt => 0,
            Self::Push | Self::Jmp | Self::Not => 1,
            Self::Mov | Self::Load | Self::JmpIf => 2,
            Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::And
            | Self::Or
            | Self::Cmp
            | Self::Eq => 3,
            Self::COUNT => panic!("not a valid OpCode with an op_count!"),
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= OpCode::COUNT as u8 {
            Err(()) // TODO: implement better error handling
        } else {
            unsafe { std::mem::transmute(value) }
        }
    }
}

/// Must only be within the range of a u8
pub const INSTR_OPERAND_LIMIT: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct InstructionOperand(pub u32);

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum InstructionOperandKind {
    Register = 0,
    Constant,
    StackOffset,
    Label,
    COUNT,
}

#[allow(non_snake_case)]
pub fn Reg(register: u8) -> InstructionOperand {
    assert!(
        (register as usize) < REGISTER_LIMIT,
        "register must be within 0 to {}",
        REGISTER_LIMIT - 1
    );

    InstructionOperand::new(InstructionOperandKind::Register, register as u32)
}

#[allow(non_snake_case)]
pub fn Const(constant: u32) -> InstructionOperand {
    InstructionOperand::new(InstructionOperandKind::Constant, constant)
}

#[allow(non_snake_case)]
pub fn StackOffset(offset: u32) -> InstructionOperand {
    InstructionOperand::new(InstructionOperandKind::StackOffset, offset)
}

#[allow(non_snake_case)]
pub fn Label(id: u32) -> InstructionOperand {
    InstructionOperand::new(InstructionOperandKind::Label, id)
}

impl InstructionOperand {
    pub fn new(kind: InstructionOperandKind, data: u32) -> Self {
        Self(((kind as u32) << 30) | (data & ((1 << 30) - 1)))
    }

    pub fn new_from_raw(data: u32) -> Self {
        let kind = data >> 30;
        assert!(
            kind < (InstructionOperandKind::COUNT as u32),
            "invalid instruction operand kind {}",
            kind
        );

        Self(data)
    }

    pub const fn kind(&self) -> InstructionOperandKind {
        unsafe { std::mem::transmute((self.0 >> 30) as u8) }
    }

    pub const fn is(&self, kind: InstructionOperandKind) -> bool {
        (self.0 >> 30) == (kind as u32)
    }

    pub const fn data(&self) -> u32 {
        self.0 & ((1 << 30) - 1)
    }

    pub const fn reg(&self) -> usize {
        assert!(
            self.is(InstructionOperandKind::Register),
            "operand must be a register!"
        );

        self.data() as usize
    }

    pub const fn stack_offset(&self) -> usize {
        assert!(
            self.is(InstructionOperandKind::StackOffset),
            "operand must be a stack offset!"
        );

        self.data() as usize
    }

    pub fn label_addr(&self, table: &LabelTable) -> usize {
        assert!(
            self.is(InstructionOperandKind::Label),
            "operand must be a label!"
        );

        let label = &table.ids[self.data() as usize];
        match table.addr_of(label) {
            Some(addr) => addr,
            None => panic!("unknown label: {label}"),
        }
    }

    pub const fn to_be_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub const fn to_le_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl Default for InstructionOperand {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub op: OpCode,
    pub operands: [InstructionOperand; INSTR_OPERAND_LIMIT],
}

impl Instruction {
    pub fn new(op: OpCode, operands: [InstructionOperand; INSTR_OPERAND_LIMIT]) -> Self {
        Self { op, operands }
    }

    pub fn new_nop(op: OpCode) -> Self {
        Self {
            op,
            operands: [InstructionOperand::new_from_raw(0); INSTR_OPERAND_LIMIT],
        }
    }
}

macro_rules! instruction {
    ($instr:ident $(,$arg:ident)*) => {
        #[allow(non_snake_case)]
        pub fn $instr($($arg: InstructionOperand),*) -> Instruction {
            let mut _i = 0;
            let op = OpCode::$instr;
            op.op_count();

            #[allow(unused_mut)]
            let mut inst = Instruction {
                op: OpCode::$instr,
                operands: [InstructionOperand(0_u32); INSTR_OPERAND_LIMIT],
            };

            $(
                inst.operands[_i] = $arg;
                _i += 1;
            )*

            inst
        }
    };
}

pub mod instructions {
    use super::*;

    // Macro-defined instructions
    instruction!(Push, register);
    instruction!(Pop);

    instruction!(Mov, register, value);
    instruction!(Load, register, stack_offset);

    instruction!(Add, receiver, op1, op2);
    instruction!(Sub, receiver, op1, op2);
    instruction!(Mul, receiver, op1, op2);
    instruction!(Div, receiver, op1, op2);

    instruction!(Not, register);
    instruction!(And, receiver, op);
    instruction!(Or, receiver, op);

    instruction!(Eq, receiver, left, right);
    instruction!(Cmp, receiver, left, right);

    instruction!(Jmp, addr);
    instruction!(JmpIf, addr, basis);

    instruction!(Halt);
}
