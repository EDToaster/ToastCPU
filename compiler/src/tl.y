%start Module
%%

Module -> Result<Module, ()>:
    FunctionList { Ok(Module { span: $span, functions: $1? }) }
    ;

FunctionList -> Result<Vec<Function>, ()>:
    { Ok(vec![]) }
    | FunctionList Function { let mut v = $1?; v.push($2?); Ok(v) }
    ;

Function -> Result<Function, ()>:
    'FN' Identifier Typelist 'RARROW' Typelist Block {
        Ok(
            Function {
                span: $span,
                name: $2?,
                in_t: $3?,
                out_t: $5?,
                body: $6?,
            }
        )
    }
    ;

Typelist -> Result<Vec<Identifier>, ()>:
    { Ok(vec![]) }
    | Typelist Identifier { let mut v = $1?; v.push($2?); Ok(v) }
    ;

Block -> Result<Block, ()>:
    'LB' Statements 'RB' { Ok(Block { span: $span, body: $2? }) }
    ;

Statements -> Result<Vec<Statement>, ()>:
    { Ok(vec![]) }
    | Statements Statement { let mut v = $1?; v.push($2?); Ok(v) }
    ;

Statement -> Result<Statement, ()>:
    IntLiteral { Ok(Statement::IntLiteral($1?)) }
    | Identifier { Ok(Statement::Identifier($1?)) }
    | Operator { Ok(Statement::Operator($1?)) }
    | Block { Ok(Statement::Block($1?)) }
    ;

Operator -> Result<Operator, ()>:
    'ADD' { Ok(Operator::Add($span)) }
    | 'SUB' { Ok(Operator::Sub($span)) }
    ;

Identifier -> Result<Identifier, ()>:
    'IDENT' {
            let v = $1.map_err(|_| ())?;
            Ok(Identifier { span: v.span(), name: $lexer.span_str(v.span()).to_string() })
        }
    ;

IntLiteral -> Result<IntLiteral, ()>:
    'DEC_INT' {
            let v = $1.map_err(|_| ())?;
            Ok(IntLiteral { span: v.span(), val: dec_int($lexer.span_str(v.span()))? })
        }
    | 'HEX_INT' {
            let v = $1.map_err(|_| ())?;
            Ok(IntLiteral { span: v.span(), val: hex_int($lexer.span_str(v.span()))? })
        }
    | 'BIN_INT' {
            let v = $1.map_err(|_| ())?;
            Ok(IntLiteral { span: v.span(), val: bin_int($lexer.span_str(v.span()))? })
        }
    ;

%%

use lrpar::Span;

#[derive(Debug)]
pub enum Operator {
    Add(Span),
    Sub(Span),
}

#[derive(Debug)]
pub struct IntLiteral {
    pub span: Span,
    pub val: isize,
}

#[derive(Debug)]
pub struct Identifier {
    pub span: Span,
    pub name: String,
}

#[derive(Debug)]
pub enum Statement {
    IntLiteral(IntLiteral),
    Identifier(Identifier),
    Operator(Operator),
    Block(Block),
}

#[derive(Debug)]
pub struct Block {
    pub span: Span,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Function {
    pub span: Span,
    pub name: Identifier,
    pub in_t: Vec<Identifier>,
    pub out_t: Vec<Identifier>,
    pub body: Block,
}

#[derive(Debug)]
pub struct Module {
    pub span: Span,
    pub functions: Vec<Function>,
}

// Any functions here are in scope for all the grammar actions above.
pub fn dec_int(s: &str) -> Result<isize, ()> {
    s.parse::<isize>().map_err(|e| ())
}

pub fn bin_int(s: &str) -> Result<isize, ()> {
    isize::from_str_radix(&s[2..], 2).map_err(|e| ())
}

pub fn hex_int(s: &str) -> Result<isize, ()> {
    isize::from_str_radix(&s[2..], 16).map_err(|e| ())
}
