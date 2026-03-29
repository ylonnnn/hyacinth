use std::{collections::HashMap, iter::Peekable, vec::IntoIter};

use hycc_source::Source;
use hycc_util::{is_ascii_digit, ternary};

use hycc_vm::{instr::Instruction, label::LabelTable};

#[derive(Debug, Clone)]
pub struct HysmParser {
    pub(super) source: Source,

    pub(super) line: usize,   // Current line number
    pub(super) offset: usize, // Offset in the current line

    pub(super) labels: LabelTable,

    keywords: HashMap<&'static str, HysmTokenKind>,
}

impl HysmParser {
    pub fn new(source: &str) -> Self {
        Self {
            source: Source::new_from_file(source),
            line: 1,
            offset: 0,
            labels: LabelTable::new(),
            keywords: hashmap! {
                "push" => HysmTokenKind::Push,
                "pop" => HysmTokenKind::Pop,
                "load" => HysmTokenKind::Load,
                "mov" => HysmTokenKind::Mov,
                "add" => HysmTokenKind::Add,
                "sub" => HysmTokenKind::Sub,
                "mul" => HysmTokenKind::Mul,
                "div" => HysmTokenKind::Div,
                "not" => HysmTokenKind::Not,
                "and" => HysmTokenKind::And,
                "or" => HysmTokenKind::Or,
                "cmp" => HysmTokenKind::Cmp,
                "eq" => HysmTokenKind::Eq,
                "jmp" => HysmTokenKind::Jmp,
                "jmp_if" => HysmTokenKind::JmpIf,
                "halt" => HysmTokenKind::Halt,
            },
        }
    }

    fn read_ident(&mut self) -> HysmToken {
        let bytes = self.source.lines[self.line - 1].as_bytes();
        let start = (self.offset, self.offset += 1).0;

        while let Some(byte) = bytes.get(self.offset)
            && (byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            self.offset += 1;
        }

        let register_marker = "%r";
        let token = match str::from_utf8(&bytes[start..self.offset]) {
            Ok(view) if view.starts_with(register_marker) => {
                let Ok(data) = view[register_marker.len()..].parse::<u32>() else {
                    panic!("invalid register provided!")
                };

                HysmToken::new_with(
                    HysmTokenKind::Register,
                    hysm_token_span!(self.line, start, self.offset),
                    data,
                )
            }

            Ok(view) => HysmToken::new(
                match self.keywords.get(view) {
                    Some(kind) => kind.clone(),
                    None => HysmTokenKind::Ident,
                },
                hysm_token_span!(self.line, start, self.offset),
            ),
            Err(err) => panic!("an error occurred: {err:?}"),
        };

        // Identifier processor
        match token.kind {
            HysmTokenKind::Register => token,
            _ => token,
        }
    }

    fn read_digits(&mut self, base: u32) -> bool {
        let bytes = self.source.lines[self.line - 1].as_bytes();

        while let Some(byte) = bytes.get(self.offset)
            && !byte.is_ascii_whitespace()
            && byte.is_ascii_alphanumeric()
        {
            if !is_ascii_digit(*byte, base) {
                // TODO: throw error: invalid numeric literal digit for {base}
                return false;
            }

            self.offset += 1;
        }

        true
    }

