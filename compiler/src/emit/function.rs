use lrpar::Span;
use crate::emit::block::emit_block;
use crate::emit::types::GlobalState;
use crate::tl_y::Function;
use crate::emit::types::*;

pub fn emit_isr(f: &Function, global_state: &mut GlobalState) -> Result<String, (Span, String)> {
    let func_name = &f.name.name;
    let function_out_label = format!("{func_name}_exit");

    if f.in_t.len() > 0 || f.out_t.len() > 0 {
        return Err((f.span.clone(), format!("Interrupt service routine (isr) cannot take in or emit stack items.")))
    }

    let mut function_state = FunctionState {
        current_bindings: vec![],
        stack_view: vec![],
        function_out_stack: vec![],
        function_out_label: function_out_label.clone(),
        function_let_bindings: 0,
    };

    let block = emit_block(&*format!("{func_name}_body"),
                           &f.body, global_state, &mut function_state)
        .map_err(|(span, err)| (span, format!("{func_name}: {err}")))?;

    if !function_state.stack_view.is_empty() {
        return Err((f.span.clone(), format!("Interrupt service routine (isr) has to handle all stack items, but {:?} remains on the stack.", function_state.stack_view)));
    }

    let mut func = String::new();
    tasm!(func;;
    r"
fn .isr
    isr!
{block}
.{function_out_label}
    rti!
    ");
    Ok(func)
}

pub fn emit_function(f: &Function, global_state: &mut GlobalState) -> Result<String, (Span, String)> {
    // at this point we really only care about one function
    let func_name = &f.name.name;
    let func_exit = format!("{func_name}_exit", );
    let mut func = format!("fn .{func_name}\n", );

    let (in_t, out_t) = global_state.function_signatures.get(func_name).unwrap();

    let stack_view: Vec<Type> = (*in_t).clone();

    let mut function_state = FunctionState {
        current_bindings: vec![],
        function_out_stack: out_t.clone(),
        function_out_label: func_exit.clone(),
        function_let_bindings: 0,
        stack_view,
    };

    let block = emit_block(&*format!("{func_name}_body"),
                           &f.body, global_state, &mut function_state)
        .map_err(|(span, err)| (span, format!("{func_name}: {err}")))?;

    let (_, out_t) = global_state.function_signatures.get(func_name).unwrap();

    if out_t != &function_state.stack_view {
        return Err((f.span.clone(), format!("Function `{func_name}` signature expects return of {:?}, but {:?} was gotten", out_t, function_state.stack_view)));
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
    Ok(func)
}