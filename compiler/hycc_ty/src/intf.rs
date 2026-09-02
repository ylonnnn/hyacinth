use std::collections::HashMap;

use hycc_hir::{
    HirId,
    def::{Binding, DefSpace},
};
use hycc_symbol::Symbol;
use hycc_util::ternary;

use crate::ctx::TyId;

#[derive(Debug)]
pub struct IntfTable {
    data: Vec<Intf>,
    hir_map: HashMap<HirId, IntfId>,
    ty_map: HashMap<TyId, IntfId>,
}

impl IntfTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            hir_map: HashMap::new(),
            ty_map: HashMap::new(),
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

    pub fn attach_hir_intf(&mut self, hir_id: HirId, intf: Intf) -> IntfId {
        let intf_id = self.insert(intf);
        self.attach_hir_intf_id(hir_id, intf_id);

        intf_id
    }

    pub fn attach_hir_intf_id(&mut self, hir_id: HirId, intf_id: IntfId) {
        self.hir_map.insert(hir_id, intf_id);
    }

    pub fn attach_ty_intf(&mut self, ty_id: TyId, intf: Intf) -> IntfId {
        let intf_id = self.insert(intf);
        self.attach_ty_intf_id(ty_id, intf_id);

        intf_id
    }

    pub fn attach_ty_intf_id(&mut self, ty_id: TyId, intf_id: IntfId) {
        self.ty_map.insert(ty_id, intf_id);
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

    pub fn get_ty_intf_id(&self, ty_id: TyId) -> Option<IntfId> {
        self.ty_map.get(&ty_id).cloned()
    }

    pub fn expect_ty_intf_id(&self, ty_id: TyId) -> IntfId {
        self.get_ty_intf_id(ty_id)
            .unwrap_or_else(|| panic!("expected an intf id attached to ty id {ty_id:?}"))
    }

    // pub fn get_assoc_items(&self, space: DefSpace, name: Symbol) -> Vec
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
