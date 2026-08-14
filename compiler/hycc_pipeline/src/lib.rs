use std::{collections::HashMap, fs};

use hycc_ast::item::{ItemKind, Petal, PetalKind};
// use hycc_collect::{collector::Collector, diag::CollectorDiagDataCtx};
use hycc_const::table::ConstTable;
use hycc_diagnostic::{
    DiagnosticContext,
    reporter::{CLIReporter, DiagnosticReporter},
};
use hycc_hir::{HirTable, builder::HirBuilder, item::HirItem};
// use hycc_infer::{diag::InferDiagDataCtx, inferer::TyInferer};
// use hycc_mir::{body::MirBodyId, builder::MirBuilder};
use hycc_parser::{
    lexer::{Lexer, diag::LexerDiagDataCtx},
    parser::{Parser, diag::ParserDiagDataCtx},
};
use hycc_resolve::{
    diag::ResolverDiagDataCtx,
    resolver::{self, Resolver},
    ty_resolver::TyResolver,
};
use hycc_session::session::Session;
use hycc_source::{Source, SourceRegistry, source::SourceId};
use hycc_util::{bug, ternary};

pub struct Driver {
    registry: SourceRegistry,
}

impl Driver {
    pub fn invoke(root_path: &str) {
        let mut driver = Driver {
            registry: SourceRegistry::new(Source::new(root_path)),
        };

        // TODO: scan the dependencies of the main unit to allow multiple compilation units

        for source_id in driver.registry.sources() {
            let mut session = Session::new(driver.registry.root().0);
            driver.compile(&mut session);

            let registry = &driver.registry;
            let reporter = CLIReporter::new(&session.dctx, &registry);
            reporter.report();
        }
    }

    fn parse(&mut self, session: &mut Session) -> Option<Petal> {
        let source = self.registry.get(session.root);

        let mut lexer = Lexer::new(&source);
        let tok_stream = lexer.tokenize();

        lexer
            .dctx
            .emit(&mut session.dctx, LexerDiagDataCtx::new(&self.registry));

        if session.dctx.error_occurred() {
            return None;
        }

        let mut parser = Parser::new(tok_stream, &source);
        let petal = parser.parse();

        parser
            .dctx
            .emit(&mut session.dctx, ParserDiagDataCtx::new(&self.registry));

        if session.dctx.error_occurred() {
            return None;
        }

        ternary!(parser.dctx.error_occurred(), None, Some(petal))
    }

    fn parse_source(&mut self, session: &mut Session) -> Option<Petal> {
        let Some(mut root_petal) = self.parse(session) else {
            return None;
        };

        self.parse_expand_file_petals(session, &mut root_petal);
        Some(root_petal)
    }

    fn parse_expand_file_petals(&mut self, session: &mut Session, petal: &mut Petal) {
        for item in &mut petal.items {
            let ItemKind::Petal(petal) = &mut item.kind else {
                continue;
            };

            match &mut petal.kind {
                PetalKind::File(_, buf) => {
                    let src_id = self.registry.register(Source::new(buf.to_str().unwrap()));
                    let Some(mut file_petal) = self.parse_source(session) else {
                        continue;
                    };

                    std::mem::swap(&mut file_petal.items, &mut petal.items);
                }

                _ => {
                    self.parse_expand_file_petals(session, petal);
                }
            }
        }
    }

    fn resolve<'s, 'h>(&mut self, session: &'s mut Session<'h>, tree: &HirItem)
    where
        'h: 's,
    {
        let mut resolver = Resolver::new(&mut session.interner);
        resolver.resolve(&tree);

        resolver.dctx.emit(
            &mut session.dctx,
            ResolverDiagDataCtx::new(
                &resolver.collector.interner,
                &session.hir_table,
                &resolver.collector.definitions,
                &resolver.collector.scope_ctx,
            ),
        );

        let mut ty_resolver = TyResolver::new();
    }

