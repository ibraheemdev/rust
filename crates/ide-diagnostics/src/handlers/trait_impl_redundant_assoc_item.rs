use hir::{db::ExpandDatabase, Const, Function, HasSource, HirDisplay, TypeAlias};
use ide_db::{
    assists::{Assist, AssistId, AssistKind},
    base_db::FileRange,
    label::Label,
    source_change::SourceChangeBuilder,
};
use syntax::{AstNode, SyntaxKind};
use text_edit::TextRange;

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: trait-impl-redundant-assoc_item
//
// Diagnoses redundant trait items in a trait impl.
pub(crate) fn trait_impl_redundant_assoc_item(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::TraitImplRedundantAssocItems,
) -> Diagnostic {
    let db = ctx.sema.db;
    let name = d.assoc_item.0.clone();
    let redundant_assoc_item_name = name.display(db);
    let assoc_item = d.assoc_item.1;

    let default_range = d.impl_.syntax_node_ptr().text_range();
    let trait_name = d.trait_.name(db).to_smol_str();

    let (redundant_item_name, diagnostic_range, redundant_item_def) = match assoc_item {
        hir::AssocItem::Function(id) => {
            let function = Function::from(id);
            (
                format!("`fn {}`", redundant_assoc_item_name),
                function
                    .source(db)
                    .map(|it| it.syntax().value.text_range())
                    .unwrap_or(default_range),
                format!("\n    {};", function.display(db).to_string()),
            )
        }
        hir::AssocItem::Const(id) => {
            let constant = Const::from(id);
            (
                format!("`const {}`", redundant_assoc_item_name),
                constant
                    .source(db)
                    .map(|it| it.syntax().value.text_range())
                    .unwrap_or(default_range),
                format!("\n    {};", constant.display(db).to_string()),
            )
        }
        hir::AssocItem::TypeAlias(id) => {
            let type_alias = TypeAlias::from(id);
            (
                format!("`type {}`", redundant_assoc_item_name),
                type_alias
                    .source(db)
                    .map(|it| it.syntax().value.text_range())
                    .unwrap_or(default_range),
                format!("\n    type {};", type_alias.name(ctx.sema.db).to_smol_str()),
            )
        }
    };

    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0407"),
        format!("{redundant_item_name} is not a member of trait `{trait_name}`"),
        FileRange { file_id: d.file_id.file_id().unwrap(), range: diagnostic_range },
    )
    .with_fixes(quickfix_for_redundant_assoc_item(
        ctx,
        d,
        redundant_item_def,
        diagnostic_range,
    ))
}

/// add assoc item into the trait def body
fn quickfix_for_redundant_assoc_item(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::TraitImplRedundantAssocItems,
    redundant_item_def: String,
    range: TextRange,
) -> Option<Vec<Assist>> {
    let add_assoc_item_def = |builder: &mut SourceChangeBuilder| -> Option<()> {
        let db = ctx.sema.db;
        let root = db.parse_or_expand(d.file_id);
        // don't modify trait def in outer crate
        let current_crate = ctx.sema.scope(&d.impl_.syntax_node_ptr().to_node(&root))?.krate();
        let trait_def_crate = d.trait_.module(db).krate();
        if trait_def_crate != current_crate {
            return None;
        }
        let trait_def = d.trait_.source(db)?.value;
        let where_to_insert = trait_def
            .syntax()
            .descendants_with_tokens()
            .find(|it| it.kind() == SyntaxKind::L_CURLY)
            .map(|it| it.text_range())?;

        Some(builder.insert(where_to_insert.end(), redundant_item_def))
    };
    let file_id = d.file_id.file_id()?;
    let mut source_change_builder = SourceChangeBuilder::new(file_id);
    add_assoc_item_def(&mut source_change_builder)?;

    Some(vec![Assist {
        id: AssistId("add assoc item def into trait def", AssistKind::QuickFix),
        label: Label::new("Add assoc item def into trait def".to_string()),
        group: None,
        target: range,
        source_change: Some(source_change_builder.finish()),
        trigger_signature_help: false,
    }])
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn trait_with_default_value() {
        check_diagnostics(
            r#"
trait Marker {
    const FLAG: bool = false;
    fn boo();
    fn foo () {}
}
struct Foo;
impl Marker for Foo {
    type T = i32;
  //^^^^^^^^^^^^^ error: `type T` is not a member of trait `Marker`

    const FLAG: bool = true;

    fn bar() {}
  //^^^^^^^^^^^ error: `fn bar` is not a member of trait `Marker`

    fn boo() {}
}
            "#,
        )
    }

    #[test]
    fn dont_work_for_negative_impl() {
        check_diagnostics(
            r#"
trait Marker {
    const FLAG: bool = false;
    fn boo();
    fn foo () {}
}
struct Foo;
impl !Marker for Foo {
    type T = i32;
    const FLAG: bool = true;
    fn bar() {}
    fn boo() {}
}
            "#,
        )
    }
}
