use crate::emit::types::{parse_types, Type};
use crate::tl_y::Global;

// return emitted code, label, initial value
pub fn emit_global(g: &Global) -> Result<(String, String, isize, Type), String> {
    let global_name = &g.name.name;
    let label = &format!("variable_alloc_{global_name}");
    // todo: change size to be dynamic based on type
    let size = g.size;
    let prog = format!(r"
.{label} [{size}]
    ");
    Ok((prog, label.to_string(), g.val.val, Type::parse(&g.var_type.name).map_err(|_| "Global variable type issue")?))
}
