use hycc_vm::instr::{
    Const, Instruction, InstructionOperand, Label, Reg, StackOffset,
    instructions::{Add, And, Cmp, Div, Eq, Halt, Jmp, JmpIf, Mov, Mul, Not, Or, Pop, Push, Sub},
};

use crate::parser::{HysmParser, HysmTokenIter, HysmTokenKind};

impl HysmParser {
    // push [register]
    pub(super) fn parse_push(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let register = iter.require(HysmTokenKind::Register);

        Push(Reg(register.data.unwrap() as u8))
    }

    // pop
    pub(super) fn parse_pop(&mut self) -> Instruction {
        Pop()
    }

    // load [register], [offset]
    pub(super) fn parse_load(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // // Register
        // let register = iter.expect(HysmTokenKind::Register);

        // Push(Reg(register.data.unwrap() as u8)))
        todo!()
    }

    // mov [register], [constant]
    pub(super) fn parse_mov(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let register = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Constant
        let constant = iter.require(HysmTokenKind::Constant);

        Mov(
            Reg(register.data.unwrap() as u8),
            Const(constant.data.unwrap()),
        )
    }

    // add [register], [register], [register]
    pub(super) fn parse_add(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let r1 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r2 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r3 = iter.require(HysmTokenKind::Register);

        Add(
            Reg(r1.data.unwrap() as u8),
            Reg(r2.data.unwrap() as u8),
            Reg(r3.data.unwrap() as u8),
        )
    }

    // sub [register], [register], [register]
    pub(super) fn parse_sub(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let r1 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r2 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r3 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        Sub(
            Reg(r1.data.unwrap() as u8),
            Reg(r2.data.unwrap() as u8),
            Reg(r3.data.unwrap() as u8),
        )
    }

    // mul [register], [register], [register]
    pub(super) fn parse_mul(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let r1 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r2 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r3 = iter.require(HysmTokenKind::Register);

        Mul(
            Reg(r1.data.unwrap() as u8),
            Reg(r2.data.unwrap() as u8),
            Reg(r3.data.unwrap() as u8),
        )
    }

    // div [register], [register], [register]
    pub(super) fn parse_div(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let r1 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r2 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r3 = iter.require(HysmTokenKind::Register);

        Div(
            Reg(r1.data.unwrap() as u8),
            Reg(r2.data.unwrap() as u8),
            Reg(r3.data.unwrap() as u8),
        )
    }

    // not [register]
    pub(super) fn parse_not(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let register = iter.require(HysmTokenKind::Register);

        Not(Reg(register.data.unwrap() as u8))
    }

    // and [register], [register|constant]
    pub(super) fn parse_and(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let r1 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        if let (Some(reg), eq) = iter.expect(HysmTokenKind::Register)
            && eq
        {
            And(Reg(r1.data.unwrap() as u8), Reg(reg.data.unwrap() as u8))
        } else {
            let constant = iter.require(HysmTokenKind::Constant);
            And(Reg(r1.data.unwrap() as u8), Const(constant.data.unwrap()))
        }
    }

    // or [register], [register|constant]
    pub(super) fn parse_or(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let r1 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        if let (Some(reg), eq) = iter.expect(HysmTokenKind::Register)
            && eq
        {
            Or(Reg(r1.data.unwrap() as u8), Reg(reg.data.unwrap() as u8))
        }
        // Constant
        else {
            let constant = iter.require(HysmTokenKind::Constant);
            Or(Reg(r1.data.unwrap() as u8), Const(constant.data.unwrap()))
        }
    }

    // cmp [register], [register], [register]
    pub(super) fn parse_cmp(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let r1 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r2 = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register
        let r3 = iter.require(HysmTokenKind::Register);

        Cmp(
            Reg(r1.data.unwrap() as u8),
            Reg(r2.data.unwrap() as u8),
            Reg(r3.data.unwrap() as u8),
        )
    }

    // eq [register], [register], [register]
    pub(super) fn parse_eq(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        // Register
        let receiver = iter.require(HysmTokenKind::Register);
        iter.require(HysmTokenKind::Comma);

        // Register | Constant
        let left = if let (Some(reg), eq) = iter.expect(HysmTokenKind::Register)
            && eq
        {
            Reg(reg.data.unwrap() as u8)
        } else {
            Const(iter.require(HysmTokenKind::Constant).data.unwrap())
        };

        iter.require(HysmTokenKind::Comma);

        // Register | Constant
        let right = if let (Some(reg), eq) = iter.expect(HysmTokenKind::Register)
            && eq
        {
            Reg(reg.data.unwrap() as u8)
        } else {
            Const(iter.require(HysmTokenKind::Constant).data.unwrap())
        };

        Eq(Reg(receiver.data.unwrap() as u8), left, right)
    }

    // jmp [label|offset]
    pub(super) fn parse_jmp(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        if let (Some(ident), eq) = iter.expect(HysmTokenKind::Ident)
            && eq
        {
            let label =
                self.source.lines[ident.span.line - 1][ident.span.start..ident.span.end].to_owned();

            if let Some(addr) = self.labels.addr_of(&label) {
                Jmp(StackOffset(addr as u32))
            } else {
                Jmp(Label(self.labels.defer(label) as u32))
            }
        } else {
            todo!("implement jmp offsets")
        }
    }

    // jmp_if [label|offset], [register]
    pub(super) fn parse_jmp_if(&mut self, iter: &mut HysmTokenIter) -> Instruction {
        if let (Some(ident), eq) = iter.expect(HysmTokenKind::Ident)
            && eq
        {
            iter.require(HysmTokenKind::Comma);

            let register = Reg(iter.require(HysmTokenKind::Register).data.unwrap() as u8);
            let label =
                self.source.lines[ident.span.line - 1][ident.span.start..ident.span.end].to_owned();

            if let Some(addr) = self.labels.addr_of(&label) {
                JmpIf(StackOffset(addr as u32), register)
            } else {
                JmpIf(Label(self.labels.defer(label) as u32), register)
            }
        } else {
            todo!("implement jmp_if offsets")
        }
    }

    // halt
    pub(super) fn parse_halt(&mut self) -> Instruction {
        Halt()
    }
}
