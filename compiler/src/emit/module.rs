use crate::emit::function::{emit_function, emit_isr};
use crate::emit::global::emit_global;
use crate::emit::string_defs::emit_string_defs;
use crate::emit::types::{parse_types, tasm, GlobalState, StructDefinition, Type, TypeSize};
use crate::tl_y::Module;
use std::collections::HashMap;

// todo: allow for multiple modules (and includes)
pub fn emit_module(m: &Module) -> Result<String, String> {
    let mut global_state = GlobalState {
        function_signatures: HashMap::new(),
        struct_defs: HashMap::new(),
        string_allocs_counter: 0,
        string_allocs: HashMap::new(),
        globals: HashMap::new(),
        inlines: HashMap::new(),
    };

    // preprocess struct defs
    for s in &m.struct_defs {
        let name = &s.name.name;
        let mut members: HashMap<String, (isize, Type)> = HashMap::new();
        let mut counter = 0;
        for member in &s.members {
            let member_name = &member.name.name;
            let t = Type::parse(&member.var_type.name, &global_state.struct_defs)
                .map_err(|_| format!("Could not parse Type `{}`", &member.var_type.name))?;
            members.insert(member_name.clone(), (counter, t.clone()));
            counter += t.type_size(&global_state.struct_defs)? * member.size;
        }
        global_state.struct_defs.insert(
            name.clone(),
            StructDefinition {
                size: counter,
                members,
            },
        );
    }

    // preprocess function signatures
    for f in &m.functions {
        let name = &f.name.name;
        global_state.function_signatures.insert(
            name.clone(),
            parse_types(&f.in_t, &f.out_t, &global_state.struct_defs)
                .map_err(|_| "Could not parse some types!".to_string())?,
        );
    }

    // grab global variables
    let mut global_prog: String = String::new();

    let mut global_init_routine = r#"
    push! t0
    "#
    .to_string();

    // preprocess inlines
    for inline in &m.inlines {
        global_state
            .inlines
            .insert(inline.name.name.clone(), inline.statement.clone());
    }

    for g in &m.globals {
        let (emitted, label, val, var_type) = emit_global(g, &global_state)?;
        global_prog.push_str(emitted.as_str());
        global_state
            .globals
            .insert(g.name.name.clone(), (label.clone(), var_type));

        // todo: better array initialize in global variables
        tasm!(
            global_init_routine;;
            r"
    imov! t0 {val}
    str!  .{label} t0
            "
        );
    }

    global_init_routine.push_str(
        r"
    pop!  t0
    jmpr
                                        ",
    );

    let mut prog = format!(
        r#"
# allocate 1024 words on the heap
.ret_stack [0x0400]

fn .reset
    # setup isr and jump to main
    imov! isr .isr
    # initialize ret stack ptrs
    imov! t5 .ret_stack
    imov! t0 0x03FF
    add   t5 t0
    imov! t1 .reset_ret
    push  t5 t1
    # initialize our global variables
    call! .init_globals
    jmp!  .main
.reset_ret
    halt
#end .reset

fn .init_globals
{global_init_routine}
#end .init_globals

{global_prog}
"#
    );

    let mut isr_found = false;

    let mut functions: String = String::new();

    for f in &m.functions {
        match &*f.name.name {
            "isr" => {
                isr_found = true;
                functions.push_str(&emit_isr(f, &mut global_state).map_err(|(_, b)| b)?);
            }
            _ => {
                functions.push_str(&emit_function(f, &mut global_state).map_err(|(_, b)| b)?);
            }
        }
    }

    // provide default no-op isr if none was provided in code.
    if !isr_found {
        tasm!(prog;;
        r"
fn .isr
    isr!
    rti!
#end .isr
        ");
    }

    // gather string defs
    prog.push_str(&emit_string_defs(&global_state));

    prog.push_str(&functions);
    Ok(prog)
}
