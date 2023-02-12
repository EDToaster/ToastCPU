use std::collections::HashMap;

use crate::{emit::types::{tasm, GlobalState}, util::labels::const_label, is_verbose};

use super::types::Marker;

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

pub fn emit_consts(global_state: &GlobalState) -> String {
    let mut words = 0;
    let mut string_defs: String = String::new();

    for alloc in &global_state.const_allocs.allocs {
        // translate markers to map
        let mut marker_map: HashMap<usize, Vec<String>> = HashMap::new();
        for Marker { id, offset } in &alloc.markers {
            marker_map.entry(*offset).or_insert_with(|| vec![]).push(id.clone());
        }

        for (i, u) in alloc.seq.iter().enumerate() {
            if let Some(ids) = marker_map.get(&i) {
                for id in ids {
                    let label = const_label(id);
                    tasm!(
                        string_defs;;
                        ".{label}\n"
                    );
                }
            }
            tasm!(
                string_defs;
                char_comment(*u);
                "    {u:#06X}{}\n"
            );
            words += 1;
        }
    }

    if is_verbose() {
        println!("Strings take up {words} words")
    }

    string_defs
}
