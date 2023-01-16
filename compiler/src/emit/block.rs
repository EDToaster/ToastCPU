use lrpar::Span;
use crate::emit::operator::emit_operator;
use crate::emit::type_check::check_and_apply_stack_transition;
use crate::emit::types::*;
use crate::tl_y::*;

pub fn emit_block(block_id: &str, b: &Block, global_state: &mut GlobalState, function_state: &mut FunctionState) -> Result<String, (Span, String)> {
    let mut block = "".to_string();

    let mut counter = 0;
    let mut subblock_counter = 0;

    for i in &b.body {
        match i {
            Statement::IntLiteral(IntLiteral { val, span }) => {
                // push val onto stack
                tasm!(
                    block; val;
                    r"
    imov! t0 {}
    push t0
                    "
                );
                check_and_apply_stack_transition(val.to_string().as_str(), span, function_state, &vec![], &vec![Type::U16])?;
            }
            Statement::IntArray(IntArray { val, span }) => {
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
                check_and_apply_stack_transition(format!("{val:?}").as_str(), span, function_state,
                                                 &vec![], &vec![Type::Pointer(1, Box::new(Type::U16))])?
            }
            Statement::Identifier(Identifier { name, span }) => {
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
                        check_and_apply_stack_transition("dup", span, function_state,
                                                         &vec![Type::new_generic("$a")], &vec![Type::new_generic("$a"), Type::new_generic("$a")])?;
                    }
                    "over" => {
                        tasm!(
                            block;;
                            r"
    pop! t0 t1
    push! t1 t0 t1
                            "
                        );
                        check_and_apply_stack_transition("over", span, function_state,
                                                         &vec![Type::new_generic("$a"), Type::new_generic("$b")],
                                                         &vec![Type::new_generic("$a"), Type::new_generic("$b"), Type::new_generic("$a")])?;
                    }
                    "swap" => {
                        tasm!(
                            block;;
                            r"
    pop! t0 t1
    push! t0 t1
                            "
                        );
                        check_and_apply_stack_transition("swap", span, function_state,
                                                         &vec![Type::new_generic("$a"), Type::new_generic("$b")],
                                                         &vec![Type::new_generic("$b"), Type::new_generic("$a")])?;
                    }
                    "halt" => {
                        tasm!(
                            block;;
                            r"
    halt
                            "
                        );
                    }
                    "drop" => {
                        tasm!(
                            block;;
                            r"
    pop! t0
                            "
                        );
                        check_and_apply_stack_transition("swap", span, function_state,
                                                         &vec![Type::new_generic("$a")],
                                                         &vec![])?;
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
                        check_and_apply_stack_transition("load", span, function_state,
                                                         &vec![Type::Pointer(1, Box::new(Type::new_generic("$a")))],
                                                         &vec![Type::new_generic("$a")])?;
                    }
                    "store" => {
                        tasm!(
                            block;;
                            r"
    pop! t1 t0
    str  t1 t0
                            "
                        );
                        check_and_apply_stack_transition("store", span, function_state,
                                                         &vec![Type::new_generic("$a"), Type::Pointer(1, Box::new(Type::new_generic("$a")))],
                                                         &vec![])?;
                    }
                    s => {
                        let mut offset_opt = None;
                        let mut offset = 0;
                        for (n, t) in function_state.current_bindings.iter().rev() {
                            offset += t.type_size(&global_state.struct_defs).map_err(|e| (span.clone(), e))?;
                            if n == s {
                                offset_opt = Some(t);
                                break;
                            }
                        }
                        if let Some(t) = offset_opt {
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
                            check_and_apply_stack_transition(s, span, function_state,
                                                             &vec![],
                                                             &vec![t.add_ref()])?;
                        } else if let Some((label, t)) = global_state.globals.get(s) {
                            // global variable
                            tasm!(
                                block;;
                                r"
    imov! t0 .{label}
    push! t0
                                "
                            );
                            check_and_apply_stack_transition(s, span, function_state,
                                                             &vec![],
                                                             &vec![t.add_ref()])?;
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
                            // todo: temporary while we get static analysis going
                            let (in_t, out_t) = global_state.function_signatures
                                .get(s)
                                .ok_or((span.clone(), format!("Was not able to find function signature of function {s}")))?;
                            check_and_apply_stack_transition(s, &span, function_state, in_t, out_t)?;
                        }

                    }
                }
            }
            Statement::Operator(r) => {
                let op = emit_operator(format!("{block_id}_{subblock_counter}_operator").as_str(), r, global_state, function_state)?;
                subblock_counter += 1;
                tasm!(
                    block;;
                    r"
{op}
                    "
                );
            }
            Statement::Block(b) => {
                let subblock_id = format!("{block_id}_{subblock_counter}");
                subblock_counter += 1;
                let subblock = emit_block(&*subblock_id, b, global_state, function_state)?;
                tasm!(
                    block;;
                    r"
{subblock}
                    "
                );
            }
            Statement::Unroll(Unroll { times: IntLiteral { val, .. }, body, ..}) => {
                for _ in 0..*val {
                    let subblock_id = format!("{block_id}_{subblock_counter}");
                    subblock_counter += 1;
                    let subblock = emit_block(&*subblock_id, body, global_state, function_state)?;
                    tasm!(
                        block;;
                        r"
{subblock}
                        "
                    );
                }
            }
            Statement::If(If { if_block, else_block, span }) => {
                let if_id = format!("{block_id}_{subblock_counter}_if");
                let else_id = format!("{block_id}_{subblock_counter}_else");
                let if_else_exit = format!("{block_id}_{subblock_counter}_if_exit");
                subblock_counter += 1;

                match function_state.stack_view.pop() {
                    None => {
                        Err((span.clone(), "If statement needs an element at the top of the stack to serve as a conditional. Current stack is empty.".to_string()))
                    }
                    Some(Type::Pointer(..)) | Some(Type::U16) => { Ok(()) }
                    e => {
                        Err((span.clone(), format!("Element `{e:?}` at the top of the stack cannot be used as the conditional for the if statement.")))
                    }
                }?;

                let else_stack = function_state.stack_view.clone();
                let if_subblock = emit_block(&*if_id, if_block, global_state, function_state)?;
                let if_stack = function_state.stack_view.clone();
                function_state.stack_view = else_stack;

                let else_subblock = match else_block {
                    None => "".to_string(),
                    Some(e) => emit_block(&*else_id, e, global_state, function_state)?,
                };

                if if_stack != function_state.stack_view {
                    return Err((span.clone(), format!("If and else blocks do not have the same elements on the stack after execution. If has {:?} and else has {:?}",
                                                      if_stack, function_state.stack_view)));
                }

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
            Statement::While(While { eval, body, span }) => {
                // stack states
                // .., eval, pop, body, ..
                let while_eval_id = format!("{block_id}_{subblock_counter}_while_eval");
                let while_body_id = format!("{block_id}_{subblock_counter}_while_block");
                let while_eval_exit = format!("{block_id}_{subblock_counter}_while_exit");
                subblock_counter += 1;

                let before_eval = function_state.stack_view.clone();

                let while_eval_subblock =
                    emit_block(&*while_eval_id, eval, global_state, function_state)?;

                match function_state.stack_view.pop() {
                    None => {
                        Err((span.clone(), "While statement needs an element at the top of the stack to serve as a conditional. Current stack is empty.".to_string()))
                    }
                    Some(Type::Pointer(..)) | Some(Type::U16) => { Ok(()) }
                    e => {
                        Err((span.clone(), format!("Element `{e:?}` at the top of the stack cannot be used as the conditional for the while statement.")))
                    }
                }?;

                let after_eval = function_state.stack_view.clone();

                let while_body_subblock =
                    emit_block(&*while_body_id, body, global_state, function_state)?;

                // after body == before eval
                if function_state.stack_view != before_eval {
                    return Err((span.clone(), format!(
                        "Stack state after while body and before condition evaluation need to identical. Before evaluation {before_eval:?}, after body {:?}",
                        function_state.stack_view)))
                }

                // after eval is new state
                function_state.stack_view = after_eval;

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
            Statement::Let(Let { bindings, body, span }) => {
                let num_bindings = bindings.len() as usize;

                tasm! (
                    block;;
                    r"
    load!   t0 .ret_stack_ptr
                    "
                );

                // for each binding starting at end
                for i in bindings.iter().rev() {
                    let t_opt = function_state.stack_view.pop();

                    match t_opt {
                        None => {
                            return Err((span.clone(), format!("Cannot bind `{}` since the stack is empty at this point.", i.name)))
                        }
                        Some(t) => {
                            // append to bindings list for function
                            function_state.current_bindings.push((i.name.clone(), t));
                        }
                    }

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
                let let_block = emit_block(let_block_id, body, global_state, function_state)?;
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
                function_state.current_bindings.truncate(function_state.current_bindings.len() - num_bindings);
            }
        }
    }

    Ok(block)
}