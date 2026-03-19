use hyacinth::{hysm::hysm::Hysm, vm::vm::VirtualMachine};

fn main() {
    let mut hysm = Hysm::new("hyc/hysm/loop_test.hysm");

    hysm.execute();

    println!("{:?}", hysm.vm.registers);
    for frame in &hysm.vm.frames {
        println!("Frame\n{}", frame);
    }

    let out = "hyc/hycb/loop_test.hycb";
    hysm.compile(out);

    let mut vm = VirtualMachine::new_from_file(out).unwrap();
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
