use lrpar::Span;
use crate::emit::block::check_stack_and_mutate;
use crate::emit::types::{FunctionState, tasm, Type};
use crate::tl_y::Operator;

// allows add and sub operators to include pointer arithmetic
fn check_add_sub_and_mutate(r: &str, span: &Span, function_state: &mut FunctionState) -> Result<(), (Span, String)> {
    // assert at most 1 of the top two is a pointer
    let prev_size = function_state.stack_view.len();
    if prev_size < 2 {
        Err((span.clone(), format!("Operator `{r}` requires 2 numeric elements at the top of the stack. Current stack is {:?}", function_state.stack_view)))
    } else {
        let a = &function_state.stack_view.pop().expect("whoopsies");
        let b = &function_state.stack_view.pop().expect("whoopsies");
        let result_type = match (a, b) {
            (Type::Pointer(i1, b1), Type::Pointer(i2, b2)) => if i1 == i2 && b1 == b2 {
                Ok(&Type::U16)
            } else {
                Err((span.clone(), format!("Operator `{r}` requires pointer operands to be the same type. Types were [{b:?}, {a:?}].")))
            },
            (Type::Pointer(_, _), Type::U16) => Ok(a),
            (Type::U16, Type::Pointer(_, _)) => Ok(b),
            (Type::U16, Type::U16) => Ok(a),
            _ => Err((span.clone(), format!("Operator `{r}` requires operands of two numeric types. At most one of the operands can be a pointer. Types were [{b:?}, {a:?}]."))),
        }?;
        function_state.stack_view.push(result_type.clone());

        Ok(())
    }
}

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
            // todo: allow for pointer arith
            check_add_sub_and_mutate("+", &span, function_state)?;
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
            check_add_sub_and_mutate("-", &span, function_state)?;
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
            check_stack_and_mutate("|", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("&", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("~", &span, function_state, &vec![Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate(">>", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate(">>>", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("<<", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("^", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("||", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("&&", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("!", &span, function_state, &vec![Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("=", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("<", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate("<=", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate(">", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
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
            check_stack_and_mutate(">=", &span, function_state, &vec![Type::U16, Type::U16], &vec![Type::U16])?;
        }
        Operator::Hole(span) => {
            println!("Stack at {span:?}: {:?}", function_state.stack_view);
        }
    }
    Ok(operation)
}