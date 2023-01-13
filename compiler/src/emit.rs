use std::arch::global_asm;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::format;
use crate::tl_y::{Block, char_int, Function, Global, Identifier, If, IntArray, IntLiteral, Let, Module, Operator, Statement, Unroll, While};
macro_rules! tasm {
    ($prog:ident; $($params:expr),*; $asm:literal) => {
        $prog.push_str(&*format!($asm, $($params),*));
    }
}

pub struct GlobalState {
    pub string_allocs_counter: isize,
    pub string_allocs: HashMap<Vec<u16>, String>,
    pub stack_changes: HashMap<String, isize>,
    // [a b c d] means that `d` is at the top of the ret stack
    pub current_bindings: Vec<String>,
    pub global_labels: HashMap<String, String>, // name -> label
}

// todo: do Result<String> instead
// todo: allow for multiple modules (and includes)
pub fn emit_module(m: &Module) -> Result<String, ()> {
    // preprocess function sizes
    let mut stack_changes: HashMap<String, isize> = HashMap::new();
    for f in &m.functions {
        let name = &f.name.name;
        stack_changes.insert(name.clone(), f.out_t.len() as isize - f.in_t.len() as isize);
    }

    // grab global variables
    let mut global_prog: String = String::new();
    let mut global_labels: HashMap<String, String> = HashMap::new();

    let mut global_init_routine = r#"
    push! t0
    "#.to_string();

    for g in &m.globals {
        let (emitted, label, val) = emit_global(g)?;
        global_prog.push_str(emitted.as_str());
        global_labels.insert(g.name.name.clone(), label.clone());

        tasm!(
            global_init_routine;;
            r"
    imov!  t0 {val}
    str!   .{label} t0
            "
        );
    }

    global_init_routine.push_str(r"
    pop! t0
    jmpr
                                        ");

    let mut global_state = GlobalState {
        stack_changes,
        string_allocs_counter: 0,
        string_allocs: HashMap::new(),
        current_bindings: vec![],
        global_labels
    };

    let mut prog = format!(r#"
.reset
    # setup isr and jump to main
    imov!   isr .isr
    # call!   .print_init

    # initialize ret stack ptrs
    imov!   t0 .ret_stack
    imov!   t1 .reset_ret

    str     t0 t1
    iadd    t0 1
    str!    .ret_stack_ptr t0

    # initialize our global variables
    call!   .init_globals
    jmp!    .main
.reset_ret
    halt

.init_globals
{global_init_routine}

{global_prog}

# allocate 1024 words on the heap
.ret_stack_ptr [1]
.ret_stack [0x0400]

# #include<../../lib/std/print>
# #include<../../lib/std/keyboard>

fn .isr
    isr!
    rti!
#end .isr
"#);

    let mut functions: String = String::new();

    for f in &m.functions {
        functions.push_str(&*emit_function(&f, &mut global_state));
    }

    // gather string defs
    prog.push_str(&*emit_string_defs(&global_state));

    prog.push_str(&*functions);
    Ok(prog)
}

pub fn emit_string_defs(global_state: &GlobalState) -> String {
    let mut string_defs: String = String::new();

    for (k, v) in global_state.string_allocs.iter() {
        tasm!(
            string_defs;;
            r"
.{v}
"
        );

        for u in k {
            tasm!(
                string_defs;;
                "    0x{u:X}\n"
            );
        }
    }

    string_defs
}


pub fn emit_global(g: &Global) -> Result<(String, String, isize), ()> {
    let global_name = &g.name.name;
    let label = &format!("variable_alloc_{global_name}");
    // todo: change size to be dynamic based on type
    let size = 1;
    let prog = format!(r"
.{label} [{size}]
    ");
    Ok((prog, label.to_string(), g.val.val))
}

pub fn emit_block(block_id: &str, b: &Block, global_state: &mut GlobalState) -> (String, isize) {
    let mut block = "".to_string();

    let mut counter = 0;
    let mut subblock_counter = 0;

    let mut stack_size = 0;

    for i in &b.body {
        match i {
            Statement::IntLiteral(IntLiteral { val, .. }) => {
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
            Statement::IntArray(IntArray { val, ..}) => {
                let mut string_alloc_label = &*format!("string_alloc_{}", global_state.string_allocs_counter);
                global_state.string_allocs_counter += 1;

                string_alloc_label = global_state.string_allocs
                    .entry(val.clone()).or_insert_with(|| string_alloc_label.into());

                tasm!(
                    block;;
                    r"
    imov! t0 .{string_alloc_label}
    push! t0
                    "
                );
                stack_size += 1;
            }
            Statement::Identifier(Identifier { name, .. }) => {
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
                    "over" => {
                        tasm!(
                            block;;
                            r"
    pop! t0 t1
    push! t1 t0 t1
                            "
                        );
                        stack_size += 1;
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
                    "halt" => {
                        tasm!(
                            block;;
                            r"
    halt
                            "
                        );
                    }
    //                 "p" => {
    //                     tasm!(
    //                         block;;
    //                         r"
    // pop! p0
    // call! .print_word
    //                         "
    //                     );
    //                     stack_size -= 1;
    //                 }
    //                 "pc" => {
    //                     tasm!(
    //                         block;;
    //                         r"
    // pop! p0
    // call! .print_char
    //                         "
    //                     );
    //                     stack_size -= 1;
    //                 }
    //                 "ps" => {
    //                     tasm!(
    //                         block;;
    //                         r"
    // pop! p0
    // call! .print_string
    //                         "
    //                     );
    //                     stack_size -= 1;
    //                 }
                    "drop" => {
                        tasm!(
                            block;;
                            r"
    pop! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    "load" => {
                        tasm!(
                            block;;
                            r"
    pop! t0
    load t0 t0
    push! t0
                            "
                        );
                    }
                    "store" => {
                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    str  t1 t0
                            "
                        );
                        stack_size -= 2;
                    }
                    s => {
                        // check if it's a binding
                        let offset_opt = global_state
                            .current_bindings
                            .iter()
                            .rev()
                            .enumerate()
                            .filter(|(_, b)| **b == s.to_string())
                            .map(|(i, _)| i + 1)
                            .next();

                        if let Some(offset) = offset_opt {
                            // is a binding
                            tasm!(
                                block;;
                                r"
    load!   t0 .ret_stack_ptr
    imov!   t1 {offset}
    sub     t0 t1
    push!   t0
                                "
                            );
                            stack_size += 1;
                        } else if let Some(label) = global_state.global_labels.get(s) {
                            // global variable
                            tasm!(
                                block;;
                                r"
    imov! t0 .{label}
    push! t0
                                "
                            );
                            stack_size += 1;
                        } else {
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
                            stack_size += global_state.stack_changes.get(s).unwrap();
                        }

                    }
                }
            }
            Statement::Operator(r) => {
                match r {
                    Operator::Add(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    add   t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Sub(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    sub   t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::BOr(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    or    t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::BAnd(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    and   t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::BNot(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t0
    not   t0
    push! t0
                            "
                        );
                    }
                    Operator::Sshr(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    sshr  t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Shr(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    shr   t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Shl(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    shl   t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Xor(_) => {
                        tasm!(
                            block;;
                            r"
    pop!  t1 t0
    xor   t0 t1
    push! t0
                            "
                        );
                        stack_size -= 1;
                    }

                    Operator::LOr(_) => {
                        let true_label = format!("{block_id}_{subblock_counter}_ortrue");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    imov t2 1
    tst  t0
    jnz! .{true_label}
    tst  t1
    jnz! .{true_label}
    imov t2 0
.{true_label}
    push! t2
                            "
                        );
                        stack_size -= 1
                    }

                    Operator::LAnd(_) => {
                        let false_label = format!("{block_id}_{subblock_counter}_andfalse");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    imov t2 0
    tst  t0
    jz! .{false_label}
    tst  t1
    jz! .{false_label}
    imov t2 1
.{false_label}
    push! t2
                            "
                        );
                        stack_size -= 1;
                    }

                    Operator::LNot(_) => {
                        let false_label = format!("{block_id}_{subblock_counter}_nottrue");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t0
    imov t1 1
    tst  t0
    jz! .{false_label}
    imov t1 0
.{false_label}
    push! t1
                            "
                        );
                    }

                    Operator::Eq(_) => {
                        let skip_label = format!("{block_id}_{subblock_counter}_opskip");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    imov t2 1  # default value
    sub  t0 t1
    jz!  .{skip_label}
    imov t2 0
.{skip_label}
    push! t2
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Lt(_) => {
                        let skip_label = format!("{block_id}_{subblock_counter}_opskip");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    imov t2 1  # default value
    sub  t0 t1
    jn!  .{skip_label}
    imov t2 0
.{skip_label}
    push! t2
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Lte(_) => {
                        let skip_label = format!("{block_id}_{subblock_counter}_opskip");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    imov t2 0  # default value
    sub  t0 t1
    jp!  .{skip_label}
    imov t2 1
.{skip_label}
    push! t2
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Gt(_) => {
                        let skip_label = format!("{block_id}_{subblock_counter}_opskip");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    imov t2 1  # default value
    sub  t0 t1
    jp!  .{skip_label}
    imov t2 0
.{skip_label}
    push! t2
                            "
                        );
                        stack_size -= 1;
                    }
                    Operator::Gte(_) => {
                        let skip_label = format!("{block_id}_{subblock_counter}_opskip");
                        subblock_counter += 1;

                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    imov t2 0  # default value
    sub  t0 t1
    jn!  .{skip_label}
    imov t2 1
.{skip_label}
    push! t2
                            "
                        );
                        stack_size -= 1;
                    }
                }
            }
            Statement::Block(b) => {
                let subblock_id = format!("{block_id}_{subblock_counter}");
                subblock_counter += 1;
                let (subblock, stack_change) = emit_block(&*subblock_id, b, global_state);
                tasm!(
                    block;;
                    r"
{subblock}
                    "
                );
                stack_size += stack_change;
            }
            Statement::Unroll(Unroll { times: IntLiteral { val, .. }, body, ..}) => {
                for _ in 0..*val {
                    let subblock_id = format!("{block_id}_{subblock_counter}");
                    subblock_counter += 1;
                    let (subblock, stack_change) = emit_block(&*subblock_id, body, global_state);
                    tasm!(
                        block;;
                        r"
{subblock}
                        "
                    );
                    stack_size += stack_change;
                }
            }
            Statement::If(If { if_block, else_block, span }) => {
                let if_id = format!("{block_id}_{subblock_counter}_if");
                let else_id = format!("{block_id}_{subblock_counter}_else");
                let if_else_exit = format!("{block_id}_{subblock_counter}_if_exit");
                subblock_counter += 1;

                let (if_subblock, if_sc) = emit_block(&*if_id, if_block, global_state);
                let (else_subblock, else_sc) = match else_block {
                    None => ("".to_string(), 0),
                    Some(e) => emit_block(&*else_id, e, global_state),
                };

                if if_sc != else_sc {
                    panic!("If and else blocks do not have the same stack change size in {if_id}, if has {if_sc} and else has {else_sc}");
                }
                stack_size += if_sc - 1;

                tasm!(
                    block;;
                    r"
    pop! t0
    tst  t0
    jz!  .{else_id}
.{if_id}
{if_subblock}
    jmp! .{if_else_exit}
.{else_id}
{else_subblock}
.{if_else_exit}
                    "
                );
            }
            Statement::While(While { eval, body, .. }) => {
                let while_eval_id = format!("{block_id}_{subblock_counter}_while_eval");
                let while_body_id = format!("{block_id}_{subblock_counter}_while_block");
                let while_eval_exit = format!("{block_id}_{subblock_counter}_while_exit");
                subblock_counter += 1;

                let (while_eval_subblock, while_eval_sc) =
                    emit_block(&*while_eval_id, eval, global_state);
                let (while_body_subblock, while_body_sc) =
                    emit_block(&*while_body_id, body, global_state);

                if while_eval_sc + while_body_sc != 1 {
                    panic!("While eval block and body block added together, must have stack change size of 1, but has {}", while_eval_sc + while_body_sc);
                }

                stack_size += while_eval_sc - 1;

                tasm!(
                    block;;
                    r"
.{while_eval_id}
{while_eval_subblock}
    pop! t0
    tst  t0
    jz!  .{while_eval_exit}
.{while_body_id}
{while_body_subblock}
    jmp! .{while_eval_id}
.{while_eval_exit}
                    "
                );
            }
            Statement::Let(Let { bindings, body, .. }) => {
                let num_bindings = bindings.len() as usize;

                tasm! (
                    block;;
                    r"
    load!   t0 .ret_stack_ptr
                    "
                );

                // for each binding starting at end
                for i in bindings.iter().rev() {
                    // append to bindings list for function
                    global_state.current_bindings.push(i.name.clone());

                    tasm! (
                        block;;
                        r"
    # pop from the data stack
    # push to return stack
    pop!    t1
    str     t0 t1
    iadd    t0 1
                        "
                    );
                }

                tasm! (
                    block;;
                    r"
    str!    .ret_stack_ptr t0
                    "
                );

                // emit inner block
                let let_block_id = &*format!("{block_id}_{subblock_counter}_let_block");
                subblock_counter += 1;
                let (let_block, let_sc) = emit_block(let_block_id, body, global_state);
                tasm!(
                    block;;
                    r"
{let_block}
    # pop {num_bindings} elements from return stack
    load!   t0 .ret_stack_ptr
    imov!   t1 {num_bindings}
    sub     t0 t1
    str!    .ret_stack_ptr t0
                    "
                );

                // pop bindings off
                global_state.current_bindings.truncate(global_state.current_bindings.len() - num_bindings);

                stack_size += let_sc - num_bindings as isize;
            }
        }
    }

    (block, stack_size)
}

pub fn emit_function(f: &Function, global_state: &mut GlobalState) -> String {
    // at this point we really only care about one function
    let func_name = &f.name.name;
    let func_exit = format!("{}_exit", func_name);
    let mut func = format!("fn .{}\n", func_name);

    // reset the bindings
    global_state.current_bindings.clear();

    let (block, stack_size) = emit_block(&*format!("{func_name}_body"), &f.body, global_state);

    global_state.current_bindings.clear();

    let declared_stack_size = *(global_state.stack_changes.get(func_name).unwrap());
    if stack_size != declared_stack_size {
        panic!("Stack size of {stack_size} does not correspond to declared stack size of {declared_stack_size} in function {func_name}");
    }

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