use lrpar::Span;

use crate::emit::function::{emit_function, emit_isr};
use crate::emit::string_defs::emit_string_defs;
use crate::emit::types::*;
use crate::is_verbose;
use crate::parser_util::types::*;
use crate::util::dep_graph::DependencyGraph;
use crate::util::gss::Stack;
use crate::util::labels::global_label;
use std::collections::{HashMap, HashSet};

fn gather_struct_declarations(
    m: &Module,
    module_prefix: &str,
    usings: &Stack<String>,
    structs: &mut HashMap<String, (Stack<String>, StructDef)>,
) -> Result<(), (Span, String)> {
    let mut usings = usings.clone();

    for using in &m.usings {
        usings.push(format!("{}::", using.name.name));
    }

    for s in &m.struct_defs {
        let name = format!("{module_prefix}{}", s.name.name);
        if structs.contains_key(&name) {
            return Err((Span::new(0, 0), format!("Struct `{name}` is already defined.")));
        }
        structs.insert(name, (usings.clone(), s.clone()));
    }

    // submodules
    for module in &m.modules {
        gather_struct_declarations(
            &module.module,
            &format!("{module_prefix}{}::", &module.name.name),
            &usings,
            structs,
        )?;
    }

    Ok(())
}

pub fn gather_definitions(
    m: &Module,
    module_prefix: &str,
    global_state: &mut GlobalState,
    using_stack: &mut Stack<String>,
) -> Result<(), String> {
    // gather definitions for submodules first
    for module in &m.modules {
        let submod_usings: Vec<String> = module
            .module
            .usings
            .iter()
            .map(|e| format!("{}::", e.name.name))
            .collect();
        let mut using_stack = using_stack.clone();
        for submod_using in submod_usings {
            using_stack.push(submod_using);
        }
        gather_definitions(
            &module.module,
            &format!("{module_prefix}{}::", &module.name.name),
            global_state,
            &mut using_stack,
        )?;
    }

    // preprocess function signatures
    for f in &m.functions {
        let name = &f.name.name;
        let qualified_name = format!("{module_prefix}{name}");
        if global_state
            .function_signatures
            .contains_key(&qualified_name)
        {
            return Err(format!("Function {qualified_name} is already defined."));
        }
        global_state.function_signatures.insert(
            qualified_name.clone(),
            Type::parse_func_def(&f.type_def, &global_state.struct_defs, using_stack).map_err(
                |e| format!("Could not parse some types in function {qualified_name}: {e}"),
            )?,
        );
    }

    // preprocess global variables
    for g in &m.globals {
        let name = &g.name.name;
        let lex_type = &g.var_type;
        let t = Type::parse(lex_type, &global_state.struct_defs, using_stack)
            .map_err(|_| format!("Type {lex_type:?} not in scope"))?;

        global_state
            .globals
            .insert(format!("{module_prefix}{name}"), (t, g.size, g.val.val));
    }

    // preprocess inlines
    for inline in &m.inlines {
        let name = &inline.name.name;
        global_state
            .inlines
            .insert(format!("{module_prefix}{name}"), inline.statement.clone());
    }

    Ok(())
}

pub fn emit_functions(
    m: &Module,
    module_prefix: &str,
    global_state: &mut GlobalState,
    function_map: &mut HashMap<String, String>,
    using_stack: &Stack<String>,
) -> Result<(), (Span, String)> {
    // emit submodule functions
    for module in &m.modules {
        let submod_usings: Vec<String> = module
            .module
            .usings
            .iter()
            .map(|e| format!("{}::", e.name.name))
            .collect();
        let mut using_stack = using_stack.clone();
        for submod_using in submod_usings {
            using_stack.push(submod_using);
        }
        emit_functions(
            &module.module,
            &format!("{module_prefix}{}::", &module.name.name),
            global_state,
            function_map,
            &using_stack,
        )?;
    }

    for f in &m.functions {
        match &*format!("{module_prefix}{}", &f.name.name) {
            s @ "isr" => {
                function_map.insert(
                    s.to_string(),
                    emit_isr(f, global_state, using_stack)?,
                );
            }
            s => {
                function_map.insert(
                    s.to_string(),
                    emit_function(f, module_prefix, global_state, using_stack)?,
                );
            }
        }
    }

    Ok(())
}

