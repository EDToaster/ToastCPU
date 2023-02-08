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
) -> Result<String, (Span, String)> {
    let mut block = "".to_string();

    for i in &b.body {
        let statement = emit_statement(
            block_id,
            i,
            global_state,
            function_state,
            stack_view,
            using_stack,
        )?;
        tasm!(
          block;;
            r"
{statement}
            "
        );
    }

    Ok(block)
}
