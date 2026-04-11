use hycc_ast::{
    ItemKind,
    item::{Petal, PetalKind},
};
use hycc_collection::{collector::Collector, diag::CollectorDiagDataCtx};
use hycc_diagnostic::{
    DiagnosticContext,
    reporter::{CLIReporter, DiagnosticReporter},
};
use hycc_hir::{HirTable, builder::HirBuilder, item::HirPetal};
use hycc_parser::{
    lexer::{Lexer, diag::LexerDiagDataCtx},
    parser::{Parser, diag::ParserDiagDataCtx},
};
use hycc_session::{session::Session, unit::CompilationUnitId};
use hycc_source::{Source, source::SourceId};
use hycc_util::ternary;

pub fn invoke(root_path: &str) {
    let mut session = Session::new();

    // TODO: scan the dependencies of the main unit to allow multiple compilation units

    let unit_id = session.create_unit(Source::new(root_path));
    compile(&mut session, unit_id);

    let reporter = CLIReporter::new(&session.dctx, &session.registry);
    reporter.report();
}

pub fn parse(session: &mut Session, src_id: SourceId) -> Option<Petal> {
    let source = session.registry.get(src_id);

    let mut lexer = Lexer::new(&source);
    let tok_stream = lexer.tokenize();

    lexer
        .dctx
        .emit(&mut session.dctx, LexerDiagDataCtx::new(&session.registry));

    if session.dctx.error_occurred() {
        return None;
    }

    let mut parser = Parser::new(tok_stream, &source);
    let petal = parser.parse();

    parser
        .dctx
        .emit(&mut session.dctx, ParserDiagDataCtx::new(&session.registry));

    if session.dctx.error_occurred() {
        return None;
    }

    ternary!(parser.dctx.error_occurred(), None, Some(petal))
}

pub fn parse_source(session: &mut Session, src_id: SourceId) -> Option<Petal> {
    let Some(mut root_petal) = parse(session, src_id) else {
        return None;
    };

    for item in &mut root_petal.items {
        let ItemKind::Petal(petal) = &mut item.kind else {
            continue;
        };

        let PetalKind::File(_, buf) = &mut petal.kind else {
            continue;
        };

        let src_id = session
            .registry
            .register(Source::new(buf.to_str().unwrap()));

        let Some(mut file_petal) = parse_source(session, src_id) else {
            continue;
        };

        std::mem::swap(&mut file_petal.items, &mut petal.items);
    }

    Some(root_petal)
}

pub fn lower_hir<'h>(session: &mut Session, tree: Petal) -> (HirTable<'h>, &'h HirPetal<'h>) {
    let hir_table = HirTable::new();
    let mut hir_builder =
        HirBuilder::new(&mut session.interner, session.registry.root(), &hir_table);

    let hir = hir_builder.lower(tree);
    (hir_table, hir)
}

pub fn compile(session: &mut Session, unit_id: CompilationUnitId) {
    let unit = session.get_unit(unit_id);
    let Some(tree) = parse_source(session, unit.root) else {
        return;
    };

    let (hir_table, hir) = lower_hir(session, tree);
    let mut collector = Collector::new(&hir_table);

    collector.collect(hir);

    let (definitions, scope_ctx) = (&collector.definitions, &collector.scope_ctx);

    collector.dctx.emit(
        &mut session.dctx,
        CollectorDiagDataCtx::new(&session.interner, &hir_table, &definitions, &scope_ctx),
    );
}
