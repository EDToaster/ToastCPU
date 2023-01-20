use crate::emit::types::{GlobalState, Type, TypeSize};
use crate::tl_y::Global;

// return emitted code, label, initial value
pub fn emit_global(g: &Global, global_state: &GlobalState) -> Result<(String, String, isize, Type), String> {
    let global_name = &g.name.name;
    let label = &format!("variable_alloc_{global_name}");

    let type_name = &*g.var_type.name;

    let t = Type::parse(type_name, &global_state.struct_defs)
        .map_err(|_| format!("Type {type_name} not in scope"))?;

    let size = g.size * t.type_size(&global_state.struct_defs)?;
    let prog = format!(r"
.{label} [{size}]
    ");
    Ok((prog, label.to_string(), g.val.val, t))
}
