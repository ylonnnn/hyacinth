use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use hycc_hir::{
    HirId,
    def::{Binding, DefId, DefSpace},
};
use hycc_symbol::Symbol;
use hycc_util::ternary;

use crate::{ctx::TyId, extension::ExtensionId, ty::GenericArg};

#[derive(Debug)]
pub struct IntfTable {
    hir_map: HashMap<HirId, IntfId>,
    ext_map: HashMap<ExtensionId, IntfInstId>,

    inst_map: HashMap<IntfInst, IntfInstId>,

    data: Vec<Intf>,
    inst: Vec<IntfInst>,
}

impl IntfTable {
    pub fn new() -> Self {
        Self {
            hir_map: HashMap::new(),
            ext_map: HashMap::new(),

            inst_map: HashMap::new(),

            data: Vec::new(),
            inst: Vec::new(),
        }
    }

    pub fn insert(&mut self, intf: Intf) -> IntfId {
        self.data.push(intf);
        IntfId(self.data.len() - 1)
    }

    pub fn get(&self, intf_id: IntfId) -> &Intf {
        &self.data[intf_id.unwrap()]
    }

    pub fn get_mut(&mut self, intf_id: IntfId) -> &mut Intf {
        &mut self.data[intf_id.unwrap()]
    }

    pub fn get_inst(&self, inst_id: IntfInstId) -> &IntfInst {
        &self.inst[inst_id.unwrap()]
    }

    pub fn get_mut_inst(&mut self, inst_id: IntfInstId) -> &mut IntfInst {
        &mut self.inst[inst_id.unwrap()]
    }

    pub fn attach_hir_intf_id(&mut self, hir_id: HirId, intf_id: IntfId) {
        self.hir_map.insert(hir_id, intf_id);
    }

    pub fn attach_hir_intf(&mut self, hir_id: HirId, intf: Intf) -> IntfId {
        let intf_id = self.insert(intf);
        self.attach_hir_intf_id(hir_id, intf_id);

        intf_id
    }

    pub fn attach_ext_inst_id(&mut self, ext_id: ExtensionId, inst_id: IntfInstId) {
        self.ext_map.insert(ext_id, inst_id);
    }

    pub fn attach_ext_inst(&mut self, ext_id: ExtensionId, inst: IntfInst) -> IntfInstId {
        let inst_id = self.intern_inst(inst);
        self.ext_map.insert(ext_id, inst_id);

        inst_id
    }

    pub fn get_hir_intf_id(&self, hir_id: HirId) -> Option<IntfId> {
        self.hir_map.get(&hir_id).cloned()
    }

    pub fn get_hir_intf(&self, hir_id: HirId) -> Option<&Intf> {
        self.get_hir_intf_id(hir_id)
            .map(|intf_id| self.get(intf_id))
    }

    pub fn expect_hir_intf_id(&self, hir_id: HirId) -> IntfId {
        self.get_hir_intf_id(hir_id)
            .unwrap_or_else(|| panic!("expected an intf id attached to hir id {hir_id:?}"))
    }

    pub fn expect_hir_intf(&self, hir_id: HirId) -> &Intf {
        self.get(self.expect_hir_intf_id(hir_id))
    }

    pub fn get_ext_inst_id(&self, ext_id: ExtensionId) -> Option<IntfInstId> {
        self.ext_map.get(&ext_id).cloned()
    }

    pub fn get_ext_inst(&self, ext_id: ExtensionId) -> Option<&IntfInst> {
        self.get_ext_inst_id(ext_id)
            .map(|inst_id| self.get_inst(inst_id))
    }

    pub fn expect_ext_inst_id(&self, ext_id: ExtensionId) -> IntfInstId {
        self.get_ext_inst_id(ext_id)
            .unwrap_or_else(|| panic!("expected an intf inst id attached to ext id {ext_id:?}"))
    }

    pub fn expect_ext_inst(&self, ext_id: ExtensionId) -> &IntfInst {
        self.get_inst(self.expect_ext_inst_id(ext_id))
    }

    pub fn intern_inst(&mut self, inst: IntfInst) -> IntfInstId {
        *self.inst_map.entry(inst.clone()).or_insert_with(|| {
            self.inst.push(inst);
            IntfInstId(self.inst.len() - 1)
        })
    }

    pub fn instantiate(&mut self, intf_id: IntfId, args: Arc<[GenericArg]>) -> IntfInstId {
        self.intern_inst(IntfInst::new(intf_id, args))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntfItemKind {
    Req,
    Opt,
}

impl IntfItemKind {
    pub fn is_req(&self) -> bool {
        matches!(&self, Self::Req)
    }

    pub fn is_opt(&self) -> bool {
        matches!(&self, Self::Opt)
    }
}

impl From<bool> for IntfItemKind {
    fn from(value: bool) -> Self {
        ternary!(value, Self::Req, Self::Opt)
    }
}

#[derive(Debug)]
pub struct IntfItem {
    pub binding: Binding,
    pub kind: IntfItemKind,
}

impl IntfItem {
    pub fn req(binding: Binding) -> Self {
        Self {
            binding,
            kind: IntfItemKind::Req,
        }
    }

    pub fn opt(binding: Binding) -> Self {
        Self {
            binding,
            kind: IntfItemKind::Opt,
        }
    }

    pub fn is_req(&self) -> bool {
        self.kind.is_req()
    }

    pub fn is_opt(&self) -> bool {
        self.kind.is_opt()
    }
}

#[derive(Debug)]
pub struct Intf {
    pub items: HashMap<(DefSpace, Symbol), IntfItem>,
    pub hir_id: HirId,
    pub name: Symbol,
    pub generic_param_count: usize,
}

impl Intf {
    pub fn new(
        hir_id: HirId,
        name: Symbol,
        generic_param_count: usize,
        items: HashMap<(DefSpace, Symbol), IntfItem>,
    ) -> Self {
        Self {
            items,
            hir_id,
            name,
            generic_param_count,
        }
    }

    pub fn get(&self, space: DefSpace, name: Symbol) -> Option<&IntfItem> {
        self.items.get(&(space, name))
    }

    pub fn get_mut(&mut self, space: DefSpace, name: Symbol) -> Option<&mut IntfItem> {
        self.items.get_mut(&(space, name))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntfId(usize);

impl IntfId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "intf id is invalid!");
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntfInst {
    pub args: Arc<[GenericArg]>,
    pub intf_id: IntfId,
}

impl IntfInst {
    pub fn new(intf_id: IntfId, args: Arc<[GenericArg]>) -> Self {
        Self { args, intf_id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntfInstId(usize);

impl IntfInstId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "intf inst id is invalid!");
        self.0
    }
}
