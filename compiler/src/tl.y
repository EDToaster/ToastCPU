%start Module
%%

Module -> Result<Module, ()>:
    { Ok(Module { span: $span, globals: vec![], functions: vec![] }) }
    | Module Function { let mut m = $1?; m.functions.push($2?); Ok(m) }
    | Module Global { let mut m = $1?; m.globals.push($2?); Ok(m) }
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

Global -> Result<Global, ()>:
    'GLOBAL' Identifier Identifier IntLiteral { Ok(Global { span: $span, name: $2?, var_type: $3?, val: $4? }) }
    ;

Typelist -> Result<Vec<Identifier>, ()>:
    { Ok(vec![]) }
    | Typelist Identifier { let mut v = $1?; v.push($2?); Ok(v) }
    ;

Block -> Result<Block, ()>:
    'LB' Statements 'RB' { Ok(Block { span: $span, body: $2? }) }
    ;

Unroll -> Result<Unroll, ()>:
    'UNROLL' IntLiteral Block { Ok(Unroll { span: $span, times: $2?, body: $3? }) }
    ;

If -> Result<If, ()>:
    'IF' Block 'ELSE' Block { Ok(If { span: $span, if_block: $2?, else_block: Some($4?) }) }
    | 'IF' Block { Ok(If { span: $span, if_block: $2?, else_block: None }) }
    ;

While -> Result<While, ()>:
    'WHILE' Block Block { Ok(While { span: $span, eval: $2?, body: $3? }) }
    ;

Let -> Result<Let, ()>:
    'LET' Identifiers Block { Ok(Let { span: $span, bindings: $2?, body: $3? }) }
    ;

Statements -> Result<Vec<Statement>, ()>:
    { Ok(vec![]) }
    | Statements Statement { let mut v = $1?; v.push($2?); Ok(v) }
    ;

Statement -> Result<Statement, ()>:
    IntLiteral { Ok(Statement::IntLiteral($1?)) }
    | IntArray { Ok(Statement::IntArray($1?)) }
    | Identifier { Ok(Statement::Identifier($1?)) }
    | Operator { Ok(Statement::Operator($1?)) }
    | Block { Ok(Statement::Block($1?)) }
    | Unroll { Ok(Statement::Unroll($1?)) }
    | If { Ok(Statement::If($1?)) }
    | While { Ok(Statement::While($1?)) }
    | Let { Ok(Statement::Let($1?)) }
    ;

Operator -> Result<Operator, ()>:
    'ADD' { Ok(Operator::Add($span)) }
    | 'SUB' { Ok(Operator::Sub($span)) }
    | 'BOR' { Ok(Operator::BOr($span)) }
    | 'BAND' { Ok(Operator::BAnd($span)) }
    | 'BNOT' { Ok(Operator::BNot($span)) }
    | 'SSHR' { Ok(Operator::Sshr($span)) }
    | 'SHR' { Ok(Operator::Shr($span)) }
    | 'SHL' { Ok(Operator::Shl($span)) }
    | 'XOR' { Ok(Operator::Xor($span)) }
    | 'LOR' { Ok(Operator::LOr($span)) }
    | 'LAND' { Ok(Operator::LAnd($span)) }
    | 'LNOT' { Ok(Operator::LNot($span)) }
    | 'EQ' { Ok(Operator::Eq($span)) }
    | 'LT' { Ok(Operator::Lt($span)) }
    | 'LTE' { Ok(Operator::Lte($span)) }
    | 'GT' { Ok(Operator::Gt($span)) }
    | 'GTE' { Ok(Operator::Gte($span)) }
    | 'HOLE' { Ok(Operator::Hole($span)) }
    | 'AS' 'LP' Identifier 'RP' { Ok(Operator::As($span, $3?)) }
    ;

Identifiers -> Result<Vec<Identifier>, ()>:
    { Ok(vec![]) }
    | Identifiers Identifier { let mut v = $1?; v.push($2?); Ok(v) }
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
    | 'CHAR' {
            let v = $1.map_err(|_| ())?;
            Ok(IntLiteral { span: v.span(), val: char_int($lexer.span_str(v.span()))? })
        }
    ;

IntArray -> Result<IntArray, ()>:
    'STRING' {
            let v = $1.map_err(|_| ())?;
            Ok(IntArray { span: v.span(), val: string_int_arr($lexer.span_str(v.span()))? })
        }
    ;

%%

