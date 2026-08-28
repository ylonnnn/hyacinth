pub mod color {
    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m"; // 
    pub const BRIGHT_BLACK: &str = "\x1b[90m"; // Gray
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m"; // 
    pub const BACKGROUND_RED: &str = "\x1b[41m";
    pub const BACKGROUND_GREEN: &str = "\x1b[42m";
    pub const BACKGROUND_YELLOW: &str = "\x1b[43m";
    pub const BACKGROUND_BLUE: &str = "\x1b[44m";
    pub const BACKGROUND_MAGENTA: &str = "\x1b[45m";
    pub const BACKGROUND_CYAN: &str = "\x1b[46m";
    pub const BACKGROUND_WHITE: &str = "\x1b[47m"; // 
    pub const BACKGROUND_BRIGHT_BLACK: &str = "\x1b[100m";
    pub const BACKGROUND_BRIGHT_RED: &str = "\x1b[101m";
    pub const BACKGROUND_BRIGHT_GREEN: &str = "\x1b[102m";
    pub const BACKGROUND_BRIGHT_YELLOW: &str = "\x1b[103m";
    pub const BACKGROUND_BRIGHT_BLUE: &str = "\x1b[104m";
    pub const BACKGROUND_BRIGHT_MAGENTA: &str = "\x1b[105m";
    pub const BACKGROUND_BRIGHT_CYAN: &str = "\x1b[106m";
    pub const BACKGROUND_BRIGHT_WHITE: &str = "\x1b[107m";
}

pub mod style {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const BLINK: &str = "\x1b[5m";
    pub const REVERSE: &str = "\x1b[7m";
    pub const HIDDEN: &str = "\x1b[8m";
}

macro_rules! impl_style {
    ($n:ident, $c:expr) => {
        fn $n(&self) -> String {
            self.style($c)
        }
    };
}

use color::*;
use style::*;

use crate::ternary;

pub trait Style
where
    Self: ToString,
{
    fn style(&self, style: &str) -> String {
        style.to_owned() + &self.to_string()
    }

    // Standard colors
    impl_style!(black, BLACK);
    impl_style!(red, RED);
    impl_style!(green, GREEN);
    impl_style!(yellow, YELLOW);
    impl_style!(blue, BLUE);
    impl_style!(magenta, MAGENTA);
    impl_style!(cyan, CYAN);
    impl_style!(white, WHITE);

    // Bright colors
    impl_style!(bright_black, BRIGHT_BLACK);
    impl_style!(bright_red, BRIGHT_RED);
    impl_style!(bright_green, BRIGHT_GREEN);
    impl_style!(bright_yellow, BRIGHT_YELLOW);
    impl_style!(bright_blue, BRIGHT_BLUE);
    impl_style!(bright_magenta, BRIGHT_MAGENTA);
    impl_style!(bright_cyan, BRIGHT_CYAN);
    impl_style!(bright_white, BRIGHT_WHITE);

    impl_style!(bg_red, BACKGROUND_RED);
    impl_style!(bg_green, BACKGROUND_GREEN);
    impl_style!(bg_yellow, BACKGROUND_YELLOW);
    impl_style!(bg_blue, BACKGROUND_BLUE);
    impl_style!(bg_magenta, BACKGROUND_MAGENTA);
    impl_style!(bg_cyan, BACKGROUND_CYAN);
    impl_style!(bg_white, BACKGROUND_WHITE);

    impl_style!(bg_bright_black, BACKGROUND_BRIGHT_BLACK);
    impl_style!(bg_bright_red, BACKGROUND_BRIGHT_RED);
    impl_style!(bg_bright_green, BACKGROUND_BRIGHT_GREEN);
    impl_style!(bg_bright_yellow, BACKGROUND_BRIGHT_YELLOW);
    impl_style!(bg_bright_blue, BACKGROUND_BRIGHT_BLUE);
    impl_style!(bg_bright_magenta, BACKGROUND_BRIGHT_MAGENTA);
    impl_style!(bg_bright_cyan, BACKGROUND_BRIGHT_CYAN);
    impl_style!(bg_bright_white, BACKGROUND_BRIGHT_WHITE);

    impl_style!(reset, RESET);
    impl_style!(bold, BOLD);
    impl_style!(dim, DIM);
    impl_style!(underline, UNDERLINE);
    impl_style!(blink, BLINK);
    impl_style!(reverse, REVERSE);
    impl_style!(hidden, HIDDEN);
}

impl Style for str {}
impl Style for String {}

pub fn list_enumeration<T: ToString>(list: &[T]) -> String {
    let (mut series, n) = (String::new(), list.len());
    if list.len() == 1 {
        return list[0].to_string();
    }

    for (i, entry) in list.iter().enumerate() {
        let is_last = i == n - 1;
        series += &format!(
            "{}{}{}",
            ternary!(is_last, "and ", ""),
            entry.to_string(),
            ternary!(is_last, "", ", ")
        );
    }

    series
}