    fn compile<'s, 'h>(&mut self, session: &'s mut Session<'h>)
    where
        'h: 's,
    {
        let Some(tree) = self.parse_source(session) else {
            return;
        };

        let mut hir_builder = HirBuilder::new(
            &mut session.interner,
            &self.registry,
            &session.hir_table,
            &mut session.const_table,
        );
        let hir = hir_builder.lower(tree);

        self.resolve(session, hir);

        // let mut collector = Collector::new(&mut session.interner);
        // collector.collect(&hir);

        // let (definitions, scope_ctx) = (&collector.definitions, &collector.scope_ctx);
        // collector.dctx.emit(
        //     &mut session.dctx,
        //     CollectorDiagDataCtx::new(&collector.interner, &hir_table, &definitions, &scope_ctx),
        // );

        // if session.dctx.error_occurred() {
        //     return;
        // }

        // let mut resolver = Resolver::new(&mut collector, &hir_table);
        // resolver.resolve(&hir);

        // let (definitions, scope_ctx) = (
        //     &resolver.collector.definitions,
        //     &resolver.collector.scope_ctx,
        // );

        // resolver.collector.dctx.emit(
        //     &mut session.dctx,
        //     CollectorDiagDataCtx::new(
        //         &mut resolver.collector.interner,
        //         &hir_table,
        //         &definitions,
        //         &scope_ctx,
        //     ),
        // );

        // resolver.dctx.emit(
        //     &mut session.dctx,
        //     ResolverDiagDataCtx::new(
        //         &mut resolver.collector.tctx,
        //         &definitions,
        //         &resolver.collector.interner,
        //     ),
        // );

        // if session.dctx.error_occurred() {
        //     return;
        // }

        // let definitions = &mut collector.definitions;
        // let tctx = &mut collector.tctx;
        // let scope_ctx = &mut collector.scope_ctx;
        // let petal_ctx = &mut collector.petal_ctx;

        // let mut ty_resolver = TyResolver::new(tctx, definitions, scope_ctx, &hir_table);

        // ty_resolver.resolve(&hir);
        // ty_resolver.dctx.emit(
        //     &mut session.dctx,
        //     ResolverDiagDataCtx::new(
        //         &mut ty_resolver.tctx,
        //         &ty_resolver.definitions,
        //         &session.interner,
        //     ),
        // );

        // if session.dctx.error_occurred() {
        //     return;
        // }

        // let mut ty_inferer = TyInferer::new(
        //     ty_resolver.tctx,
        //     definitions,
        //     &const_table,
        //     &hir_table,
        //     &petal_ctx,
        // );

        // ty_inferer.infer(&hir);

        // ty_inferer.dctx.emit(
        //     &mut session.dctx,
        //     InferDiagDataCtx::new(
        //         &mut ty_inferer.tctx,
        //         &ty_inferer.definitions,
        //         &session.interner,
        //     ),
        // );

        // if session.dctx.error_occurred() {
        //     return;
        // }

        // let mut mir_builder = MirBuilder::new(&mut ty_inferer.tctx, &definitions);
        // mir_builder.lower(&hir);

        // // TEMP: for display only
        // let mut bodies = mir_builder.ctx.table.defs().iter().collect::<Vec<_>>();
        // bodies.sort_by(|(_, a_body_id), (_, b_body_id)| a_body_id.unwrap().cmp(&b_body_id.unwrap()));

        // for (def_id, body_id) in bodies {
        //     println!(
        //         "MirBody[{def_id:?}]:\n{}",
        //         mir_builder.ctx.table.get(*body_id)
        //     );
        // }

        // let body_def_map = mir_builder
        //     .ctx
        //     .table
        //     .defs()
        //     .iter()
        //     .map(|(key, value)| (value.unwrap(), key))
        //     .collect::<HashMap<_, _>>();
        // for (i, body) in mir_builder.ctx.table.bodies().iter().enumerate() {
        //     if body_def_map.contains_key(&i) {
        //         continue;
        //     }

        //     println!("MirBody:\n{}", body);
        // }
    }
}
