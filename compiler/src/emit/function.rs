use crate::emit::block::emit_block;
use crate::emit::types::GlobalState;
use crate::emit::types::*;
use crate::tl_y::Function;
use crate::util::gss::Stack;
use lrpar::Span;

pub fn emit_isr(f: &Function, global_state: &mut GlobalState) -> Result<String, (Span, String)> {
    let func_name = &f.name.name;
    let function_out_label = format!("{func_name}_exit");

    if !f.in_t.is_empty() || !f.out_t.is_empty() {
        return Err((
            f.span,
            "Interrupt service routine (isr) cannot take in or emit stack items.".to_string(),
        ));
    }

    let mut function_state = FunctionState {
        current_bindings: vec![],
        function_out_stack: vec![],
        function_out_label: function_out_label.clone(),
        function_let_bindings: 0,
        function_name: "isr".to_string(),
    };

    let mut stack_view = Stack::empty();

    let block = emit_block(
        &format!("{func_name}_body"),
        &f.body,
        global_state,
        &mut function_state,
        &mut stack_view,
    )
    .map_err(|(span, err)| (span, format!("{func_name}: {err}")))?;

    if !stack_view.is_empty() {
        return Err((f.span, format!("Interrupt service routine (isr) has to handle all stack items, but {stack_view:?} remains on the stack.")));
    }

    let mut func = String::new();
    tasm!(func;;
    r"
fn .isr
    isr!
    push! p0 p1 p2 p3 v0 t0 t1 t2 t3 t4
{block}
.{function_out_label}
    pop!  t4 t3 t2 t1 t0 v0 p3 p2 p1 p0
    rti!
#end .isr
    ");
    Ok(func)
}

pub fn emit_function(
    f: &Function,
    global_state: &mut GlobalState,
) -> Result<String, (Span, String)> {
    // at this point we really only care about one function
    let func_name = &f.name.name;
    let func_exit = format!("{func_name}_exit",);
    let mut func = format!("\nfn .{func_name}\n",);

    let (in_t, out_t) = global_state.function_signatures.get(func_name).unwrap();

    let mut function_state = FunctionState {
        current_bindings: vec![],
        function_out_stack: out_t.clone(),
        function_out_label: func_exit.clone(),
        function_let_bindings: 0,
        function_name: func_name.clone(),
    };

    let mut stack_view: Stack<Type> = Stack::from(in_t);

    let block = emit_block(
        &format!("{func_name}_body"),
        &f.body,
        global_state,
        &mut function_state,
        &mut stack_view,
    )
    .map_err(|(span, err)| (span, format!("{func_name}: {err}")))?;

    let (_, out_t) = global_state.function_signatures.get(func_name).unwrap();

    if !stack_view.eq_vec(out_t) {
        return Err((
            f.span,
            format!("Function `{func_name}` signature expects return of {out_t:?}, but {stack_view:?} was gotten"),
        ));
    }

    func.push_str(&format!(
        r"
{block}
    "
    ));

    func.push_str(&format!(
        r"
.{func_exit}
    pop   t0 t5
    jmp   t0
#end .{}
",
        f.name.name
    ));
    Ok(func)
}
