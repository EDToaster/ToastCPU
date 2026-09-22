use crate::emit::statement::emit_statement;
use crate::emit::types::*;
use crate::parser_util::types::*;
use crate::util::gss::Stack;
use lrpar::Span;

pub fn emit_block(
    block_id: &str,
    b: &Block,
    global_state: &mut GlobalState,
    function_state: &mut FunctionState,
    stack_view: &mut Stack<Type>,
    using_stack: &Stack<String>,
) -> Result<(String, Flow), (Span, String)> {
    let mut block = "".to_string();
    let mut flow = Flow::Falls;

    for i in &b.body {
        // Once a statement diverges (e.g. via `return`), any statement after it
        // in the block can never execute. Reject it as unreachable code.
        if flow.diverges() {
            return Err((
                i.span(),
                "Unreachable code: this statement can never be reached because control has already left via an earlier `return`.".to_string(),
            ));
        }

        let (statement, sflow) = emit_statement(
            block_id,
            i,
            global_state,
            function_state,
            stack_view,
            using_stack,
        )?;
        flow = sflow;

        // Record the stack state after this statement, for editor stack
        // inspection (no-op unless probe recording is active).
        crate::emit::probe::push_probe(
            i.span(),
            crate::emit::probe::ProbeKind::Statement,
            stack_view.to_vec(),
            &function_state.function_name,
        );
        tasm!(
          block;;
            r"
{statement}
            "
        );
    }

    Ok((block, flow))
}
