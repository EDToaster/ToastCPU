use crate::ast::*;

mod ast;
mod parse;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tl); // synthesized by LALRPOP

#[test]
fn test_literals() {
    assert!(matches!(
        tl::LiteralParser::new().parse("\"\"").unwrap(),
        Literal::String("")
    ));
    
    assert!(matches!(
        tl::LiteralParser::new().parse("\"hello\"").unwrap(),
        Literal::String("hello")
    ));

    assert!(tl::LiteralParser::new().parse("\"hello world").is_err());

    assert!(matches!(
        tl::LiteralParser::new().parse("13").unwrap(),
        Literal::Int(13)
    ));

    assert!(matches!(
        tl::LiteralParser::new().parse("0x0A").unwrap(),
        Literal::Int(0x0A)
    ));

    assert!(matches!(
        tl::LiteralParser::new().parse("0b0101").unwrap(),
        Literal::Int(0b0101)
    ));

    assert!(matches!(
        tl::LiteralParser::new().parse("'2'").unwrap(),
        Literal::Char(50)
    ));

    assert!(tl::LiteralParser::new().parse("'92'").is_err());
}

#[test]
fn test_identifiers() {
    assert!(matches!(tl::IdentifierParser::new().parse("hello").unwrap(), Identifier { id: "hello" }));
    assert!(tl::IdentifierParser::new().parse("2hello").is_err());
}

fn main() {}
