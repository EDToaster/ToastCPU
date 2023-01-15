use crate::emit::types::{GlobalState, tasm};

pub fn emit_string_defs(global_state: &GlobalState) -> String {
    let mut string_defs: String = String::new();

    for (k, v) in global_state.string_allocs.iter() {
        tasm!(
            string_defs;;
            r"
.{v}
"
        );

        for u in k {
            tasm!(
                string_defs;;
                "    0x{u:X}\n"
            );
        }
    }

    string_defs
}