use lrpar::Span;

#[derive(Debug)]
pub enum Operator {
    Add(Span),
    Sub(Span),
    BOr(Span),
    BAnd(Span),
    BNot(Span),
    Sshr(Span),
    Shr(Span),
    Shl(Span),
    Xor(Span),

    LOr(Span),
    LAnd(Span),
    LNot(Span),

    Eq(Span),
    Lt(Span),
    Lte(Span),
    Gt(Span),
    Gte(Span),

    As(Span, Identifier),

    Hole(Span),
}

#[derive(Debug)]
pub struct IntLiteral {
    pub span: Span,
    pub val: isize,
}

#[derive(Debug)]
pub struct IntArray {
    pub span: Span,
    pub val: Vec<u16>,
}

#[derive(Debug)]
pub struct Identifier {
    pub span: Span,
    pub name: String,
}

#[derive(Debug)]
pub enum Statement {
    IntLiteral(IntLiteral),
    IntArray(IntArray),
    Identifier(Identifier),
    Operator(Operator),
    Block(Block),
    Unroll(Unroll),
    If(If),
    While(While),
    Let(Let),
}

#[derive(Debug)]
pub struct Unroll {
    pub span: Span,
    pub times: IntLiteral,
    pub body: Block,
}

#[derive(Debug)]
pub struct If {
    pub span: Span,
    pub if_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug)]
pub struct While {
    pub span: Span,
    pub eval: Block,
    pub body: Block,
}

#[derive(Debug)]
pub struct Let {
    pub span: Span,
    pub bindings: Vec<Identifier>,
    pub body: Block,
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
pub struct Global {
    pub span: Span,
    pub name: Identifier,
    pub var_type: Identifier,
    pub val: IntLiteral,
}

#[derive(Debug)]
pub struct Module {
    pub span: Span,
    pub globals: Vec<Global>,
    pub functions: Vec<Function>,
}

// Any functions here are in scope for all the grammar actions above.
pub fn dec_int(s: &str) -> Result<isize, ()> {
    s.parse::<isize>().map_err(|_| ())
}

pub fn bin_int(s: &str) -> Result<isize, ()> {
    isize::from_str_radix(&s[2..], 2).map_err(|_| ())
}

pub fn hex_int(s: &str) -> Result<isize, ()> {
    isize::from_str_radix(&s[2..], 16).map_err(|_| ())
}

pub fn char_int(s: &str) -> Result<isize, ()> {
    let inner = s.trim_matches('\'');

    // \' single quote
    // \" double quote
    // \\ backslash
    // \n new line
    // \r carriage return
    // \t tab
    // \b backspace
    // \f form feed
    // \v vertical tab (Internet Explorer 9 and older treats '\v as 'v instead of a vertical tab ('\x0B). If cross-browser compatibility is a concern, use \x0B instead of \v.)
    // \0 null character (U+0000 NULL) (only if the next character is not a decimal digit; else it is an octal escape sequence)
    // \xFF character represented by the hexadecimal byte "FF"

    match inner {
        "\\'" => {
            Ok('\'' as isize)
        }
        "\\\"" => {
            Ok('"' as isize)
        }
        "\\\\" => {
            Ok('\\' as isize)
        }
        "\\n" => {
            Ok('\n' as isize)
        }
        "\\r" => {
            Ok('\r' as isize)
        }
        "\\t" => {
            Ok('\t' as isize)
        }
        "\\0" => {
            Ok('\0' as isize)
        }
        c => {
            c.chars()
                .next()
                .ok_or(())
                .map(|e| e as isize)
        }
    }
}

pub fn char_int_single(c: char, escaped: bool) -> Result<u16, ()> {
    match (c, escaped) {
        ('n', true) => {
            Ok('\n' as u16)
        }
        ('r', true) => {
            Ok('\r' as u16)
        }
        ('t', true) => {
            Ok('\t' as u16)
        }
        ('0', true) => {
            Ok('\0' as u16)
        }
        (_, _) => {
            Ok(c as u16)
        }
    }
}

pub fn string_int_arr(s: &str) -> Result<Vec<u16>, ()> {
    let mut string = vec![];
    let mut escaped = false;

    for c in s[1..s.len()-1].chars().into_iter() {
        if escaped {
            string.push(char_int_single(c, true)?);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else {
            string.push(char_int_single(c, false)?);
        }
    };

    string.push(0);

    Ok(string)
}