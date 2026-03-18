use crate::{
    hysm::parser::{HysmParser, HysmTokenIter, HysmTokenKind, HysmTokenSpan},
    vm::instr::{
        Const, Instruction, Label, Reg, StackOffset,
        instructions::{Add, Div, Halt, Jmp, Mov, Mul, Pop, Push, Sub},
    },
};

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

    // jmp [offset]
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
            todo!("implement offsets")
        }
    }

    // halt
    pub(super) fn parse_halt(&mut self) -> Instruction {
        Halt()
    }
}
