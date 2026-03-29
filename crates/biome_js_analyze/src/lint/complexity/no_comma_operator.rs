use biome_analyze::context::RuleContext;
use biome_analyze::{Ast, Rule, RuleDiagnostic, RuleSource, declare_lint_rule};
use biome_diagnostics::Severity;
use biome_js_syntax::{
    JsFileSource, JsForStatement, JsInExpression, JsParenthesizedExpression, JsSequenceExpression,
};
use biome_rowan::AstNode;
use biome_rule_options::no_comma_operator::NoCommaOperatorOptions;

declare_lint_rule! {
    /// Disallow comma operator.
    ///
    /// The comma operator includes multiple expressions where only one is expected.
    /// It evaluates every operand from left to right and returns the value of the last operand.
    /// It frequently obscures side effects, and its use is often an accident.
    ///
    /// The use of the comma operator in the initialization and update parts of a `for` is still allowed.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// const foo = (doSomething(), 0);
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// for (; doSomething(), !!test; ) {}
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// // Use a semicolon instead.
    /// let a, b;
    /// a = 1, b = 2;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```js
    /// for(a = 0, b = 0; (a + b) < 10; a++, b += 2) {}
    /// ```
    ///
    pub NoCommaOperator {
        version: "1.0.0",
        name: "noCommaOperator",
        language: "js",
        sources: &[RuleSource::Eslint("no-sequences").same()],
        recommended: true,
        severity: Severity::Warning,
    }
}

impl Rule for NoCommaOperator {
    type Query = Ast<JsSequenceExpression>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = NoCommaOperatorOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let seq = ctx.query();
        // Ignore nested sequences
        if seq.parent::<JsSequenceExpression>().is_some() {
            return None;
        }
        if let Some(for_stmt) = seq.parent::<JsForStatement>() {
            // Allow comma operator in initializer and update parts of a `for`
            if for_stmt.test().map(AstNode::into_syntax).as_ref() != Some(seq.syntax()) {
                return None;
            }
        }

        let file_source = ctx.source_type::<JsFileSource>();
        if file_source.is_template_expression()
            && file_source.as_embedding_kind().is_vue()
            && is_vue_v_for_alias(seq)
        {
            return None;
        }

        Some(())
    }

    fn diagnostic(ctx: &RuleContext<Self>, _: &Self::State) -> Option<RuleDiagnostic> {
        let seq = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                seq.comma_token().ok()?.text_trimmed_range(),
                "The comma operator is disallowed.",
            )
            .note("Its use is often confusing and obscures side effects."),
        )
    }
}

fn is_vue_v_for_alias(seq: &JsSequenceExpression) -> bool {
    if let Some(in_expr) = seq.parent::<JsInExpression>() {
        return in_expr
            .property()
            .ok()
            .and_then(|property| property.as_any_js_expression().cloned())
            .is_some_and(|property| property.syntax() == seq.syntax());
    }

    let Some(parenthesized) = seq.parent::<JsParenthesizedExpression>() else {
        return false;
    };
    let Some(in_expr) = parenthesized.parent::<JsInExpression>() else {
        return false;
    };

    in_expr
        .property()
        .ok()
        .and_then(|property| property.as_any_js_expression().cloned())
        .is_some_and(|property| property.syntax() == parenthesized.syntax())
}
