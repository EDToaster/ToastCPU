use lrpar::Span;
use crate::emit::statement::emit_statement;
use crate::emit::types::*;
use crate::tl_y::*;
use crate::util::gss::Stack;

pub fn emit_block(block_id: &str, b: &Block, global_state: &mut GlobalState, function_state: &mut FunctionState, stack_view: &mut Stack<Type>) -> Result<String, (Span, String)> {
    let mut block = "".to_string();

    for i in &b.body {
        let statement = emit_statement(block_id, i, global_state, function_state, stack_view)?;
        tasm!(
          block;;
            r"
{statement}
            "
        );
    }

    Ok(block)
}