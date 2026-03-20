use hyacinth::{hysm::hysm::Hysm, vm::vm::VirtualMachine};

fn main() {
    //
}

fn usage() {
    println!("Usage: hyc [options] input.hysm");
}

#[inline]
fn shift(args: Vec<String>) -> (String, Vec<String>) {
    println!("{args:?}");
    assert!(args.len() >= 1);
    let mut args = args.into_iter();
    (args.next().unwrap(), args.as_slice().to_vec())
}

fn command() {
    let args: Vec<String> = std::env::args().collect();
    let (_, args) = shift(args);

    if args.len() < 1 {
        usage();
        std::process::exit(1);
    }

    let (opt, args) = shift(args);
    let (input, _) = shift(args);

    match &*opt {
        "compile" | "c" => {
            let mut hysm = Hysm::new(&input);
            hysm.compile(&input.replace(".hysm", ".hycb"));
        }

        "run" | "r" => {
            let mut vm = VirtualMachine::new_from_file(&input).unwrap();
            match vm.execute() {
                Ok(_) => {
                    println!("{:?}", vm.registers);
                    for frame in &vm.frames {
                        println!("Frame\n{frame}");
                    }
                }

                Err(err) => panic!("{err:?}"),
            }
        }

        _ => {
            panic!("unknown option: {opt}");
        }
    }
}
