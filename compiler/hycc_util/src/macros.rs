#[macro_export]
macro_rules! ternary {
    ($cond:expr, $t_res:expr, $f_res:expr) => {
        if $cond { $t_res } else { $f_res }
    }
}

#[macro_export]
macro_rules! coalesce {
    ($expr:expr, $fail:expr) => {
        match $expr {
            None => $fail,
            Some(v) => v,
        }
    };
}

#[macro_export]
macro_rules! hashmap {
    [$($($key:expr $(,)?)* => $value:expr $(,)?)*] => {{
        #[allow(unused_mut)]
        let mut map = HashMap::new();
        $($(map.insert($key, $value);)*)*

        map
    }}
}

#[macro_export]
macro_rules! terminate {
    ($($arg:tt)*) => {
        print!("[termination] ");
        println!($($arg)*);
        std::process::exit(1);
    };
}
