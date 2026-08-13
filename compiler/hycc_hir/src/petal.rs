use std::collections::HashMap;

use hycc_util::bug;

use crate::{
    def::{DefAccessibility, DefId, DefPubAccessibilityKind, Definition},
    scope::{ScopeCtx, ScopeId},
};

#[derive(Debug, Clone)]
pub struct PetalCtx {
    defs: HashMap<DefId, PetalId>,
    data: Vec<Petal>,
    stack: Vec<PetalId>,
}

impl PetalCtx {
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
            data: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn root_petal_id(&self) -> PetalId {
        PetalId(0)
    }

    pub fn create_root_petal(&mut self, scope_id: ScopeId) -> PetalId {
        if self.data.len() >= 1 {
            bug!("root petals cannot be created more than once!")
        } else {
            self.add_petal(Petal::Root(scope_id))
        }
    }

    pub fn create_child_petal(&mut self, def_id: DefId) -> PetalId {
        if self.data.len() < 1 {
            bug!("child petals cannot be created before the root petal!")
        }

        self.add_petal(Petal::Child {
            def_id,
            parent: self.expect_top_id(),
        })
    }

    pub fn try_create_child_petal(&mut self, def_id: DefId) -> PetalId {
        if let Some(petal_id) = self.defs.get(&def_id) {
            *petal_id
        } else {
            self.create_child_petal(def_id)
        }
    }

    pub fn add_petal(&mut self, petal: Petal) -> PetalId {
        let petal_id = PetalId(self.data.len());
        if let Petal::Child { def_id, .. } = &petal {
            self.attach_petal_id(*def_id, petal_id);
        }

        self.data.push(petal);
        petal_id
    }

    pub fn attach_petal(&mut self, def_id: DefId, petal: Petal) -> PetalId {
        let petal_id = self.add_petal(petal);
        self.attach_petal_id(def_id, petal_id);

        petal_id
    }

    pub fn attach_petal_id(&mut self, def_id: DefId, petal_id: PetalId) {
        self.defs.insert(def_id, petal_id);
    }

    pub fn get(&self, id: PetalId) -> Option<&Petal> {
        self.data.get(id.unwrap())
    }

    pub fn get_mut(&mut self, id: PetalId) -> Option<&mut Petal> {
        self.data.get_mut(id.unwrap())
    }

    pub fn expect(&self, id: PetalId) -> &Petal {
        self.get(id)
            .unwrap_or_else(|| panic!("expected a petal attached to {id:?}"))
    }

    pub fn expect_mut(&mut self, id: PetalId) -> &mut Petal {
        self.get_mut(id)
            .unwrap_or_else(|| panic!("expected a petal attached to {id:?}"))
    }

    pub fn get_def_petal_id(&self, def_id: DefId) -> Option<PetalId> {
        self.defs.get(&def_id).cloned()
    }

    pub fn get_def_petal(&self, def_id: DefId) -> Option<&Petal> {
        self.get_def_petal_id(def_id)
            .and_then(|petal_id| self.get(petal_id))
    }

    pub fn get_def_mut_petal(&mut self, def_id: DefId) -> Option<&mut Petal> {
        self.get_def_petal_id(def_id)
            .and_then(|petal_id| self.get_mut(petal_id))
    }

    pub fn expect_def_petal_id(&self, def_id: DefId) -> PetalId {
        self.get_def_petal_id(def_id)
            .unwrap_or_else(|| panic!("expected a petal attached to {def_id:?}"))
    }

    pub fn expect_def_petal(&self, def_id: DefId) -> &Petal {
        self.expect(self.expect_def_petal_id(def_id))
    }

    pub fn expect_def_mut_petal(&mut self, def_id: DefId) -> &mut Petal {
        self.expect_mut(self.expect_def_petal_id(def_id))
    }

    pub fn push(&mut self, id: PetalId) {
        self.stack.push(id)
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn top_id(&self) -> Option<PetalId> {
        self.stack.last().cloned()
    }

    pub fn top(&self) -> Option<&Petal> {
        self.top_id().and_then(|id| self.get(id))
    }

    pub fn top_mut(&mut self) -> Option<&mut Petal> {
        self.top_id().and_then(|id| self.get_mut(id))
    }

    pub fn expect_top_id(&self) -> PetalId {
        self.top_id()
            .unwrap_or_else(|| panic!("expected a petal at the top of the stack to exist"))
    }

    pub fn expect_top(&self) -> &Petal {
        self.expect(self.expect_top_id())
    }

    pub fn expect_top_mut(&mut self) -> &mut Petal {
        self.expect_mut(self.expect_top_id())
    }

    pub fn from_top_id(&self, offset: usize) -> Option<PetalId> {
        let n = self.stack.len();
        if n == 0 || offset > n - 1 {
            return None;
        }

        self.stack.get((n - 1) - offset).cloned()
    }

    pub fn use_petal<F, U>(&mut self, petal_id: PetalId, mut f: F) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        (self.push(petal_id), f(self), self.pop()).1
    }

    pub fn is_ancestor(&self, a: PetalId, b: PetalId) -> bool {
        let mut current = b;
        while let Petal::Child { parent, .. } = self.expect(current) {
            if *parent == a {
                return true;
            }

            current = *parent;
        }

        false
    }

    pub fn relationship(&self, a: PetalId, b: PetalId) -> PetalRelationship {
        if a == b {
            return PetalRelationship::This;
        }

        let (a_petal, b_petal) = (self.expect(a), self.expect(b));
        match (a_petal, b_petal) {
            (Petal::Child { parent: a_par, .. }, _) if *a_par == b => PetalRelationship::Child,
            (_, Petal::Child { parent: b_par, .. }) if *b_par == a => PetalRelationship::Super,
            (Petal::Child { parent: a_par, .. }, Petal::Child { parent: b_par, .. })
                if a_par == b_par =>
            {
                PetalRelationship::Peer
            }

            _ if self.is_ancestor(a, b) => PetalRelationship::Ancestor,
            _ if self.is_ancestor(b, a) => PetalRelationship::Descendant,

            _ => PetalRelationship::Unknown,
        }
    }

    pub fn accessible(&self, definition: &Definition) -> bool {
        let Some(petal_id) = definition.petal else {
            return true;
        };

        let current = self.expect_top_id();
        let relationship = self.relationship(current, petal_id);

        use PetalRelationship::*;
        let private_match = matches!(relationship, This | Child | Descendant);

        match definition.accessibility {
            DefAccessibility::Priv => private_match,
            DefAccessibility::Pub(kind) => match kind {
                DefPubAccessibilityKind::This => private_match,
                DefPubAccessibilityKind::Super => {
                    private_match || matches!(relationship, Peer | Super)
                }
                DefPubAccessibilityKind::Spathe => {
                    private_match || matches!(relationship, Peer | Super | Spathe | Ancestor)
                }
                DefPubAccessibilityKind::All => true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PetalId(usize);

impl PetalId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "petal id is not valid!");
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PetalRelationship {
    Unknown,
    This,
    Peer,
    Child,
    Descendant,
    Super,
    Ancestor,
    Spathe, // TODO
}

#[derive(Debug, Clone)]
pub enum Petal {
    Root(ScopeId),
    Child { parent: PetalId, def_id: DefId },
}

impl Petal {
    pub fn scope_id(&self, scope_ctx: &ScopeCtx) -> ScopeId {
        match self {
            Self::Root(scope_id) => *scope_id,
            Self::Child { def_id, .. } => scope_ctx
                .get_id_from_def(*def_id)
                .expect("expected a scope attached to the definition of a child petal!"),
        }
    }
}
