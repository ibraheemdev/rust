use clippy_utils::diagnostics::span_lint_and_sugg;
use hir::{Node, PatKind};
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_clippy_lint! {
    /// ### What it does
    /// Checks for usage of `_` in patterns of type `()`.
    ///
    /// ### Why is this bad?
    /// Matching with `()` explicitly instead of `_` outlines
    /// the fact that the pattern contains no data. Also it
    /// would detect a type change that `_` would ignore.
    ///
    /// ### Example
    /// ```rust
    /// match std::fs::create_dir("tmp-work-dir") {
    ///    Ok(_) => println!("Working directory created"),
    ///    Err(s) => eprintln!("Could not create directory: {s}"),
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// match std::fs::create_dir("tmp-work-dir") {
    ///    Ok(()) => println!("Working directory created"),
    ///    Err(s) => eprintln!("Could not create directory: {s}"),
    /// }
    /// ```
    #[clippy::version = "1.73.0"]
    pub IGNORED_UNIT_PATTERNS,
    pedantic,
    "suggest replacing `_` by `()` in patterns where appropriate"
}
declare_lint_pass!(IgnoredUnitPatterns => [IGNORED_UNIT_PATTERNS]);

impl<'tcx> LateLintPass<'tcx> for IgnoredUnitPatterns {
    fn check_pat(&mut self, cx: &LateContext<'tcx>, pat: &'tcx hir::Pat<'tcx>) {
        match cx.tcx.hir().get_parent(pat.hir_id) {
            Node::Param(param) if matches!(cx.tcx.hir().get_parent(param.hir_id), Node::Item(_)) => {
                // Ignore function parameters
                return;
            },
            Node::Local(local) if local.ty.is_some() => {
                // Ignore let bindings with explicit type
                return;
            },
            _ => {},
        }
        if matches!(pat.kind, PatKind::Wild) && cx.typeck_results().pat_ty(pat).is_unit() {
            span_lint_and_sugg(
                cx,
                IGNORED_UNIT_PATTERNS,
                pat.span,
                "matching over `()` is more explicit",
                "use `()` instead of `_`",
                String::from("()"),
                Applicability::MachineApplicable,
            );
        }
    }
}
