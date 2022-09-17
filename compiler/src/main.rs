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

#[test]
fn test_statements() {
    if let Statement::For(ForStatement { pre, cond: Expression::Literal(Literal::Int(1)), body}) = tl::StatementParser::new().parse("for ( a = 2 ; 1 ; ) {}").unwrap() {
        assert!(matches!(*pre, 
            Statement::Assignment(
                Identifier { id: "a" }, 
                Expression::Literal(
                    Literal::Int(2)))));
        assert!(body.is_empty());
    } else {
        panic!();
    }
    
    if let Statement::For(ForStatement { pre, cond: Expression::Literal(Literal::Int(1)), body }) = tl::StatementParser::new().parse("for (; 1 ;) {}").unwrap() {
        assert!(matches!(*pre, Statement::Empty()));
        assert!(body.is_empty());
    } else {
        panic!();
    }

    assert!(matches!(tl::StatementParser::new().parse("word a;").unwrap(), Statement::Declaration(Identifier { id: "a" })));
    assert!(matches!(tl::StatementParser::new().parse("a = 1;").unwrap(), Statement::Assignment(Identifier { id: "a" }, Expression::Literal(Literal::Int(1)))));

    
}


fn main() {
    let s: &str = include_str!("../sample_programs/main.tl");
    
}