    fn read_const(&mut self) -> Option<HysmToken> {
        let bytes = self.source.lines[self.line - 1].as_bytes();
        let start = self.offset;

        // Identify base according to prefix
        let mut base: u32 = 10;
        let mut pref_len = 0;

        if let Some(b'0') = bytes.get(self.offset) {
            if let Some(c) = bytes.get(self.offset + 1) {
                #[allow(unused)]
                let mut n = 1; // Used but is not detected

                base = match c {
                    b'b' => 2,
                    b'o' => 8,
                    b'x' => 16,
                    _ if c.is_ascii_digit() || c.is_ascii_whitespace() => 10,
                    _ => {
                        n = c.is_ascii_alphabetic() as usize;
                        ternary!(n == 1, u32::MAX, 10)
                    }
                };

                n = match base {
                    2 | 8 | 16 => 2,
                    _ => 1,
                };

                self.offset += n;
                pref_len = ternary!(n == 2, n, 0);
            }
        }

        if base == u32::MAX {
            // TODO: throw error invalid numeric litera prefix
            // self.dctx.error(
            //     DiagnosticErrorKind::InvalidNumericLiteralPrefix.into(),
            //     &format!(
            //         "invalid numeric literal prefix `{}`.",
            //         &self.lexer.source.data[(start as usize)..=(self.offset as usize)]
            //     ),
            //     (start, self.offset + 1).into(),
            // );

            return None;
        }

        self.read_digits(base); // Integral Part

        Some(HysmToken::new_with(
            HysmTokenKind::Constant,
            hysm_token_span!(self.line, start, self.offset),
            u32::from_str_radix(
                str::from_utf8(
                    &self.source.lines[self.line - 1].as_bytes()[(start + pref_len)..self.offset],
                )
                .unwrap(),
                base,
            )
            .unwrap(),
        ))
    }

    pub fn tokenize_line(&mut self) -> Vec<HysmToken> {
        let mut tokens = Vec::<HysmToken>::new();

        let line = self.source.lines[self.line - 1].clone();
        let bytes = line.as_bytes();

        self.offset = 0;

        'line: while self.offset < bytes.len() {
            let Some(mut byte) = bytes.get(self.offset) else {
                break;
            };

            while byte.is_ascii_whitespace() {
                byte = match bytes.get((self.offset += 1, self.offset).1) {
                    Some(b) => b,
                    None => break 'line,
                }
            }

            let b_bp = self.offset;
            let token = match byte {
                b'%' | b'a'..=b'z' | b'A'..=b'Z' | b'_' => Some(self.read_ident()),
                b'0'..=b'9' => self.read_const(),

                b'[' => Some(HysmToken::new(
                    HysmTokenKind::LeftBracket,
                    hysm_token_span!(self.line, self.offset, self.offset + 1),
                )),

                b']' => Some(HysmToken::new(
                    HysmTokenKind::RightBracket,
                    hysm_token_span!(self.line, self.offset, self.offset + 1),
                )),

                b';' => break,

                b',' => Some(HysmToken::new(
                    HysmTokenKind::Comma,
                    hysm_token_span!(self.line, self.offset, self.offset + 1),
                )),
                b':' => Some(HysmToken::new(
                    HysmTokenKind::Colon,
                    hysm_token_span!(self.line, self.offset, self.offset + 1),
                )),

                _ => panic!("unknown character: {} ({byte})", *byte as char),
            };

            if let Some(token) = token {
                self.offset = b_bp + (token.span.end - token.span.start);
                tokens.push(token);
            }
        }

