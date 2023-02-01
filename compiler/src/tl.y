%start Module
%%

Module -> Result<Module, ()>:
    { Ok(Module { span: $span, globals: vec![], inlines: vec![], functions: vec![], struct_defs: vec![], modules: vec![], usings: vec![], }) }
    | Module Function       { let mut m = $1?; m.functions.push($2?);   Ok(m)   }
    | Module Global         { let mut m = $1?; m.globals.push($2?);     Ok(m)   }
    | Module Inline         { let mut m = $1?; m.inlines.push($2?);     Ok(m)   }
    | Module StructDef      { let mut m = $1?; m.struct_defs.push($2?); Ok(m)   }
    | Module NamedModule    { let mut m = $1?; m.modules.push($2?);     Ok(m)   }
    | Module Using          { let mut m = $1?; m.usings.push($2?);      Ok(m)   }
    ;

NamedModule -> Result<NamedModule, ()>:
    'MOD' Identifier 'LB' Module 'RB' { Ok(NamedModule { name: $2?, module: $4? }) }
    ;


Function -> Result<Function, ()>:
    'FN' Identifier FuncType Block {
        Ok(
            Function {
                span: $span,
                name: $2?,
                type_def: $3?,
                body: $4?,
            }
        )
    }
    ;

Global -> Result<Global, ()>:
    'GLOBAL' Identifier Type IntLiteral                          { Ok(Global { span: $span, name: $2?, var_type: $3?, val: $4?, size: 1 })       }
    | 'GLOBAL' Identifier 'LS' IntLiteral 'RS' Type IntLiteral   { Ok(Global { span: $span, name: $2?, size: $4?.val, var_type: $6?, val: $7? }) }
    ;

Inline -> Result<Inline, ()>:
    'INLINE' Identifier Statement { Ok(Inline { span: $span, name: $2?, statement: $3? }) }
    ;

Using -> Result<Using, ()>:
    'USING' Identifier { Ok(Using { span: $span, name: $2? }) }
    ;

StructDef -> Result<StructDef, ()>:
    'STRUCT' Identifier 'LB' StructMembers 'RB' { Ok(StructDef { span: $span, name: $2?, members: $4? }) }
    ;

StructMembers -> Result<Vec<StructMember>, ()>:
    { Ok(vec![]) }
    | StructMembers StructMember { let mut v = $1?; v.push($2?); Ok(v) }
    ;

StructMember -> Result<StructMember, ()>:
    Identifier Type { Ok(StructMember { span: $span, name: $1?, var_type: $2?, size: 1 }) }
    | Identifier 'LS' IntLiteral 'RS' Type { Ok(StructMember { span: $span, name: $1?, size: $3?.val, var_type: $5? }) }
    ;

Typelist -> Result<Vec<LexType>, ()>:
    { Ok(vec![]) }
    | Typelist Type { let mut v = $1?; v.push($2?); Ok(v) }
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
    IntLiteral      { Ok(Statement::IntLiteral($1?))    }
    | IntArray      { Ok(Statement::IntArray($1?))      }
    | Identifier    { Ok(Statement::Identifier($1?))    }
    | Operator      { Ok(Statement::Operator($1?))      }
    | Block         { Ok(Statement::Block($1?))         }
    | Unroll        { Ok(Statement::Unroll($1?))        }
    | If            { Ok(Statement::If($1?))            }
    | While         { Ok(Statement::While($1?))         }
    | Let           { Ok(Statement::Let($1?))           }
    ;

