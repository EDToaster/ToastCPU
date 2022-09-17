


#[derive(Debug)]
pub enum Literal<'input> {
    Int(i32),
    Char(i32),
    String(&'input str),
}

#[derive(Debug)]
pub struct Identifier<'input> {
    pub id: &'input str,
}

#[derive(Debug)]
pub enum Statement<'input> {
    For(ForStatement<'input>),
    Assignment(Identifier<'input>, Expression<'input>),
    Declaration(Identifier<'input>),
    Empty()
}

#[derive(Debug)]
pub struct ForStatement<'input> {
    pub pre: Box<Statement<'input>>,
    pub cond: Expression<'input>,
    pub body: Vec<Statement<'input>>,
}


#[derive(Debug)]
pub enum Expression<'input> {
    Literal(Literal<'input>)
}