        tokens
    }

    pub fn tokenize(&mut self) -> Vec<Vec<HysmToken>> {
        let mut tokens = Vec::new();

        while self.line <= self.source.lines.len() {
            tokens.push(self.tokenize_line());
            self.line += 1;
        }

        tokens
    }

    pub fn parse_token_set(&mut self, set: Vec<HysmToken>) -> HysmVirtualMachineData {
        let mut iter = HysmTokenIter::new(set);
        let Some(lead) = iter.next() else {
            panic!("empty token set at line {}", self.line);
        };

        let data = match lead.kind {
            HysmTokenKind::Push => Instr(self.parse_push(&mut iter)),
            HysmTokenKind::Pop => Instr(self.parse_pop()),
            HysmTokenKind::Load => Instr(self.parse_load(&mut iter)),
            HysmTokenKind::Mov => Instr(self.parse_mov(&mut iter)),
            HysmTokenKind::Add => Instr(self.parse_add(&mut iter)),
            HysmTokenKind::Sub => Instr(self.parse_sub(&mut iter)),
            HysmTokenKind::Mul => Instr(self.parse_mul(&mut iter)),
            HysmTokenKind::Div => Instr(self.parse_div(&mut iter)),
            HysmTokenKind::Not => Instr(self.parse_not(&mut iter)),
            HysmTokenKind::And => Instr(self.parse_and(&mut iter)),
            HysmTokenKind::Or => Instr(self.parse_or(&mut iter)),
            HysmTokenKind::Cmp => Instr(self.parse_cmp(&mut iter)),
            HysmTokenKind::Eq => Instr(self.parse_eq(&mut iter)),
            HysmTokenKind::Jmp => Instr(self.parse_jmp(&mut iter)),
            HysmTokenKind::JmpIf => Instr(self.parse_jmp_if(&mut iter)),
            HysmTokenKind::Halt => Instr(self.parse_halt()),

            // Labels
            HysmTokenKind::Ident => {
                iter.require(HysmTokenKind::Colon);

                let span = lead.span;
                Label(self.source.lines[span.line - 1][span.start..span.end].to_owned())
            }

            // HysmTokenKind::Ident => todo!("ident"),
            _ => panic!("invalid leading token: {:?}", lead),
        };

        if iter.tokens.len() != 0 {
            panic!("invalid instruction at line {}", self.line);
        }

        data
    }

    pub fn parse(&mut self) -> Vec<Instruction> {
        let tokens = self.tokenize();
        let mut instructions = Vec::new();

        for token_set in tokens {
            if token_set.is_empty() {
                continue;
            }

            let Some(token) = token_set.get(0) else {
                continue;
            };

            self.line = token.span.line;

            match self.parse_token_set(token_set) {
                Instr(instr) => {
                    instructions.push(instr);
                }

                Label(label) => {
                    self.labels.add(label, instructions.len());
                }
            }
        }

        instructions
    }
}

pub enum HysmVirtualMachineData {
    Instr(Instruction),
    Label(String),
}

use HysmVirtualMachineData::*;

#[derive(Debug, Clone)]
pub struct HysmToken {
    pub kind: HysmTokenKind,
    pub span: HysmTokenSpan,
    pub data: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct HysmTokenSpan {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HysmTokenKind {
    // Instructions
    Push,
    Pop,
    Load,
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Not,
    And,
    Or,
    Cmp,
    Eq,
    Jmp,
    JmpIf,
    Halt,

    Register,
    Constant,
    Offset,

    Ident,

    // Delimeters
    Comma,
    Colon,
    LeftBracket,
    RightBracket,
}

impl HysmToken {
    pub fn new(kind: HysmTokenKind, span: HysmTokenSpan) -> Self {
        Self {
            kind,
            span,
            data: None,
        }
    }

    pub fn new_with(kind: HysmTokenKind, span: HysmTokenSpan, data: u32) -> Self {
        Self {
            kind,
            span,
            data: Some(data),
        }
    }
}

impl HysmTokenSpan {
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        Self { line, start, end }
    }
}

#[macro_export]
macro_rules! hysm_token_span {
    ($line:expr, $start:expr, $end:expr) => {
        HysmTokenSpan::new($line, $start, $end)
    };
}

pub(super) struct HysmTokenIter {
    tokens: Peekable<IntoIter<HysmToken>>,
}

impl HysmTokenIter {
    pub fn new(tokens: Vec<HysmToken>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn next(&mut self) -> Option<HysmToken> {
        self.tokens.next()
    }

    pub fn expect(&mut self, kind: HysmTokenKind) -> (Option<HysmToken>, bool) {
        let token = self.tokens.peek();
        let eq = ternary!(token.is_some(), token.as_ref().unwrap().kind == kind, false);

        (ternary!(eq, self.tokens.next(), None), eq)
    }

    pub fn require(&mut self, kind: HysmTokenKind) -> HysmToken {
        let (token, eq) = self.expect(kind.clone());
        if !eq {
            panic!(
                "expected {:?}{}",
                kind,
                ternary!(
                    token.is_some(),
                    format!(", received {:?}", token.unwrap()),
                    "".into()
                )
            )
        }

        token.unwrap()
    }
}
