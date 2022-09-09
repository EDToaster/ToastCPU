use crate::tasm_ops::*;

#[derive(Debug)]
pub struct Label<'input> {
    pub label: &'input str
}

#[derive(Debug)]
pub struct Register {
    pub register: i32
}

#[derive(Debug)]
pub struct Allocation {
    pub size: i32
}

#[derive(Debug)]
pub enum Instruction<'input> {
    PushPop(PushPopOpcode, Register),
    LoadStore(LoadStoreOpcode, Register, Register),
    IMove(IMoveOpcode, Register, IVal<'input>),
    NoArgument(NoArgumentOpcode),

    Jump(JumpOpcode, Register),
    JumpLink(JumpLinkOpcode, Register),
    JumpRet(JumpRetOpcode),

    SingleALU(SingleALUOpcode, Register),
    ALU(ALUOpcode, Register, Register),
    IALU(IALUOpcode, Register, IVal<'input>),
} 

#[derive(Debug)]
pub enum Statement<'input> {
    Label(Label<'input>),
    Instruction(Instruction<'input>),
    Numeric(i32),
}

#[derive(Debug)]
pub enum IVal<'input> {
    Label(Label<'input>),
    Numeric(i32),
}