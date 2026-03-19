use std::{fs, io};

use crate::{
    ternary,
    vm::{
        frame::Frame,
        instr::{
            INSTR_OPERAND_LIMIT, Instruction, InstructionOperand, InstructionOperandKind, OpCode,
        },
        label::LabelTable,
    },
};

#[derive(Debug)]
pub enum VirtualMachineError {
    ReadFailure(io::Error),
    NonOpCodeByte(u8),
    CompilationError(io::Error),
    InvalidAddress(u32),
    EmptyStackFrame,
}

pub const REGISTER_LIMIT: usize = 16;

#[derive(Debug)]
pub struct VirtualMachine {
    pub instr: Vec<Instruction>,
    pub ip: usize,

    pub labels: LabelTable,
    pub registers: [u32; REGISTER_LIMIT],

    pub frames: Vec<Frame>,
}

impl VirtualMachine {
    pub fn new(instr: Vec<Instruction>, labels: LabelTable) -> Self {
        Self {
            instr,
            ip: 0,
            labels,
            registers: [0_u32; REGISTER_LIMIT],
            frames: vec![Frame::new(usize::MAX)],
        }
    }

    pub fn new_from_file(target: &str) -> Result<Self, VirtualMachineError> {
        Ok(Self::new(Self::read(target)?, LabelTable::new()))
    }

    pub fn goto(&mut self, addr: usize) -> bool {
        ternary!(addr >= self.instr.len(), false, {
            self.ip = addr;
            true
        })
    }

    fn read_operands(
        bytes: &[u8],
        bp: &mut usize,
        n: usize,
    ) -> [InstructionOperand; INSTR_OPERAND_LIMIT] {
        let mut operands = [InstructionOperand::new_from_raw(0); INSTR_OPERAND_LIMIT];
        for i in 0..n {
            operands[i] = InstructionOperand::new_from_raw(u32::from_le_bytes(
                bytes[(*bp)..(*bp + 4)].try_into().unwrap(),
            ));
            *bp += 4;
        }

        operands
    }

    pub fn read(target: &str) -> Result<Vec<Instruction>, VirtualMachineError> {
        let bytes = match fs::read(target) {
            Ok(contents) => contents,
            Err(err) => Err(VirtualMachineError::ReadFailure(err))?,
        };

        let bytes = bytes.as_slice();
        let mut bp = 0;
        let mut instructions = Vec::<Instruction>::new();

        while bp < bytes.len() {
            let byte = bytes[(bp, bp += 1).0];
            let Ok(op) = byte.try_into() else {
                return Err(VirtualMachineError::NonOpCodeByte(byte));
            };

            // For type annotation
            let op: OpCode = op;

            instructions.push(Instruction::new(
                op.clone(),
                Self::read_operands(bytes, &mut bp, op.op_count()),
            ));
        }

        Ok(instructions)
    }

    pub fn compile(&self, output: Option<&str>) -> Result<Vec<u8>, VirtualMachineError> {
        let mut bytes = Vec::<u8>::with_capacity(32);

        for instr in &self.instr {
            bytes.push(instr.op.clone() as u8);

            for i in 0..instr.op.op_count() {
                let op = &instr.operands[i as usize];
                let slice = &match op.kind() {
                    InstructionOperandKind::Register
                    | InstructionOperandKind::Constant
                    | InstructionOperandKind::StackOffset => op.to_le_bytes(),
                    InstructionOperandKind::Label => {
                        (((InstructionOperandKind::StackOffset as u32) << 30)
                            | (op.label_addr(&self.labels) as u32) & ((1 << 30) - 1))
                            .to_le_bytes()
                    }
                    _ => unreachable!(),
                };

                bytes.extend_from_slice(slice);
            }
        }

        if let Some(out_path) = output {
            match fs::write(out_path, bytes.as_slice()) {
                Err(err) => Err(VirtualMachineError::CompilationError(err)),
                _ => Ok(()),
            }?;
        }

        Ok(bytes)
    }

