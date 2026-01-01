use std::process;

#[cold]
pub fn terminate(message: &str) -> ! {
    println!("hyacinth::termination: {message}");
    process::exit(0);
}