pub fn emit_root_module(m: &Module) -> Result<String, (Span, String)> {
    let mut global_state = GlobalState {
        function_signatures: HashMap::new(),
        function_dependencies: DependencyGraph::default(),
        struct_defs: HashMap::new(),
        string_allocs_counter: 0,
        string_allocs: HashMap::new(),
        globals: HashMap::new(),
        inlines: HashMap::new(),
    };

    let mod_usings: Vec<String> = m
        .usings
        .iter()
        .map(|e| format!("{}::", e.name.name))
        .collect();
    let mut using_stack = Stack::empty();
    using_stack.push("".to_owned());
    for using in mod_usings {
        using_stack.push(using);
    }

    // gather struct declarations
    let mut struct_names: HashMap<_, _> = HashMap::new();
    let usings = Stack::from(&vec!["".to_string()]);
    gather_struct_declarations(m, "", &usings, &mut struct_names)?;

    // resolve members
    let mut seen: HashSet<String> = HashSet::new();

    for name in struct_names.keys() {
        if seen.contains(name) {
            continue;
        }
        Type::resolve_struct_size(
            name,
            &mut global_state.struct_defs,
            &struct_names,
            &mut seen,
        ).map_err(|s| (Span::new(0, 0), s))?;
    }

    // all structs should have sizes populated

    gather_definitions(m, "", &mut global_state, &mut using_stack).map_err(|s| (Span::new(0, 0), s))?;

    // grab global variables
    let mut global_prog: String = String::new();

    let mut global_init_routine = r#"
    push! t0
    "#
    .to_string();

    for (identifier, (t, num, init)) in &global_state.globals {
        let label = global_label(identifier);
        let size_words = t.type_size(&global_state.struct_defs).map_err(|s| (Span::new(0, 0), s))? * (*num) as usize;

        tasm!(
            global_prog;;
            r"
.{label} [{size_words}]
            "
        );

        // todo: better array initialize in global variables
        tasm!(
            global_init_routine;;
            r"
    imov! t0 {init}
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
    imov! isr .fn_isr
    # initialize ret stack ptrs
    imov! t5 .ret_stack
    imov! t0 0x03FF
    add   t5 t0
    imov! t1 .reset_ret
    push  t5 t1
    # initialize our global variables
    call! .init_globals
    jmp!  .fn_main
.reset_ret
    halt
#end .reset

fn .init_globals
{global_init_routine}
#end .init_globals

{global_prog}
"#
    );

    let mut function_map: HashMap<String, String> = HashMap::new();
    // todo: make this a dependency graph
    global_state
        .function_dependencies
        .roots
        .insert("isr".to_string());
    global_state
        .function_dependencies
        .roots
        .insert("main".to_string());

    emit_functions(m, "", &mut global_state, &mut function_map, &using_stack)?;

    let mut functions: String = String::new();
    let used_functions = global_state.function_dependencies.calculate_used();

    if is_verbose() {
        println!(
            "{} functions defined, {} will be emitted after tree shaking",
            function_map.len(),
            used_functions.len()
        );
        println!("These functions will not be emitted:");

        for func in function_map.keys() {
            if !used_functions.contains(func) {
                println!("- {func}");
            }
        }
    }

    for (f, v) in function_map.iter() {
        if used_functions.contains(f) {
            functions.push_str(v);
        }
    }

    // provide default no-op isr if none was provided in code.
    let isr_found = function_map.contains_key("isr");
    if !isr_found {
        tasm!(prog;;
        r"
fn .fn_isr
    isr!
    rti!
#end .fn_isr
        ");
    }

    // gather string defs
    prog.push_str(&emit_string_defs(&global_state));

    prog.push_str(&functions);
    Ok(prog)
}
