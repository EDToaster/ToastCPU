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
pub struct BoolLiteral {
    pub span: Span,
    pub val: isize,
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
    BoolLiteral(BoolLiteral),
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
