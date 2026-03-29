mod constants;
pub mod generated_mappings;

use super::{
    DisregardedSlotCondition, GritTargetLanguageImpl, LeafEquivalenceClass, LeafNormalizer,
    normalize_quoted_string,
};
use crate::{
    CompileError,
    grit_target_node::{GritTargetNode, GritTargetSyntaxKind},
};
use biome_css_syntax::{CssLanguage, CssSyntaxKind};
use biome_rowan::{RawSyntaxKind, SyntaxKindSet};
use constants::DISREGARDED_SNIPPET_SLOTS;
use generated_mappings::kind_by_name;

const COMMENT_KINDS: SyntaxKindSet<CssLanguage> =
    SyntaxKindSet::from_raw(RawSyntaxKind(CssSyntaxKind::COMMENT as u16)).union(
        SyntaxKindSet::from_raw(RawSyntaxKind(CssSyntaxKind::MULTILINE_COMMENT as u16)),
    );

const EQUIVALENT_LEAF_NODES: &[&[LeafNormalizer]] = &[&[LeafNormalizer::new(
    GritTargetSyntaxKind::CssSyntaxKind(CssSyntaxKind::CSS_STRING_LITERAL),
    normalize_quoted_string,
)]];

const CSS_CLASS_SELECTOR_SLOTS: &[(&str, u32)] = &[("dot_token", 0), ("name", 1)];
const CSS_DECLARATION_SLOTS: &[(&str, u32)] = &[("property", 0), ("important", 1)];
const CSS_GENERIC_PROPERTY_SLOTS: &[(&str, u32)] = &[("name", 0), ("colon_token", 1), ("value", 2)];
const CSS_ID_SELECTOR_SLOTS: &[(&str, u32)] = &[("hash_token", 0), ("name", 1)];
const CSS_TYPE_SELECTOR_SLOTS: &[(&str, u32)] = &[("namespace", 0), ("ident", 1)];

fn native_name_for_kind(kind: CssSyntaxKind) -> Option<&'static str> {
    match kind {
        CssSyntaxKind::CSS_CLASS_SELECTOR => Some("CssClassSelector"),
        CssSyntaxKind::CSS_DECLARATION => Some("CssDeclaration"),
        CssSyntaxKind::CSS_GENERIC_PROPERTY => Some("CssGenericProperty"),
        CssSyntaxKind::CSS_ID_SELECTOR => Some("CssIdSelector"),
        CssSyntaxKind::CSS_TYPE_SELECTOR => Some("CssTypeSelector"),
        _ => None,
    }
}

fn native_slots_for_kind(kind: CssSyntaxKind) -> &'static [(&'static str, u32)] {
    match kind {
        CssSyntaxKind::CSS_CLASS_SELECTOR => CSS_CLASS_SELECTOR_SLOTS,
        CssSyntaxKind::CSS_DECLARATION => CSS_DECLARATION_SLOTS,
        CssSyntaxKind::CSS_GENERIC_PROPERTY => CSS_GENERIC_PROPERTY_SLOTS,
        CssSyntaxKind::CSS_ID_SELECTOR => CSS_ID_SELECTOR_SLOTS,
        CssSyntaxKind::CSS_TYPE_SELECTOR => CSS_TYPE_SELECTOR_SLOTS,
        _ => &[],
    }
}

#[derive(Clone, Debug)]
pub struct CssTargetLanguage;

impl GritTargetLanguageImpl for CssTargetLanguage {
    type Kind = CssSyntaxKind;

    /// Returns the syntax kind for a node by name.
    ///
    /// Supports native Biome AST patterns for full language coverage.
    fn kind_by_name(&self, node_name: &str) -> Option<CssSyntaxKind> {
        kind_by_name(node_name)
    }

