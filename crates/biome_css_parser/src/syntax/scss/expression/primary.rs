use crate::parser::CssParser;
use crate::syntax::declaration::{is_at_declaration_important, parse_declaration_important};
use crate::syntax::parse_error::expected_component_value;
use crate::syntax::property::parse_generic_component_value;
use crate::syntax::scss::{parse_scss_fallback_value, parse_scss_regular_interpolation};
use biome_css_syntax::CssSyntaxKind::SCSS_UNARY_EXPRESSION;
use biome_css_syntax::T;
use biome_parser::Parser;
use biome_parser::prelude::ParsedSyntax;
use biome_parser::prelude::ParsedSyntax::{Absent, Present};

use super::interpolation::{is_at_scss_interpolation, is_nth_at_scss_interpolation};
use super::map::parse_scss_parenthesized_or_map_expression;

/// Parses SCSS primaries such as parenthesized values, maps, `!important`, and
/// SCSS-only fallback values.
///
/// Example:
/// ```scss
/// foo($x: 1, $y: 2);
/// ```
///
/// Docs: https://sass-lang.com/documentation/at-rules/function
#[inline]
pub(super) fn parse_scss_primary_expression(p: &mut CssParser) -> ParsedSyntax {
    if is_at_scss_interpolation(p) {
        parse_scss_regular_interpolation(p)
    } else if is_at_scss_bang_interpolation_expression(p) {
        parse_scss_bang_interpolation_expression(p)
    } else if p.at(T!['(']) {
        parse_scss_parenthesized_or_map_expression(p)
    } else if is_at_declaration_important(p) {
        parse_declaration_important(p)
    } else {
        let value = parse_generic_component_value(p);
        if value.is_absent() {
            parse_scss_fallback_value(p)
        } else {
            value
        }
    }
}

#[inline]
fn is_at_scss_bang_interpolation_expression(p: &mut CssParser) -> bool {
    p.at(T![!]) && is_nth_at_scss_interpolation(p, 1)
}

#[inline]
fn parse_scss_bang_interpolation_expression(p: &mut CssParser) -> ParsedSyntax {
    if !is_at_scss_bang_interpolation_expression(p) {
        return Absent;
    }

    let m = p.start();
    p.bump(T![!]);
    parse_scss_regular_interpolation(p).or_add_diagnostic(p, expected_component_value);

    Present(m.complete(p, SCSS_UNARY_EXPRESSION))
}
