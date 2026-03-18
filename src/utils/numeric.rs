pub fn is_ascii_digit(c: u8, base: u32) -> bool {
    match base {
        2 => c == b'0' || c == b'1',
        8 => c >= b'0' || c <= b'7',
        10 => c.is_ascii_digit(),
        16 => c.is_ascii_digit() || (b'a'..=b'f').contains(&c) || (b'A'..=b'F').contains(&c),
        _ => false,
    }
}