    /// Returns the node name for a given syntax kind.
    ///
    /// For compatibility with existing Grit snippets (as well as the online
    /// Grit playground), node names should be aligned with TreeSitter's
    /// `ts_language_symbol_name()`.
    fn name_for_kind(&self, kind: GritTargetSyntaxKind) -> &'static str {
        let Some(kind) = kind.as_css_kind() else {
            return "(unexpected language)";
        };
        native_name_for_kind(kind).unwrap_or("(unknown node)")
    }

    /// Returns the slots with their names for the given node kind.
    ///
    /// For compatibility with existing Grit snippets (as well as the online
    /// Grit playground), node names should be aligned with TreeSitter's
    /// `ts_language_field_name_for_id()`.
    fn named_slots_for_kind(&self, kind: GritTargetSyntaxKind) -> &'static [(&'static str, u32)] {
        let Some(kind) = kind.as_css_kind() else {
            return &[];
        };
        native_slots_for_kind(kind)
    }

    fn snippet_context_strings(&self) -> &[(&'static str, &'static str)] {
        &[
            ("", ""),
            ("GRIT_BLOCK { ", " }"),
            ("GRIT_BLOCK { GRIT_PROPERTY: ", " }"),
        ]
    }

    fn is_comment_kind(kind: GritTargetSyntaxKind) -> bool {
        kind.as_css_kind()
            .is_some_and(|kind| COMMENT_KINDS.matches(kind))
    }

    fn metavariable_kind() -> Self::Kind {
        CssSyntaxKind::CSS_METAVARIABLE
    }

    fn is_disregarded_snippet_field(
        &self,
        kind: GritTargetSyntaxKind,
        slot_index: u32,
        node: Option<GritTargetNode<'_>>,
    ) -> bool {
        DISREGARDED_SNIPPET_SLOTS.iter().any(
            |(disregarded_kind, disregarded_slot_index, condition)| {
                if GritTargetSyntaxKind::from(*disregarded_kind) != kind
                    || *disregarded_slot_index != slot_index
                {
                    return false;
                }

                match condition {
                    DisregardedSlotCondition::Always => true,
                    DisregardedSlotCondition::OnlyIf(node_texts) => node_texts.iter().any(|text| {
                        *text == node.as_ref().map(|node| node.text()).unwrap_or_default()
                    }),
                }
            },
        )
    }

    fn get_equivalence_class(
        &self,
        kind: GritTargetSyntaxKind,
        text: &str,
    ) -> Result<Option<LeafEquivalenceClass>, CompileError> {
        if let Some(class) = EQUIVALENT_LEAF_NODES
            .iter()
            .find(|v| v.iter().any(|normalizer| normalizer.kind() == kind))
        {
            LeafEquivalenceClass::new(text, kind, class)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CSS_CLASS_SELECTOR_SLOTS, CSS_DECLARATION_SLOTS, CSS_GENERIC_PROPERTY_SLOTS,
        CSS_ID_SELECTOR_SLOTS, CSS_TYPE_SELECTOR_SLOTS, CssTargetLanguage,
    };
    use crate::grit_target_language::GritTargetLanguageImpl;
    use crate::grit_target_node::GritTargetSyntaxKind;
    use biome_css_syntax::CssSyntaxKind;

    #[test]
    fn returns_native_names_for_supported_css_nodes() {
        let language = CssTargetLanguage;

        assert_eq!(
            language.name_for_kind(GritTargetSyntaxKind::from(
                CssSyntaxKind::CSS_CLASS_SELECTOR
            )),
            "CssClassSelector"
        );
        assert_eq!(
            language.name_for_kind(GritTargetSyntaxKind::from(CssSyntaxKind::CSS_DECLARATION)),
            "CssDeclaration"
        );
        assert_eq!(
            language.name_for_kind(GritTargetSyntaxKind::from(
                CssSyntaxKind::CSS_GENERIC_PROPERTY
            )),
            "CssGenericProperty"
        );
        assert_eq!(
            language.name_for_kind(GritTargetSyntaxKind::from(CssSyntaxKind::CSS_ID_SELECTOR)),
            "CssIdSelector"
        );
        assert_eq!(
            language.name_for_kind(GritTargetSyntaxKind::from(CssSyntaxKind::CSS_TYPE_SELECTOR)),
            "CssTypeSelector"
        );
    }

    #[test]
    fn returns_named_slots_for_supported_css_nodes() {
        let language = CssTargetLanguage;

        assert_eq!(
            language.named_slots_for_kind(GritTargetSyntaxKind::from(
                CssSyntaxKind::CSS_CLASS_SELECTOR
            )),
            CSS_CLASS_SELECTOR_SLOTS
        );
        assert_eq!(
            language
                .named_slots_for_kind(GritTargetSyntaxKind::from(CssSyntaxKind::CSS_DECLARATION)),
            CSS_DECLARATION_SLOTS
        );
        assert_eq!(
            language.named_slots_for_kind(GritTargetSyntaxKind::from(
                CssSyntaxKind::CSS_GENERIC_PROPERTY
            )),
            CSS_GENERIC_PROPERTY_SLOTS
        );
        assert_eq!(
            language
                .named_slots_for_kind(GritTargetSyntaxKind::from(CssSyntaxKind::CSS_ID_SELECTOR)),
            CSS_ID_SELECTOR_SLOTS
        );
        assert_eq!(
            language
                .named_slots_for_kind(GritTargetSyntaxKind::from(CssSyntaxKind::CSS_TYPE_SELECTOR)),
            CSS_TYPE_SELECTOR_SLOTS
        );
    }
}
