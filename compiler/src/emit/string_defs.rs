use crate::emit::types::{tasm, GlobalState};

fn char_comment(i: u16) -> String {
    match i {
        0x20..=0x7E => format!(" # '{} '", i as u8 as char),
        0x09 => r" # '\t'".to_string(),
        0x0A => r" # '\n'".to_string(),
        0x0D => r" # '\r'".to_string(),
        0x00 => r" # '\0'".to_string(),
        _ => "".to_string(),
    }
}

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
                string_defs;
                char_comment(*u);
                "    {u:#06X}{}\n"
            );
        }
    }

    string_defs
}
