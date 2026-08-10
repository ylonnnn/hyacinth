use std::collections::{HashMap, hash_map::Entry};

use hycc_hir::{
    HirId,
    def::{Binding, DefId, DefSpace},
};
use hycc_symbol::Symbol;

use crate::{context::TyId, ty::TyKind};

#[derive(Debug, Clone)]
pub struct ExtensionTable {
    data: Vec<Extension>,
    native: HashMap<ExtTargetKind, Vec<ExtensionId>>,

    hir_map: HashMap<HirId, ExtensionId>,
    // TODO: protocol: HashMap<DefId, Vec<ExtensionId>>
}

impl ExtensionTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            native: HashMap::new(),
            hir_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, ext: Extension) -> ExtensionId {
        self.data.push(ext);
        ExtensionId(self.data.len() - 1)
    }

    pub fn get(&self, id: ExtensionId) -> &Extension {
        &self.data[id.unwrap()]
    }

    pub fn get_mut(&mut self, id: ExtensionId) -> &mut Extension {
        &mut self.data[id.unwrap()]
    }

    pub fn get_hir_ext_id(&self, hir_id: HirId) -> Option<ExtensionId> {
        self.hir_map.get(&hir_id).cloned()
    }

    pub fn get_hir_ext(&self, hir_id: HirId) -> Option<&Extension> {
        self.hir_map.get(&hir_id).map(|ext_id| self.get(*ext_id))
    }

    pub fn get_hir_mut_ext(&mut self, hir_id: HirId) -> Option<&mut Extension> {
        self.hir_map
            .get(&hir_id)
            .cloned()
            .map(|ext_id| self.get_mut(ext_id))
    }

    pub fn expect_hir_ext_id(&self, hir_id: HirId) -> ExtensionId {
        self.get_hir_ext_id(hir_id).expect(&format!(
            "expected an extension id attached to the hir id {hir_id:?}"
        ))
    }

    pub fn expect_hir_ext(&self, hir_id: HirId) -> &Extension {
        self.get(self.expect_hir_ext_id(hir_id))
    }

    pub fn expect_hir_mut_ext(&mut self, hir_id: HirId) -> &mut Extension {
        self.get_mut(self.expect_hir_ext_id(hir_id))
    }

    pub fn attach(&mut self, target: ExtTargetKind, ext: Extension) -> ExtensionId {
        // TODO: identify whether the extension is native or protocol-based
        let ext_id = self.insert(ext);
        self.attach_id(target, ext_id);

        ext_id
    }

    // pub fn attach(&mut self, def_id: DefId, ext: Extension) -> ExtensionId {
    //     let ext_id = self.insert(ext);
    //     self.attach_id(def_id, ext_id);

    //     ext_id
    // }

    pub fn attach_id(&mut self, target: ExtTargetKind, ext_id: ExtensionId) {
        match self.native.entry(target) {
            Entry::Vacant(entry) => {
                entry.insert(vec![ext_id]);
            }

            Entry::Occupied(mut entry) => {
                let extensions: &mut Vec<ExtensionId> = entry.get_mut();
                extensions.push(ext_id);
            }
        }
    }

    pub fn attach_hir_ext_id(&mut self, hir_id: HirId, ext_id: ExtensionId) {
        self.hir_map.insert(hir_id, ext_id);
    }

    // pub fn attach_id(&mut self, def_id: DefId, ext_id: ExtensionId) {
    //     match self.native.entry(def_id) {
    //         Entry::Vacant(entry) => {
    //             entry.insert(vec![ext_id]);
    //         }

    //         Entry::Occupied(mut entry) => {
    //             let extensions: &mut Vec<ExtensionId> = entry.get_mut();
    //             extensions.push(ext_id);
    //         }
    //     }
    // }

    pub fn get_ty_native_exts(&self, target: ExtTargetKind) -> Option<&[ExtensionId]> {
        self.native.get(&target).map(|exts| exts.as_slice())
    }

    pub fn get_assoc_item(
        &self,
        target: ExtTargetKind,
        space: DefSpace,
        name: Symbol,
    ) -> Option<(ExtensionId, Binding)> {
        let f = |ext_id: &ExtensionId| {
            self.get(*ext_id)
                .items
                .get(&(space, name))
                .map(|binding| (*ext_id, binding.clone()))
        };

        let find = |exts: Option<&[ExtensionId]>| exts?.iter().find_map(f);
        find(self.get_ty_native_exts(target)).or_else(|| {
            find(self.get_ty_native_exts(ExtTargetKind::Nominal(ExtNominalTargetKind::Blanket)))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtTargetKind {
    Nominal(ExtNominalTargetKind),
    Tuple(usize),
    Slice,
    Array,
    Ref,
    // TODO: Fn (?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtNominalTargetKind {
    Def(DefId),
    Blanket,
}

#[derive(Debug, Clone)]
pub struct Extension {
    items: HashMap<(DefSpace, Symbol), Binding>,
    pub target: Option<TyId>,
    pub hir_id: HirId,
}

impl Extension {
    pub fn new(
        hir_id: HirId,
        target: Option<TyId>,
        items: HashMap<(DefSpace, Symbol), Binding>,
    ) -> Self {
        Self {
            items,
            target,
            hir_id,
        }
    }

    pub fn get_target(&self) -> Option<TyId> {
        self.target
    }

    pub fn expect_target(&self) -> TyId {
        self.target
            .expect("expected the ty id of the extension to already be attached!")
    }

    pub fn attach_target(&mut self, target: TyId) {
        self.target.replace(target);
    }

    pub fn get(&self, space: DefSpace, name: Symbol) -> Option<&Binding> {
        self.items.get(&(space, name))
    }

    pub fn get_mut(&mut self, space: DefSpace, name: Symbol) -> Option<&mut Binding> {
        self.items.get_mut(&(space, name))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtensionId(usize);

impl ExtensionId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "extension id is not valid!");
        self.0
    }
}
