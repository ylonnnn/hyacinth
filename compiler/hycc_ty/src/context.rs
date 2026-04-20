use std::collections::HashMap;

use crate::ty::TyKind;

#[derive(Debug)]
pub struct TyCtx {
    storage: Vec<TyKind>,
    map: HashMap<TyKind, TyId>,
}

impl TyCtx {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn intern(&mut self, ty: TyKind) -> TyId {
        if let Some(ty_id) = self.map.get(&ty) {
            return *ty_id;
        }

        let ty_id = TyId(self.storage.len());

        self.map.insert(ty.clone(), ty_id);
        self.storage.push(ty);

        ty_id
    }

    pub fn get(&self, ty_id: TyId) -> &TyKind {
        &self.storage[ty_id.unwrap()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyId(usize);

impl TyId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "type id is not valid!");
        self.0
    }
}
