use crate::{tasm_ast::*, tasm_ops::*};

mod tasm_ast;
mod tasm_convert;
mod tasm_ops;
mod toast_lex;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tasm); // synthesized by LALRPOP

#[test]
fn parse_num() {
    assert!(tasm::NumParser::new().parse("42").is_ok());

    assert!(tasm::NumParser::new().parse("0x42BeeF").is_ok());
    assert!(tasm::NumParser::new().parse("0x 42BeeF").is_err());

    assert!(tasm::NumParser::new().parse("0b0100100100").is_ok());
    assert!(tasm::NumParser::new().parse("0b 0100100100").is_err());

    assert!(tasm::AllocationParser::new().parse("[1]").unwrap().size == 1);
    assert!(tasm::AllocationParser::new().parse("[ 12 ]").unwrap().size == 12);
    assert!(
        tasm::AllocationParser::new()
            .parse("[ 0x10 ]")
            .unwrap()
            .size
            == 16
    );
}

#[test]
fn parse_label() {
    assert!(tasm::LabelParser::new().parse(".hello42").is_ok());
    assert!(tasm::LabelParser::new().parse(".hello42").unwrap().label == ".hello42");
    assert!(tasm::LabelParser::new().parse(".fn_hello42").is_ok());
    assert!(tasm::LabelParser::new().parse(".42fun").is_err());
}

#[test]
fn parse_register() {
    assert!(tasm::RegisterParser::new().parse("r0").is_ok());
    assert!(tasm::RegisterParser::new().parse("r10").is_ok());
    assert!(tasm::RegisterParser::new().parse("r16").is_err());
    assert!(tasm::RegisterParser::new().parse("r 16").is_err());

    assert!(tasm::RegisterParser::new().parse("p0").unwrap().register == 1);
    assert!(tasm::RegisterParser::new().parse("t0").unwrap().register == 6);

    assert!(tasm::RegisterParser::new().parse("isr").unwrap().register == 12);
    assert!(tasm::RegisterParser::new().parse(".isr").is_err());
    assert!(tasm::RegisterParser::new().parse("sp").unwrap().register == 13);
    assert!(tasm::RegisterParser::new().parse("sr").unwrap().register == 14);
    assert!(tasm::RegisterParser::new().parse("pc").unwrap().register == 15);
}

#[test]
fn parse_instruction() {
    assert!(matches!(
        tasm::InstructionParser::new().parse("jz r7").unwrap(),
        Instruction::Jump(JumpOpcode::JZ, Register { register: 7 })
    ));

    assert!(tasm::InstructionParser::new().parse(".jz r7").is_err());

    assert!(matches!(
        tasm::InstructionParser::new().parse("jmpl r7").unwrap(),
        Instruction::JumpLink(JumpLinkOpcode::JMPL, Register { register: 7 })
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("jmpr").unwrap(),
        Instruction::JumpRet(JumpRetOpcode::JMPR)
    ));

    assert!(tasm::InstructionParser::new().parse("jmpr r7").is_err());

    assert!(matches!(
        tasm::InstructionParser::new().parse("push ar").unwrap(),
        Instruction::PushPop(PushPopOpcode::PUSH, Register { register: 0 })
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("load ar r0").unwrap(),
        Instruction::LoadStore(LoadStoreOpcode::LOAD, Register { register: 0 }, Register { register: 0 })
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("imov ar 0x00FF").unwrap(),
        Instruction::IMove(IMoveOpcode::IMOV, Register { register: 0 }, IVal::Numeric(0x00FF))
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("imov ar .label").unwrap(),
        Instruction::IMove(IMoveOpcode::IMOV, Register { register: 0 }, IVal::Label(Label { label: ".label"}))
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("halt").unwrap(),
        Instruction::NoArgument(NoArgumentOpcode::HALT)
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("not r0").unwrap(),
        Instruction::SingleALU(SingleALUOpcode::NOT, Register { register: 0 })
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("mov r0 r1").unwrap(),
        Instruction::ALU(ALUOpcode::MOV, Register { register: 0 }, Register { register: 1 })
    ));

    assert!(matches!(
        tasm::InstructionParser::new().parse("iadd r0 0xA").unwrap(),
        Instruction::IALU(IALUOpcode::IADD, Register { register: 0 }, IVal::Numeric(0xA))
    ));
}

#[test]
fn parse_statements() {
    assert!(tasm::StatementsParser::new().parse("").unwrap().len() == 0);
    assert!(tasm::StatementsParser::new().parse(".label \n .label2 0x0000").unwrap().len() == 2);
    assert!(tasm::StatementsParser::new().parse(".label \n .label2 \n 0x0000").unwrap().len() == 3);
    assert!(tasm::StatementsParser::new().parse(
        "
        .reset
            halt    # something something 
        /*  Comments  */
        .isr
            rti
        "
    ).unwrap().len() == 4);
}

#[test]
fn parse_file() {
    let s = include_str!("../../sample_programs/src/text/text.tasm");
    dbg!(tasm::StatementsParser::new().parse(s).unwrap());
}

fn main() {
    println!("Hello, world!");
}
