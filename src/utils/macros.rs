#[macro_export]
macro_rules! ternary {
    ($cond:expr, $t_res:expr, $f_res:expr) => {
        if $cond { $t_res } else { $f_res }
    };
}
