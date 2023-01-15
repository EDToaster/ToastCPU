use lrpar::Span;
use crate::emit::type_check::{check_and_apply_multiple_stack_transitions, check_and_apply_stack_transition};
use crate::emit::types::{FunctionState, tasm, Type};
use crate::tl_y::Operator;

pub fn emit_operator(block_id: &str, r: &Operator, function_state: &mut FunctionState) -> Result<String, (Span, String)> {
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
            let parsed_t = Type::parse(&t.name).map_err(|_| (span.clone(), format!("Could not parse type {}.", &t.name)))?;
            check_and_apply_stack_transition(&*format!("as({parsed_t:?})"), &span, function_state, &vec![Type::new_generic("$a")], &vec![parsed_t])?;
        }
    }
    Ok(operation)
}