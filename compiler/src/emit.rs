use crate::tl_y::{Block, Function, Identifier, IntLiteral, Module, Operator, Statement};

macro_rules! tasm {
    ($prog:ident; $($params:expr),*; $asm:literal) => {
        $prog.push_str(&*format!($asm, $($params),*));
    }
}

pub fn emit_module(m: &Module) -> String {
    let mut prog = r#"
.reset
    # setup isr and jump to main
    imov!   isr .isr
    call!   .print_init

    # initialize ret stack ptrs
    imov!   t0 .ret_stack
    imov!   t1 .reset_ret

    str     t0 t1
    iadd    t0 1
    str!    .ret_stack_ptr t0

    jmp!    .main
.reset_ret
    halt

# allocate 256 words on the heap
.ret_stack_ptr [1]
.ret_stack [0x0100]

#include<../../lib/std/print>
#include<../../lib/std/keyboard>

fn .isr
    isr!
    rti!
#end .isr
"#.to_string();

    for f in &m.functions {
        prog.push_str(&*emit_function(&f));
    }

    prog
}

pub fn emit_block(block_id: &str, b: &Block) -> String {
    let mut block = "".to_string();

    let mut counter = 0;
    let mut subblock_counter = 0;

    let mut stack_size = 0;

    for i in &b.body {
        match i {
            Statement::IntLiteral(IntLiteral { span, val }) => {
                // push val onto stack
                tasm!(
                    block; val;
                    r"
    imov! t0 {}
    push t0
                    "
                );
                stack_size += 1;
            }
            Statement::Identifier(Identifier { span, name }) => {
                // handle built in funcs
                match name.as_str() {
                    "dup" => {
                        tasm!(
                            block;;
                            r"
    pop! t0
    push! t0 t0
                            "
                        );
                        stack_size += 1;
                    }
                    "dup2" => {
                        tasm!(
                            block;;
                            r"
    pop! t0 t1
    push! t1 t0 t1 t0
                            "
                        );
                        stack_size += 2;
                    }
                    "swap" => {
                        tasm!(
                            block;;
                            r"
    pop! t0 t1
    push! t0 t1
                            "
                        );
                    }
                    "p" => {
                        tasm!(
                            block;;
                            r"
    pop! p0
    call! .print_word
                            "
                        );
                        stack_size -= 1;
                    }
                    "drop" => {
                        tasm!(
                            block;;
                            r"
    pop! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    s => {
                        // generic function call
                        let ret_label = format!("{}_retaddr{}", block_id, counter);
                        counter += 1;
                        tasm!(
                            block;;
                            r"
    # load the stack ptr addr into t0
    load!   t0 .ret_stack_ptr

    # load the return addr
    imov!   t1 .{ret_label}

    # store t1 at t0
    str     t0 t1

    # increment t0
    iadd    t0 1
    str!    .ret_stack_ptr t0

    jmp!    .{s}
.{ret_label}
                            "
                        );
                    }
                }
            }
            Statement::Operator(r) => {
                match r {
                    Operator::Add(span) => {
                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    add t0 t1
    push! t0
                            "
                        );
                    }
                    Operator::Sub(span) => {
                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    sub t0 t1
    push! t0
                            "
                        );
                    }
                }
            }
            Statement::Block(b) => {
                let subblock_id = format!("{block_id}_{subblock_counter}");
                subblock_counter += 1;
                let subblock = emit_block(&*subblock_id, b);
                tasm!(
                    block;;
                    r"
{subblock}
                    "
                );
            }
        }
    }

    block
}

pub fn emit_function(f: &Function) -> String {
    // at this point we really only care about one function
    let func_name = &f.name.name;
    let func_exit = format!("{}_exit", func_name);
    let mut func = format!("fn .{}\n", func_name);

    let block = emit_block(&*format!("{func_name}_body"), &f.body);
    func.push_str(&*format!(r"
{block}
    "));

    func.push_str(&*format!(r"
.{func_exit}
    # load the stack ptr addr into t0
    load!   t0 .ret_stack_ptr
    isub    t0 1

    # load the return addr
    load    t1 t0

    # str the stack ptr addr
    str!    .ret_stack_ptr t0

    jmp     t1
#end .{}
", f.name.name));
    func
}