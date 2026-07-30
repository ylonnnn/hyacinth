use std::collections::HashMap;

use hycc_hir::HirId;

#[derive(Debug, Clone)]
pub struct ExtensionTable {
    data: Vec<Extension>,
    hir_map: HashMap<HirId, ExtensionId>,
}

impl ExtensionTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
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

    pub fn attach(&mut self, hir_id: HirId, ext: Extension) -> ExtensionId {
        let ext_id = self.insert(ext);
        self.attach_id(hir_id, ext_id);

        ext_id
    }

    pub fn attach_id(&mut self, hir_id: HirId, ext_id: ExtensionId) {
        self.hir_map.insert(hir_id, ext_id);
    }

    pub fn get_id_by_hir(&self, hir_id: HirId) -> Option<ExtensionId> {
        self.hir_map.get(&hir_id).cloned()
    }

    pub fn expect_hir_ext_id(&self, hir_id: HirId) -> ExtensionId {
        self.get_id_by_hir(hir_id).expect(&format!(
            "expected an extension id attached to hir id {hir_id:?}"
        ))
    }

    pub fn get_by_hir(&self, hir_id: HirId) -> Option<&Extension> {
        self.get_id_by_hir(hir_id).map(|id| self.get(id))
    }

    pub fn expect_hir_ext(&self, hir_id: HirId) -> &Extension {
        self.get(self.expect_hir_ext_id(hir_id))
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

#[derive(Debug, Clone)]
pub struct Extension {
    pub target: HirId,
    // TODO: with: Option<[PROTO]>
    pub items: Vec<HirId>,
}
