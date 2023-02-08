use crate::emit::statement::find_function;
use crate::emit::type_check::*;
use crate::emit::types::*;
use crate::parser_util::types::*;
use crate::util::gss::Stack;
use crate::util::labels::{function_label, generate_label_with_context};
use lrpar::Span;

use super::types::FunctionType;

pub fn emit_operator(
    block_id: &str,
    r: &Operator,
    global_state: &mut GlobalState,
    function_state: &mut FunctionState,
    stack_view: &mut Stack<Type>,
    using_stack: &Stack<String>,
) -> Result<String, (Span, String)> {
    let mut operation = String::new();

    match r {
        Operator::Add(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    add   t0 t1
    push! t0
                            "
            );
            check_and_apply_multiple_stack_transitions(
                "+",
                span,
                stack_view,
                &vec![
                    (vec![ptr!(gen!("$a")), u16!()], vec![ptr!(gen!("$a"))]),
                    (vec![u16!(), ptr!(gen!("$a"))], vec![ptr!(gen!("$a"))]),
                    (vec![u16!(), u16!()], vec![u16!()]),
                ],
            )?;
        }
        Operator::Sub(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    sub   t0 t1
    push! t0
                            "
            );
            check_and_apply_multiple_stack_transitions(
                "-",
                span,
                stack_view,
                &vec![
                    (vec![ptr!(gen!("$a")), u16!()], vec![ptr!(gen!("$a"))]),
                    (vec![ptr!(gen!("$a")), ptr!(gen!("$a"))], vec![u16!()]),
                    (vec![u16!(), u16!()], vec![u16!()]),
                ],
            )?;
        }
        Operator::BOr(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    or    t0 t1
    push! t0
                            "
            );
            check_and_apply_stack_transition(
                "|",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::BAnd(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    and   t0 t1
    push! t0
                            "
            );
            check_and_apply_stack_transition(
                "&",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::BNot(span) => {
            tasm!(
                operation;;
                r"
    pop!  t0
    not   t0
    push! t0
                            "
            );
            check_and_apply_stack_transition("~", span, stack_view, &vec![u16!()], &[u16!()])?;
        }
        Operator::Sshr(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    sshr  t0 t1
    push! t0
                            "
            );
            check_and_apply_stack_transition(
                ">>",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::Shr(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    shr   t0 t1
    push! t0
                            "
            );
            check_and_apply_stack_transition(
                ">>>",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::Shl(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    shl   t0 t1
    push! t0
                            "
            );
            check_and_apply_stack_transition(
                "<<",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::Xor(span) => {
            tasm!(
                operation;;
                r"
    pop!  t1 t0
    xor   t0 t1
    push! t0
                            "
            );
            check_and_apply_stack_transition(
                "^",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }

        Operator::LOr(span) => {
            let true_label = format!("{block_id}_ortrue");

            tasm!(
                operation;;
                r"
    pop!  t1 t0
    imov  t2 1
    tst   t0
    jnz!  .{true_label}
    tst   t1
    jnz!  .{true_label}
    imov  t2 0
.{true_label}
    push! t2
                            "
            );
            check_and_apply_stack_transition(
                "||",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }

        Operator::LAnd(span) => {
            let false_label = format!("{block_id}_andfalse");

            tasm!(
                operation;;
                r"
    pop!  t1 t0
    imov  t2 0
    tst   t0
    jz!   .{false_label}
    tst   t1
    jz!   .{false_label}
    imov  t2 1
.{false_label}
    push! t2
                            "
            );
            check_and_apply_stack_transition(
                "&&",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }

        Operator::LNot(span) => {
            let false_label = format!("{block_id}_nottrue");

            tasm!(
                operation;;
                r"
    pop!  t0
    imov  t1 1
    tst   t0
    jz!   .{false_label}
    imov  t1 0
.{false_label}
    push! t1
                            "
            );
            check_and_apply_stack_transition("!", span, stack_view, &vec![u16!()], &[u16!()])?;
        }

        Operator::Eq(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                operation;;
                r"
    pop!  t1 t0
    imov  t2 1  # default value
    sub   t0 t1
    jz!   .{skip_label}
    imov  t2 0
.{skip_label}
    push! t2
                            "
            );
            check_and_apply_stack_transition(
                "=",
                span,
                stack_view,
                &vec![gen!("$a"), gen!("$a")],
                &[u16!()],
            )?;
        }
        Operator::Lt(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                operation;;
                r"
    pop!  t1 t0
    imov  t2 1  # default value
    sub   t0 t1
    jn!   .{skip_label}
    imov  t2 0
.{skip_label}
    push! t2
                            "
            );
            check_and_apply_stack_transition(
                "<",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::Lte(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                operation;;
                r"
    pop!  t1 t0
    imov  t2 0  # default value
    sub   t0 t1
    jp!   .{skip_label}
    imov  t2 1
.{skip_label}
    push! t2
                            "
            );
            check_and_apply_stack_transition(
                "<=",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::Gt(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                operation;;
                r"
    pop!  t1 t0
    imov  t2 1  # default value
    sub   t0 t1
    jp!   .{skip_label}
    imov  t2 0
.{skip_label}
    push! t2
                            "
            );
            check_and_apply_stack_transition(
                ">",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::Gte(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                operation;;
                r"
    pop!  t1 t0
    imov  t2 0  # default value
    sub   t0 t1
    jn!   .{skip_label}
    imov  t2 1
.{skip_label}
    push! t2
                            "
            );
            check_and_apply_stack_transition(
                ">=",
                span,
                stack_view,
                &vec![u16!(), u16!()],
                &[u16!()],
            )?;
        }
        Operator::Hole(span) => {
            println!("Stack at {span:?}: {stack_view:?}");
        }
        Operator::Call(span) => {
            // check top of the stack is a function pointer
            let top = &check_and_apply_stack_transition(
                "()",
                span,
                stack_view,
                &vec![ptr!(gen!("$a"))],
                &[],
            )?[0];

            let ret_label = generate_label_with_context(block_id, "retaddr");
            tasm!(
                operation;;
                r"
    imov! t0 .{ret_label}
    push  t5 t0
    pop   t0 
    jmp   t0
.{ret_label}
                "
            );

            if let Type::Pointer(1, fun) = top {
                if let Type::Function(FunctionType { in_t, out_t }) = &**fun {
                    check_and_apply_stack_transition("()", span, stack_view, in_t, out_t)?;
                } else {
                    return Err((*span, format!("Using the `()` operator requires function pointer to be on the top of the stack. Top of the stack is {top:?}")));
                }
            } else {
                return Err((*span, format!("Using the `()` operator requires function pointer to be on the top of the stack. Top of the stack is {top:?}")));
            }
        }
        Operator::Ptr(span, t) => {
            // check that this is a correct function
            let (name, func) =
                find_function(&t.name, &global_state.function_signatures, using_stack).ok_or((
                    *span,
                    format!("Was not able to find function signature of function {t:?}"),
                ))?;

            tasm!(
                operation;
                function_label(&name)
                ;
                r"
imov! t0 .{}
push  t0
                "
            );
            global_state
                .function_dependencies
                .add_dependency(function_state.function_name.clone(), name.clone());
            check_and_apply_stack_transition(
                &format!("ptr({})", &name),
                span,
                stack_view,
                &vec![],
                &[Type::Function(func).add_ref()],
            )?;
        }
        Operator::As(span, t) => {
            let parsed_t = Type::parse(t, &global_state.struct_defs, using_stack)
                .map_err(|_| (*span, format!("Could not parse type {t:?}.")))?;
            check_and_apply_stack_transition(
                &format!("as({parsed_t:?})"),
                span,
                stack_view,
                &vec![Type::new_generic("$a")],
                &[parsed_t],
            )?;
        }
        Operator::SizeOf(span, t) => {
            let parsed_t = Type::parse(t, &global_state.struct_defs, using_stack)
                .map_err(|_| (*span, format!("Could not parse type {t:?}.")))?;
            let size = parsed_t
                .type_size(&global_state.struct_defs)
                .err_with_span(span)?;
            tasm!(
                            operation;;
                            r"
    imov! t0 {size}
    push! t0
                            "
            );
            check_and_apply_stack_transition(
                &format!("sizeof({parsed_t:?})"),
                span,
                stack_view,
                &vec![],
                &[u16!()],
            )?;
        }
        Operator::StructAccess(span, member) => {
            // grab the type at the top of the stack
            let t = stack_view.peek()
                .ok_or((*span, format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is empty.")))?;
            let base_t = t.de_ref()
                .map_err(|_| (*span, format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is {stack_view:?}.")))?;

            if let Type::Struct(label) = base_t {
                let struct_def = global_state.struct_defs.get(&*label)
                    .ok_or((*span, format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is {stack_view:?}.")))?;
                let (offset, member_t) = struct_def.members.get(member).ok_or((
                    *span,
                    format!("Struct `{label}` does not have a member `{member}`."),
                ))?;

                tasm!(
                    operation;;
                    r"
    pop!  t0
    imov! t1 {offset}
    add   t0 t1
    push! t0
                    "
                );
                check_and_apply_stack_transition(
                    &format!(".{member}"),
                    span,
                    stack_view,
                    &vec![t],
                    &[member_t.add_ref()],
                )?;
            } else {
                return Err((*span, format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is {stack_view:?}.")));
            }
        }
        Operator::ConstArrayAccess(span, offset_literal) => {
            let offset = offset_literal.val as usize;
            let dropped_t = &check_and_apply_stack_transition(
                &format!("[{offset}]"),
                span,
                stack_view,
                &vec![ptr!(gen!("$a"))],
                &[ptr!(gen!("$a"))],
            )?[0];
            let offset_size = offset
                * dropped_t
                    .de_ref()
                    .err_with_span(span)?
                    .type_size(&global_state.struct_defs)
                    .err_with_span(span)?;
            tasm!(
                operation;;
                r"
    pop!  t0
    imov! t1 {offset_size}
    add   t0 t1
    push! t0
                "
            );
        }
        Operator::Return(span) => {
            // assert current stack is the function return stack
            let label = &function_state.function_out_label;

            if !stack_view.eq_vec(&function_state.function_out_stack) {
                return Err((*span,
                            format!("Cannot return here. Expecting stack contents of {:?} but found {stack_view:?}",
                                    &function_state.function_out_stack)));
            }

            let num_bindings = function_state.function_let_bindings;

            tasm!(
                operation;;
                r"
    # pop {num_bindings} elements from return stack
    imov! t0 {num_bindings}
    add   t5 t0
    jmp!  .{label}
                "
            );
        }
    }
    Ok(operation)
}
