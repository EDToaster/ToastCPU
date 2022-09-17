


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