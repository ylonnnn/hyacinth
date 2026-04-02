use crate::ternary;

pub fn is_ascii_digit(c: u8, base: u32) -> bool {
    match base {
        2 => c == b'0' || c == b'1',
        8 => c >= b'0' || c <= b'7',
        10 => c.is_ascii_digit(),
        16 => c.is_ascii_digit() || (b'a'..=b'f').contains(&c) || (b'A'..=b'F').contains(&c),
        _ => false,
    }
}

pub fn digit_value(c: u8, base: u32) -> u8 {
    match base {
        2 | 8 | 10 => c - b'0',
        16 => ternary!(
            c.is_ascii_digit(),
            c - b'0',
            ternary!(
                (b'a'..=b'f').contains(&c),
                c - b'a',
                ternary!((b'A'..=b'F').contains(&c), c - b'A', 0)
            ) + 10
        ),

        _ => 0,
    }
}
