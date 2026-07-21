use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{
        DefAccessibility, DefKind, DefPubAccessibilityKind, Definition, FnDef, StructDef,
        StructFieldDef,
    },
    item::{HirItem, HirItemKind, HirPetalKind},
    petal::PetalId,
    scope::Scope,
};
use hycc_span::Span;
use hycc_ty::ty::Ty;
use hycc_util::{bug, ternary};

use crate::collector::{CollectResult, CollectionLevel, Collector};

impl<'i> Collector<'i> {
    pub fn collect_item(&mut self, item: &HirItem) -> CollectResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(_) => self.collect_petal(&item),
            HirItemKind::Struct(_) => self.collect_struct(&item),
            HirItemKind::Fn(_) => self.collect_fn(&item),
            HirItemKind::VarDecl(_) => self.collect_var(&item),
        }
    }

    pub fn define_spathe_at_current_petal(&mut self) {
        let root_id = self.petal_ctx.root_petal_id();
        let root_scope_id = self.petal_ctx.get(root_id).scope_id(&self.scope_ctx);

        let spathe_def = Definition::new(
            self.interner.intern("spathe"),
            DefKind::Petal,
            None,
            HirId::Invalid,
            Span::default(),
            DefAccessibility::Pub(DefPubAccessibilityKind::This),
        );
        let spathe_def_id = self.define(spathe_def).unwrap();

        self.scope_ctx
            .attach_id_to_def(spathe_def_id, root_scope_id);
        self.petal_ctx.attach_petal_id(spathe_def_id, root_id);
    }

    pub fn define_super_at_current_petal(&mut self) {
        let Some(super_id) = self.petal_ctx.from_top_id(1) else {
            return;
        };

        let super_scope_id = self.petal_ctx.get(super_id).scope_id(&self.scope_ctx);

        let super_def = Definition::new(
            self.interner.intern("super"),
            DefKind::Petal,
            None,
            HirId::Invalid,
            Span::default(),
            DefAccessibility::Pub(DefPubAccessibilityKind::This),
        );
        let super_def_id = self.define(super_def).unwrap();

        self.scope_ctx
            .attach_id_to_def(super_def_id, super_scope_id);
        self.petal_ctx.attach_petal_id(super_def_id, super_id);
    }

    pub fn collect_petal(&mut self, petal_item: &HirItem) -> CollectResult {
        let HirItemKind::Petal(petal) = &petal_item.kind else {
            unreachable!();
        };

        let path = match &petal.kind {
            HirPetalKind::File(path) | HirPetalKind::Inline(path) => Some(path),
            _ => None,
        };

        let (mut scopes, mut petals) = (0, 0);
        let expected = self.is_expected_to_be_collected();

        if let Some(path) = path {
            for segment in &path.segments {
                let def_id = ternary!(
                    expected,
                    if let Some(def_id) = self.definitions.get_def_id(segment.id) {
                        *def_id
                    } else {
                        return Ok(());
                    },
                    {
                        let def = Definition::new(
                            segment.ident.ident,
                            DefKind::Petal,
                            Some(self.petal_ctx.top_id()),
                            segment.id,
                            petal_item.span,
                            petal_item.accessibility,
                        );

                        ternary!(petal.is_inline(), self.try_define(def), self.define(def))?
                    }
                );

                self.definitions.define_id_hir(segment.id, def_id);

                let petal_id = self.petal_ctx.try_create_child_petal(def_id);
                self.petal_ctx.push(petal_id);

                let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
                self.scope_ctx.push_id(scope_id);

                (scopes += 1, petals += 1);

                if !expected {
                    self.define_spathe_at_current_petal();
                    self.define_super_at_current_petal();
                }
            }
        }

        for item in &petal.items {
            if let Err(Some(diag)) = self.collect_item(&item) {
                self.dctx.add(diag);
            }
        }

        loop {
            if scopes > 0 {
                (self.scope_ctx.pop(), scopes -= 1);
                continue;
            }

            if petals > 0 {
                (self.petal_ctx.pop(), petals -= 1);
                continue;
            }

            break;
        }

        Ok(())
    }

    pub fn collect_struct(&mut self, struct_item: &HirItem) -> CollectResult {
        if self.is_expected_to_be_collected() {
            return Ok(());
        }

        let HirItemKind::Struct(strct) = &struct_item.kind else {
            bug!("item is ensured to be a struct")
        };

        let def_id = self.define(Definition::new(
            strct.ident.ident,
            DefKind::Struct(Box::new(StructDef::new())),
            Some(self.petal_ctx.top_id()),
            struct_item.id,
            struct_item.span,
            struct_item.accessibility,
        ))?;

        self.scope_ctx.try_attach_to_def(def_id, Scope::new());

        let ty = Ty::new(self.tctx.make_adt_ty(def_id), struct_item.span);
        self.tctx.attach_to_hir(struct_item.id, ty);

        let DefKind::Struct(def) = &mut self.definitions.get_mut(def_id).kind else {
            bug!("struct definition is expected to be defined after definition")
        };

        for field in &strct.fields.list {
            let name = field.ident.ident;
            if let Some(idx) = def.field_map.get(&name) {
                todo!("throw error: duplication: {idx:?}")
            };

            def.field_map.insert(name, def.fields.len());
            def.fields.push(StructFieldDef {
                name,
                accessibility: field.accessibility,
                span: field.span,
                ty: field.ty.id,
            });
        }

        Ok(())
    }

    pub fn collect_fn(&mut self, fn_item: &HirItem) -> CollectResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let expected = self.is_expected_to_be_collected();
        let def_id = ternary!(
            expected,
            if let Some(def_id) = self.definitions.get_def_id(fn_item.id) {
                *def_id
            } else {
                return Ok(());
            },
            self.define(Definition::new(
                func.ident.ident,
                DefKind::Fn(Box::new(FnDef::new(func.ret_ty.map(|ty| ty.id)))),
                Some(self.petal_ctx.top_id()),
                fn_item.id,
                fn_item.span,
                fn_item.accessibility,
            ))?
        );

        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        self.enter_scope(scope_id, CollectionLevel::Local, |s| {
            if expected {
                return;
            }

            // Define the function parameters
            for param in &func.params.list {
                let res = s.define(Definition::new(
                    param.ident.ident,
                    DefKind::FnParam,
                    Some(s.petal_ctx.top_id()),
                    param.id,
                    param.span,
                    DefAccessibility::Priv,
                ));

                match res {
                    Ok(def_id) => {
                        if let DefKind::Fn(def) = &mut s.definitions.get_mut(def_id).kind {
                            def.params.push(def_id)
                        }
                    }
                    Err(Some(diag)) => {
                        s.dctx.add(diag);
                    }
                    _ => {}
                };
            }
        });

        Ok(())
    }

    pub fn collect_var(&mut self, var_item: &HirItem) -> CollectResult {
        let HirItemKind::VarDecl(var) = &var_item.kind else {
            unreachable!()
        };

        if !self.is_expected_to_be_collected() {
            self.define(Definition::new(
                var.ident.ident,
                DefKind::Var,
                Some(self.petal_ctx.top_id()),
                var_item.id,
                var_item.span,
                var_item.accessibility,
            ))?;
        }

        Ok(())
    }
}
