use crate::emit::block::emit_block;
use crate::emit::operator::emit_operator;
use crate::emit::type_check::check_and_apply_stack_transition;
use crate::emit::types::tasm;
use crate::emit::types::*;
use crate::tl_y::*;
use crate::util::gss::Stack;
use crate::util::labels::{generate_label, generate_label_with_context};
use lrpar::Span;

// todo: change counter and subblock_counter to use a unique label provider.
pub fn emit_statement(
    block_id: &str,
    s: &Statement,
    global_state: &mut GlobalState,
    function_state: &mut FunctionState,
    stack_view: &mut Stack<Type>,
) -> Result<String, (Span, String)> {
    let mut block = String::new();

    match s {
        Statement::IntLiteral(IntLiteral { val, span }) => {
            // push val onto stack
            tasm!(
                block; val;
                r"
    imov! t0 {}
    push  t0
                    "
            );
            check_and_apply_stack_transition(
                val.to_string().as_str(),
                span,
                stack_view,
                &vec![],
                &[Type::U16],
            )?;
        }
        Statement::IntArray(IntArray { val, span }) => {
            let mut string_alloc_label =
                &*format!("string_alloc_{}", global_state.string_allocs_counter);
            global_state.string_allocs_counter += 1;

            string_alloc_label = global_state
                .string_allocs
                .entry(val.clone())
                .or_insert_with(|| string_alloc_label.into());

            tasm!(
                block;;
                r"
    imov! t0 .{string_alloc_label}
    push! t0
                    "
            );
            check_and_apply_stack_transition(
                format!("{val:?}").as_str(),
                span,
                stack_view,
                &vec![],
                &[Type::Pointer(1, Box::new(Type::U16))],
            )?;
        }
        Statement::Identifier(Identifier { name, span }) => {
            // handle built in funcs
            match name.as_str() {
                "dup" => {
                    tasm!(
                        block;;
                        r"
    pop!  t0
    push! t0 t0
                            "
                    );
                    check_and_apply_stack_transition(
                        "dup",
                        span,
                        stack_view,
                        &vec![Type::new_generic("$a")],
                        &[Type::new_generic("$a"), Type::new_generic("$a")],
                    )?;
                }
                "over" => {
                    tasm!(
                        block;;
                        r"
    pop!  t0 t1
    push! t1 t0 t1
                            "
                    );
                    check_and_apply_stack_transition(
                        "over",
                        span,
                        stack_view,
                        &vec![Type::new_generic("$a"), Type::new_generic("$b")],
                        &[Type::new_generic("$a"),
                            Type::new_generic("$b"),
                            Type::new_generic("$a")],
                    )?;
                }
                "swap" => {
                    tasm!(
                        block;;
                        r"
    pop!  t0 t1
    push! t0 t1
                            "
                    );
                    check_and_apply_stack_transition(
                        "swap",
                        span,
                        stack_view,
                        &vec![Type::new_generic("$a"), Type::new_generic("$b")],
                        &[Type::new_generic("$b"), Type::new_generic("$a")],
                    )?;
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
    pop!  t0
                            "
                    );
                    check_and_apply_stack_transition(
                        "drop",
                        span,
                        stack_view,
                        &vec![Type::new_generic("$a")],
                        &[],
                    )?;
                }
                "load" => {
                    tasm!(
                        block;;
                        r"
    pop!  t0
    load  t0 t0
    push! t0
                            "
                    );
                    check_and_apply_stack_transition(
                        "load",
                        span,
                        stack_view,
                        &vec![Type::Pointer(1, Box::new(Type::new_generic("$a")))],
                        &[Type::new_generic("$a")],
                    )?;
                }
                "store" => {
                    tasm!(
                        block;;
                        r"
    pop!  t1 t0
    str   t1 t0
                            "
                    );
                    check_and_apply_stack_transition(
                        "store",
                        span,
                        stack_view,
                        &vec![
                            Type::new_generic("$a"),
                            Type::Pointer(1, Box::new(Type::new_generic("$a"))),
                        ],
                        &[],
                    )?;
                }
                s => {
                    let mut offset_opt = None;
                    let mut offset = 0;

                    for (n, t) in function_state.current_bindings.iter().rev() {
                        if n == s {
                            offset_opt = Some(t);
                            break;
                        }
                        offset += t.type_size(&global_state.struct_defs).err_with_span(span)?;
                    }

                    if let Some(t) = offset_opt {
                        // is a binding
                        tasm!(
                            block;;
                            r"
    mov   t0 t5
    imov! t1 {offset}
    add   t0 t1
    push  t0
                                "
                        );
                        global_state.string_allocs_counter += 1;
                        check_and_apply_stack_transition(
                            s,
                            span,
                            stack_view,
                            &vec![],
                            &[t.add_ref()],
                        )?;
                    } else if let Some((label, t)) = global_state.globals.get(s) {
                        // global variable
                        tasm!(
                            block;;
                            r"
    imov! t0 .{label}
    push! t0
                                "
                        );
                        check_and_apply_stack_transition(
                            s,
                            span,
                            stack_view,
                            &vec![],
                            &[t.add_ref()],
                        )?;
                    } else if let Some(statement) = global_state.inlines.get(s).cloned() {
                        // is an inline value
                        let expansion = emit_statement(
                            block_id,
                            &statement,
                            global_state,
                            function_state,
                            stack_view,
                        )?;
                        tasm!(
                            block;;
                            r"
{expansion}
                            "
                        );
                    } else {
                        // generic function call
                        let ret_label = generate_label_with_context(block_id, "retaddr");
                        tasm!(
                            block;;
                            r"
    imov! t0 .{ret_label}
    push  t5 t0
    jmp!  .{s}
.{ret_label}
                                "
                        );
                        // todo: temporary while we get static analysis going
                        let (in_t, out_t) = global_state.function_signatures.get(s).ok_or((
                            *span,
                            format!("Was not able to find function signature of function {s}"),
                        ))?;
                        check_and_apply_stack_transition(s, span, stack_view, in_t, out_t)?;
                    }
                }
            }
        }
        Statement::Operator(r) => {
            let op = emit_operator(
                &generate_label_with_context(block_id, "operator"),
                r,
                global_state,
                function_state,
                stack_view,
            )?;
            tasm!(
                block;;
                r"
{op}
                    "
            );
        }
        Statement::Block(b) => {
            let subblock = emit_block(
                &generate_label(block_id),
                b,
                global_state,
                function_state,
                stack_view,
            )?;
            tasm!(
                block;;
                r"
{subblock}
                    "
            );
        }
        Statement::Unroll(Unroll {
            times: IntLiteral { val, .. },
            body,
            ..
        }) => {
            for _ in 0..*val {
                let subblock = emit_block(
                    &generate_label(block_id),
                    body,
                    global_state,
                    function_state,
                    stack_view,
                )?;
                tasm!(
                    block;;
                    r"
{subblock}
                        "
                );
            }
        }
        Statement::If(If {
            if_block,
            else_block,
            span,
        }) => {
            let if_id = generate_label_with_context(block_id, "if");
            let else_id = generate_label_with_context(block_id, "else");
            let if_else_exit = generate_label_with_context(block_id, "if_exit");

            match stack_view.pop() {
                None => {
                    Err((*span, "If statement needs an element at the top of the stack to serve as a conditional. Current stack is empty.".to_string()))
                }
                Some(Type::Pointer(..)) | Some(Type::U16) => { Ok(()) }
                e => {
                    Err((*span, format!("Element `{e:?}` at the top of the stack cannot be used as the conditional for the if statement.")))
                }
            }?;

            let mut else_stack = stack_view.clone();
            // stack_view is end == if_stack
            let if_subblock =
                emit_block(&if_id, if_block, global_state, function_state, stack_view)?;

            let else_subblock = match else_block {
                None => "".to_string(),
                Some(e) => emit_block(&else_id, e, global_state, function_state, &mut else_stack)?,
            };

            if stack_view != &else_stack {
                return Err((*span, format!("If and else blocks do not have the same elements on the stack after execution. If has {stack_view:?} and else has {else_stack:?}")));
            }

            tasm!(
                block;;
                r"
    pop!  t0
    tst   t0
    jz!   .{else_id}
.{if_id}
{if_subblock}
    jmp!  .{if_else_exit}
.{else_id}
{else_subblock}
.{if_else_exit}
                    "
            );
        }
        Statement::While(While { eval, body, span }) => {
            // stack states
            // .., eval, pop, body, ..
            let while_eval_id = generate_label_with_context(block_id, "while_eval");
            let while_body_id = generate_label_with_context(block_id, "while_body");
            let while_eval_exit = generate_label_with_context(block_id, "while_exit");

            let before_eval = stack_view.clone();

            let while_eval_subblock = emit_block(
                &while_eval_id,
                eval,
                global_state,
                function_state,
                stack_view,
            )?;

            match stack_view.pop() {
                None => {
                    Err((*span, "While statement needs an element at the top of the stack to serve as a conditional. Current stack is empty.".to_string()))
                }
                Some(Type::Pointer(..)) | Some(Type::U16) => { Ok(()) }
                e => {
                    Err((*span, format!("Element `{e:?}` at the top of the stack cannot be used as the conditional for the while statement.")))
                }
            }?;

            // end stack view is after eval
            let mut after_body = stack_view.clone();

            let while_body_subblock = emit_block(
                &while_body_id,
                body,
                global_state,
                function_state,
                &mut after_body,
            )?;

            // after body == before eval
            if after_body != before_eval {
                return Err((*span, format!(
                    "Stack state after while body and before condition evaluation need to identical. Before evaluation {before_eval:?}, after body {after_body:?}")));
            }

            tasm!(
                block;;
                r"
.{while_eval_id}
{while_eval_subblock}
    pop!  t0
    tst   t0
    jz!   .{while_eval_exit}
.{while_body_id}
{while_body_subblock}
    jmp!  .{while_eval_id}
.{while_eval_exit}
                    "
            );
        }
        Statement::Let(Let {
            bindings,
            body,
            span,
        }) => {
            let num_bindings = bindings.len();

            // for each binding starting at end
            for i in bindings.iter().rev() {
                let t_opt = stack_view.pop();

                match t_opt {
                    None => {
                        return Err((
                            *span,
                            format!(
                                "Cannot bind `{}` since the stack is empty at this point.",
                                i.name
                            ),
                        ))
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
    pop!  t1
    push  t5 t1
                        "
                );
            }

            function_state.function_let_bindings += num_bindings as isize;

            // emit inner block
            let let_block_id = &*generate_label_with_context(block_id, "let_inner");
            let let_block =
                emit_block(let_block_id, body, global_state, function_state, stack_view)?;
            tasm!(
                block;;
                r"
{let_block}
    # pop {num_bindings} elements from return stack
    imov! t0 {num_bindings}
    add   t5 t0
                    "
            );

            // pop bindings off
            function_state.function_let_bindings -= num_bindings as isize;
            function_state
                .current_bindings
                .truncate(function_state.current_bindings.len() - num_bindings);
        }
    }

    Ok(block)
}
