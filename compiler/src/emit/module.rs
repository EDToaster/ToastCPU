use std::collections::HashMap;
use crate::emit::function::{emit_function, emit_isr};
use crate::emit::global::emit_global;
use crate::emit::string_defs::emit_string_defs;
use crate::emit::types::{GlobalState, parse_types, tasm, Type};
use crate::tl_y::{Identifier, Module};

// todo: do Result<String> instead
// todo: allow for multiple modules (and includes)
pub fn emit_module(m: &Module) -> Result<String, String> {
    // preprocess function sizes
    let mut function_signatures: HashMap<String, (Vec<Type>, Vec<Type>)> = HashMap::new();
    for f in &m.functions {
        let name = &f.name.name;
        function_signatures.insert(name.clone(), parse_types(&f.in_t, &f.out_t).map_err(|_| "Could not parse some types!".to_string())?);
    }

    // grab global variables
    let mut global_prog: String = String::new();
    let mut globals: HashMap<String, (String, Type)> = HashMap::new();

    let mut global_init_routine = r#"
    push! t0
    "#.to_string();

    for g in &m.globals {
        let (emitted, label, val, var_type) = emit_global(g)?;
        global_prog.push_str(emitted.as_str());
        globals.insert(g.name.name.clone(), (label.clone(), var_type));

        tasm!(
            global_init_routine;;
            r"
    imov!  t0 {val}
    str!   .{label} t0
            "
        );
    }

    global_init_routine.push_str(r"
    pop! t0
    jmpr
                                        ");

    let mut global_state = GlobalState {
        function_signatures,
        string_allocs_counter: 0,
        string_allocs: HashMap::new(),
        globals,
    };

    let mut prog = format!(r#"
.reset
    # setup isr and jump to main
    imov!   isr .isr

    # initialize ret stack ptrs
    imov!   t0 .ret_stack
    imov!   t1 .reset_ret

    str     t0 t1
    iadd    t0 1
    str!    .ret_stack_ptr t0

    # initialize our global variables
    call!   .init_globals
    jmp!    .main
.reset_ret
    halt

.init_globals
{global_init_routine}

{global_prog}

# allocate 1024 words on the heap
.ret_stack_ptr [1]
.ret_stack [0x0400]

"#);

    let mut isr_found = false;

    let mut functions: String = String::new();

    for f in &m.functions {
        match &*f.name.name {
            "isr" => {
                isr_found = true;
                functions.push_str(&*emit_isr(&f, &mut global_state).map_err(|(_,b)| b)?);
            }
            _ => {
                functions.push_str(&*emit_function(&f, &mut global_state).map_err(|(_,b)| b)?);
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
    prog.push_str(&*emit_string_defs(&global_state));

    prog.push_str(&*functions);
    Ok(prog)
}