    pub fn execute(&mut self) -> Result<(), VirtualMachineError> {
        let instructions = self.instr.clone();
        let mut terminate = false;

        while !terminate && self.ip < instructions.len() {
            let instr = &instructions[self.ip];
            let [op1, op2, op3] = &instr.operands;

            match instr.op {
                OpCode::Push => {
                    let Some(frame) = self.frames.last_mut() else {
                        return Err(VirtualMachineError::EmptyStackFrame);
                    };

                    for byte in self.registers[instr.operands[0].reg()].to_le_bytes() {
                        frame.push(byte);
                    }
                }

                OpCode::Pop => {
                    self.frames.last_mut().map(|frame| frame.pop());
                }

                OpCode::Load => {
                    let Some(frame) = self.frames.last_mut() else {
                        return Err(VirtualMachineError::EmptyStackFrame);
                    };

                    self.registers[op1.reg()] = frame.get::<4>(op2.stack_offset());
                }

                OpCode::Mov => {
                    self.registers[op1.reg()] = op2.data();
                }

                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                    let (a, b) = (self.registers[op2.reg()], self.registers[op3.reg()]);
                    self.registers[op1.reg()] = match instr.op {
                        OpCode::Add => a + b,
                        OpCode::Sub => a - b,
                        OpCode::Mul => a * b,
                        OpCode::Div => a / b,
                        _ => unreachable!(),
                    }
                }

                OpCode::Not => {
                    let register = &mut self.registers[op1.reg()];
                    *register = !*register;
                }

                OpCode::And => {
                    let op = ternary!(
                        op2.is(InstructionOperandKind::Register),
                        self.registers[op2.reg()],
                        ternary!(
                            op2.is(InstructionOperandKind::Constant),
                            op2.data(),
                            panic!("unexpected operand: {:?}", op2.kind())
                        )
                    );

                    dbg!(op);

                    let register = &mut self.registers[op1.reg()];
                    *register = *register & op;
                }

                OpCode::Or => {
                    let op = ternary!(
                        op2.is(InstructionOperandKind::Register),
                        self.registers[op2.reg()],
                        ternary!(
                            op2.is(InstructionOperandKind::Constant),
                            op2.data(),
                            panic!("unexpected operand: {:?}", op2.kind())
                        )
                    );

                    let register = &mut self.registers[op1.reg()];
                    *register = *register | op;
                }

                OpCode::Cmp => {
                    let (left, right) = (self.registers[op2.reg()], self.registers[op3.reg()]);

                    self.registers[op1.reg()] =
                        ternary!(left > right, 1, ternary!(left < right, 2, 0));
                }

                OpCode::Eq => {
                    self.registers[op1.reg()] = (ternary!(
                        op2.is(InstructionOperandKind::Register),
                        self.registers[op2.reg()],
                        ternary!(
                            op2.is(InstructionOperandKind::Constant),
                            op2.data(),
                            panic!("unexpected operand: {:?}", op2.kind())
                        )
                    ) == ternary!(
                        op3.is(InstructionOperandKind::Register),
                        self.registers[op3.reg()],
                        ternary!(
                            op3.is(InstructionOperandKind::Constant),
                            op3.data(),
                            panic!("unexpected operand: {:?}", op3.kind())
                        )
                    )) as u32;
                }

                OpCode::Jmp => {
                    let addr = ternary!(
                        op1.is(InstructionOperandKind::Label),
                        op1.label_addr(&self.labels),
                        ternary!(
                            op1.is(InstructionOperandKind::StackOffset),
                            op1.data() as usize,
                            panic!("unexpected operand: {:?}", op1.kind())
                        )
                    );

                    if !self.goto(addr) {
                        return Err(VirtualMachineError::InvalidAddress(addr as u32));
                    }

                    continue;
                }

                OpCode::JmpIf => {
                    let addr = ternary!(
                        op1.is(InstructionOperandKind::Label),
                        op1.label_addr(&self.labels),
                        op1.data() as usize
                    );

                    let register = self.registers[op2.reg()];
                    if register == 0 {
                        self.ip += 1;
                        continue;
                    }

                    if !self.goto(addr) {
                        return Err(VirtualMachineError::InvalidAddress(addr as u32));
                    }

                    continue;
                }

                OpCode::Halt => terminate = true,

                _ => todo!(),
            }

            self.ip += 1;
        }

        Ok(())
    }
}