Operator -> Result<Operator, ()>:
    'ADD'       { Ok(Operator::Add($span))      }
    | 'SUB'     { Ok(Operator::Sub($span))      }
    | 'BOR'     { Ok(Operator::BOr($span))      }
    | 'BAND'    { Ok(Operator::BAnd($span))     }
    | 'BNOT'    { Ok(Operator::BNot($span))     }
    | 'SSHR'    { Ok(Operator::Sshr($span))     }
    | 'SHR'     { Ok(Operator::Shr($span))      }
    | 'SHL'     { Ok(Operator::Shl($span))      }
    | 'XOR'     { Ok(Operator::Xor($span))      }
    | 'LOR'     { Ok(Operator::LOr($span))      }
    | 'LAND'    { Ok(Operator::LAnd($span))     }
    | 'LNOT'    { Ok(Operator::LNot($span))     }
    | 'EQ'      { Ok(Operator::Eq($span))       }
    | 'LT'      { Ok(Operator::Lt($span))       }
    | 'LTE'     { Ok(Operator::Lte($span))      }
    | 'GT'      { Ok(Operator::Gt($span))       }
    | 'GTE'     { Ok(Operator::Gte($span))      }
    | 'RETURN'  { Ok(Operator::Return($span))   }
    | 'HOLE'    { Ok(Operator::Hole($span))     }
    | 'LP' 'RP' { Ok(Operator::Call($span)) }
    | 'PTR_OP' 'LP' Identifier 'RP' { Ok(Operator::Ptr($span, $3?)) }
    | 'AS' 'LP' Type 'RP' { Ok(Operator::As($span, $3?))          }
    | 'SIZEOF' 'LP' Type 'RP' { Ok(Operator::SizeOf($span, $3?))  }
    | 'STRUCT_ACCESS' {
            let v = $1.map_err(|_| ())?;
            Ok(Operator::StructAccess(v.span(), $lexer.span_str(v.span())[1..].to_string()))
        }
    | 'LS' IntLiteral 'RS' { Ok(Operator::ConstArrayAccess($span, $2?)) }
    ;

Type -> Result<LexType, ()>:
    BaseType    { $1 }
    | GenType   { $1 }
    | PtrType   { $1 }
    | 'LP' FuncType 'RP' { Ok(LexType::Func($2?)) }
    ;

BaseType -> Result<LexType, ()>:
    Identifier { Ok(LexType::Base($1?)) }
    ;

GenType -> Result<LexType, ()>:
    'GENERIC' Identifier { Ok(LexType::Gen($2?)) }
    ;

PtrType -> Result<LexType, ()>:
    Type 'PTR' { Ok(LexType::Ptr(Box::new($1?))) }
    ;

FuncType -> Result<FuncType, ()>:
    Typelist 'RARROW' Typelist { Ok(FuncType { i: $1?, o: $3? }) }
    ;

LexType -> Result<LexType, ()>:
    Identifier  { Ok(LexType::Base($1?)) }
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

Unmatched -> ():
    "UNMATCHED" { } 
    ;
%%

use lrpar::Span;

#[derive(Debug, Clone)]
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

    Return(Span),

    Ptr(Span, Identifier),
    Call(Span),
    As(Span, LexType),
    SizeOf(Span, LexType),
    StructAccess(Span, String),
    ConstArrayAccess(Span, IntLiteral),

    Hole(Span),
}

#[derive(Debug, Clone)]
pub struct IntLiteral {
    pub span: Span,
    pub val: isize,
}

#[derive(Debug, Clone)]
pub struct IntArray {
    pub span: Span,
    pub val: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Unroll {
    pub span: Span,
    pub times: IntLiteral,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct If {
    pub span: Span,
    pub if_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct While {
    pub span: Span,
    pub eval: Block,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub span: Span,
    pub bindings: Vec<Identifier>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub span: Span,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub span: Span,
    pub name: Identifier,
    pub type_def: FuncType,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Global {
    pub span: Span,
    pub name: Identifier,
    pub size: isize,
    pub var_type: LexType,
    pub val: IntLiteral,
}

#[derive(Debug, Clone)]
pub struct Inline {
    pub span: Span,
    pub name: Identifier,
    pub statement: Statement,
}

#[derive(Debug, Clone)]
pub struct Using {
    pub span: Span,
    pub name: Identifier,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub span: Span,
    pub name: Identifier,
    pub members: Vec<StructMember>,
}

#[derive(Debug, Clone)]
pub struct StructMember {
    pub span: Span,
    pub name: Identifier,
    pub var_type: LexType,
    pub size: isize,
}

#[derive(Debug, Clone)]
pub struct NamedModule {
    pub name: Identifier,
    pub module: Module,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub span: Span,
    pub globals: Vec<Global>,
    pub inlines: Vec<Inline>,
    pub functions: Vec<Function>,
    pub struct_defs: Vec<StructDef>,
    pub modules: Vec<NamedModule>,
    pub usings: Vec<Using>,
}

#[derive(Debug, Clone)]
pub enum LexType {
    Base(Identifier),
    // needs to be base type, cannot be generic type
    Ptr(Box<LexType>),
    Gen(Identifier),
    Func(FuncType),
}

#[derive(Debug, Clone)]
pub struct FuncType {
    pub i: Vec<LexType>,
    pub o: Vec<LexType>,
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