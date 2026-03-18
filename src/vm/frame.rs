use std::fmt::Display;

pub const STACK_SIZE_LIMIT: usize = u16::MAX as usize;

#[derive(Debug, Clone)]
pub struct Frame {
    pub stack: Vec<u8>,
    pub ret_addr: usize,
}

impl Frame {
    pub fn new(ret_addr: usize) -> Self {
        Self {
            stack: Vec::with_capacity(STACK_SIZE_LIMIT),
            ret_addr,
        }
    }

    pub fn push(&mut self, data: u8) {
        self.stack.push(data);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    /// NOTE: Ensure that the bytes stored in the stack are arranged in Big-Endian
    pub fn get<const N: usize>(&self, offset: usize) -> u32 {
        assert!(N <= 4);
        u32::from_be_bytes(self.stack[offset..(offset + N)].as_chunks::<4>().0[0])
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stack:")?;
        // for data in &self.stack {
        //     write!(f, "    0x{:02X} ({})\n", data, data)?;
        // }

        for (i, byte) in self.stack.iter().enumerate() {
            if i % 8 == 0 {
                write!(f, "\n    ")?;
            }

            write!(f, "0x{:02X} ", byte)?;
        }

        write!(f, "\n")
    }
}
