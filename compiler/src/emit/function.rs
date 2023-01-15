use lrpar::Span;
use crate::emit::block::emit_block;
use crate::emit::types::GlobalState;
use crate::tl_y::Function;
use crate::emit::types::*;


pub fn emit_function(f: &Function, global_state: &mut GlobalState) -> Result<String, (Span, String)> {
    // at this point we really only care about one function
    let func_name = &f.name.name;
    let func_exit = format!("{}_exit", func_name);
    let mut func = format!("fn .{}\n", func_name);

    let (in_t, _) = global_state.function_signatures.get(func_name).unwrap();

    let stack_view: Vec<Type> = (*in_t).clone();

    let mut function_state = FunctionState {
        current_bindings: vec![],
        stack_view,
    };

    let block = emit_block(&*format!("{func_name}_body"),
                           &f.body, global_state, &mut function_state)
        .map_err(|(span, err)| (span, format!("{func_name}: {err}")))?;

    let (_, out_t) = global_state.function_signatures.get(func_name).unwrap();

    if out_t != &function_state.stack_view {
        return Err((f.span.clone(), format!("Function signature expects return of {:?}, but {:?} was gotten", out_t, function_state.stack_view)));
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