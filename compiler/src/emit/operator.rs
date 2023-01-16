use lrpar::Span;
use crate::emit::type_check::{check_and_apply_multiple_stack_transitions, check_and_apply_stack_transition};
use crate::emit::types::{ErrWithSpan, FunctionState, GlobalState, tasm, Type, TypeSize};
use crate::emit::types::Type::U16;
use crate::tl_y::Operator;

pub fn emit_operator(block_id: &str, r: &Operator, global_state: &GlobalState, function_state: &mut FunctionState) -> Result<String, (Span, String)> {
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
            check_and_apply_multiple_stack_transitions("+", &span, function_state,
            &vec![
                    (
                        vec![
                            Type::Pointer(1, Box::new(Type::new_generic("$a"))),
                            Type::U16,
                        ], vec![Type::Pointer(1, Box::new(Type::new_generic("$a")))]
                    ),
                    (
                        vec![
                            Type::U16,
                            Type::Pointer(1, Box::new(Type::new_generic("$a"))),
                        ], vec![Type::Pointer(1, Box::new(Type::new_generic("$a")))]
                    ),
                    (
                        vec![
                            Type::U16, Type::U16
                        ], vec![Type::U16]
                    ),
                ])?;
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
            check_and_apply_multiple_stack_transitions("-", &span, function_state,
           &vec![
                   (
                       vec![
                           Type::Pointer(1, Box::new(Type::new_generic("$a"))),
                           Type::U16,
                       ], vec![Type::Pointer(1, Box::new(Type::new_generic("$a")))]
                   ),
                   (
                       vec![
                           Type::Pointer(1, Box::new(Type::new_generic("$a"))),
                           Type::Pointer(1, Box::new(Type::new_generic("$a"))),
                       ], vec![Type::U16]
                   ),
                   (
                       vec![
                           Type::U16,
                           Type::U16,
                       ], vec![Type::U16]
                   ),
               ])?;
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
            check_and_apply_stack_transition("|", span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_and_apply_stack_transition("&", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_and_apply_stack_transition("~", &span, function_state, &vec![Type::U16], &vec![Type::U16])?;
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
            check_and_apply_stack_transition(">>", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_and_apply_stack_transition(">>>", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_and_apply_stack_transition("<<", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_and_apply_stack_transition("^", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }

        Operator::LOr(span) => {
            let true_label = format!("{block_id}_ortrue");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition("||", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }

        Operator::LAnd(span) => {
            let false_label = format!("{block_id}_andfalse");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition("&&", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }

        Operator::LNot(span) => {
            let false_label = format!("{block_id}_nottrue");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition("!", &span, function_state, &vec![Type::U16], &vec![Type::U16])?;
        }

        Operator::Eq(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition("=", &span, function_state, &vec![Type::new_generic("$a"), Type::new_generic("$a")], &vec![Type::U16])?;
        }
        Operator::Lt(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition("<", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }
        Operator::Lte(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition("<=", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }
        Operator::Gt(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition(">", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }
        Operator::Gte(span) => {
            let skip_label = format!("{block_id}_opskip");

            tasm!(
                            operation;;
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
            check_and_apply_stack_transition(">=", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }
        Operator::Hole(span) => {
            println!("Stack at {span:?}: {:?}", function_state.stack_view);
        }
        Operator::As(span, t) => {
            let parsed_t = Type::parse(&t.name, &global_state.struct_defs).map_err(|_| (span.clone(), format!("Could not parse type {}.", &t.name)))?;
            check_and_apply_stack_transition(&*format!("as({parsed_t:?})"), &span, function_state, &vec![Type::new_generic("$a")], &vec![parsed_t])?;
        }
        Operator::SizeOf(span, t) => {
            let parsed_t = Type::parse(&t.name, &global_state.struct_defs).map_err(|_| (span.clone(), format!("Could not parse type {}.", &t.name)))?;
            let size = parsed_t.type_size(&global_state.struct_defs).err_with_span(span)?;
            tasm!(
                            operation;;
                            r"
    imov! t0 {size}
    push! t0
                            "
            );
            check_and_apply_stack_transition(&*format!("sizeof({parsed_t:?})"), &span, function_state, &vec![], &vec![U16])?;
        }
        Operator::StructAccess(span, member) => {
            // grab the type at the top of the stack
            let t = function_state.stack_view.last()
                .ok_or((span.clone(), format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is empty.")))?;
            let base_t = t.de_ref()
                .map_err(|_| (span.clone(), format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is {:?}.",
                    function_state.stack_view)))?;

            if let Type::Struct(label) = base_t {
                let struct_def = global_state.struct_defs.get(&*label)
                    .ok_or((span.clone(), format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is {:?}.",
                                                  function_state.stack_view)))?;
                let (offset, member_t) = struct_def.members.get(member)
                    .ok_or((span.clone(), format!("Struct `{label}` does not have a member `{member}`.")))?;

                tasm!(
                    operation;;
                    r"
    pop! t0
    imov! t1 {offset}
    add  t0 t1
    push! t0
                    "
                );
                check_and_apply_stack_transition(&*format!(".{member}"), &span, function_state, &vec![t.clone()], &vec![member_t.add_ref()])?;
            } else {
                return Err((span.clone(), format!("Struct access .{member} requires one struct pointer at the top of the stack. Current stack is {:?}.",
                                                  function_state.stack_view)))
            }

        }
        Operator::ConstArrayAccess(span, offset_literal) => {
            let offset = offset_literal.val;
            let dropped_t = &check_and_apply_stack_transition(&*format!("[{offset}]"), &span, function_state,
                                             &vec![Type::Pointer(1, Box::new(Type::new_generic("$a")))],
                                             &vec![Type::Pointer(1, Box::new(Type::new_generic("$a")))])?[0];
            let offset_size = offset *
                dropped_t
                    .de_ref()
                    .err_with_span(span)?
                    .type_size(&global_state.struct_defs)
                    .err_with_span(span)?;
            tasm!(
                operation;;
                r"
    pop! t0
    imov! t1 {offset_size}
    add  t0 t1
    push! t0
                "
            );
        }
    }
    Ok(operation)
}