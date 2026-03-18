use crate::{
    hysm::parser::HysmParser,
    vm::{label::LabelTable, vm::VirtualMachine},
};

#[derive(Debug)]
pub struct Hysm {
    pub(super) parser: HysmParser,
    pub vm: VirtualMachine,
}

impl Hysm {
    pub fn new(source: &str) -> Self {
        let mut inst = Self {
            parser: HysmParser::new(source),
            vm: VirtualMachine::new(Vec::new(), LabelTable::new()),
        };

        inst.translate();
        inst
    }

    fn translate(&mut self) {
        let mut instructions = self.parser.parse();

        std::mem::swap(&mut self.vm.labels, &mut self.parser.labels);
        std::mem::swap(&mut self.vm.instr, &mut instructions);
    }

    pub fn compile(&mut self, output: &str) {
        match self.vm.compile(Some(output)) {
            Err(err) => panic!("an error occurred: {err:?}"),
            _ => (),
        };
    }

    pub fn execute(&mut self) {
        match self.vm.execute() {
            Err(err) => panic!("an error occurred: {err:?}"),
            _ => (),
        }
    }
}
