use crate::parser_util::types::{FuncType, Identifier, LexType, Statement, StructDef};
use crate::util::dep_graph::DependencyGraph;
use crate::util::gss::Stack;
use core::fmt;
use std::cmp::min;
use lrpar::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;

macro_rules! tasm {
    ($prog:ident; $($params:expr),*; $asm:literal) => {
        $prog.push_str(&*format!($asm, $($params),*));
    }
}

#[derive(Debug)]
pub struct StructDefinition {
    pub members: HashMap<String, (usize, Type)>,
    pub size: usize,
}

pub trait TypeSize {
    fn type_size(&self, structs: &HashMap<String, StructDefinition>) -> Result<usize, String>;
}

#[derive(Clone, Eq, PartialEq)]
pub enum Type {
    U16,
    Bool,
    Pointer(isize, Box<Type>),
    Struct(String),
    Generic(String),
    Function(FunctionType),
}

#[derive(Clone, Eq, PartialEq)]
pub struct FunctionType {
    pub in_t: Vec<Type>,
    pub out_t: Vec<Type>,
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:?} -> {:?})", self.in_t, self.out_t)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::U16 => {
                write!(f, "u16")
            }
            Type::Bool => {
                write!(f, "bool")
            }
            Type::Pointer(i, t) => write!(f, "{:?}{}", t, "*".repeat(*i as usize)),
            Type::Struct(s) => write!(f, "{s}"),
            Type::Generic(s) => write!(f, "${s}"),
            Type::Function(func) => write!(f, "{func:?}"),
        }
    }
}

pub trait ErrWithSpan<T, U> {
    fn err_with_span(self, span: &Span) -> Result<T, (Span, U)>
    where
        Self: Sized,
        T: Sized,
        U: Sized;
}

impl<T, U> ErrWithSpan<T, U> for Result<T, U> {
    fn err_with_span(self, span: &Span) -> Result<T, (Span, U)>
    where
        Self: Sized,
        T: Sized,
        U: Sized,
    {
        self.map_err(|e| (*span, e))
    }
}

impl TypeSize for Type {
    fn type_size(&self, structs: &HashMap<String, StructDefinition>) -> Result<usize, String> {
        match self {
            Type::Generic(..) | Type::U16 | Type::Bool | Type::Pointer(..) | Type::Function(..) => Ok(1),
            Type::Struct(s) => structs
                .get(s)
                .ok_or(format!("Struct `{s}` not in scope"))
                .map(|e| e.size),
        }
    }
}

impl TypeSize for Vec<Type> {
    fn type_size(&self, structs: &HashMap<String, StructDefinition>) -> Result<usize, String> {
        self.iter().map(|t| t.type_size(structs)).sum()
    }
}

macro_rules! ptr {
    ($t:expr) => {
        Type::Pointer(1, Box::new($t))
    };
    ($i:expr, $t:expr) => {
        Type::Pointer($i, Box::new($t))
    };
}

macro_rules! gen {
    ($l:expr) => { Type::new_generic($l) };
}

macro_rules! u16 {
    () => { Type::U16 };
}

macro_rules! bool {
    () => { Type::Bool };
}

impl Type {
    pub fn new_generic(label: &str) -> Type {
        Type::Generic(label.to_string())
    }

    pub fn baseline_pointers(t1: &Type, t2: &Type) -> Result<(Type, Type), String> {
        match (t1, t2) {
            (p1 @ Type::Pointer(_, _), p2 @ Type::Pointer(_, _)) => {
                Type::baseline_pointers(&p1.de_ref()?, &p2.de_ref()?)
            }
            _ => Ok((t1.clone(), t2.clone())),
        }
    }

    // todo: instead of resolving everytime we encounter a type, maintain a copy of the struct_defs map which are mappings from shortname -> resolved name.
    // this way, lookups are O(1)-ish, instead of O(n*m)
    fn find_struct_def<T>(
        s: &str,
        struct_defs: &HashMap<String, T>,
        using_stack: &Stack<String>,
    ) -> Option<String> {
        // if we have two structs
        // 1. foo::A
        // 2. A
        // and `using foo`
        // prioritize foo::A

        // search usings
        for using in using_stack.iter() {
            let resolved_name = format!("{using}{s}");
            if struct_defs.contains_key(&resolved_name) {
                return Some(resolved_name);
            }
        }

        // we don't have any, since by default, we have a using called ""
        None
    }

    pub fn resolve_struct_size(
        s: &str,
        struct_defs: &mut HashMap<String, StructDefinition>,
        structs: &HashMap<String, (Stack<String>, StructDef)>,
        seen: &mut HashSet<String>,
    ) -> Result<usize, String> {
        if seen.contains(s) {
            // this means that we have queried for this type before but have not found its size yet!
            return Err(format!(
                "Struct `{s}` contains a recursive definition without indirection"
            ));
        }
        seen.insert(s.to_string());

        let (usings, subdef) = structs.get(s).ok_or(format!(
            "Error looking up struct `{s}`, that type is not in scope here."
        ))?;

        let mut members: HashMap<String, (usize, Type)> = HashMap::new();
        let mut offset = 0usize;
        for member in &subdef.members {
            let member_name = member.name.name.clone();
            match &member.var_type {
                t @ LexType::Base(Identifier { name: type_name, .. } ) => {
                    match type_name.as_str() {
                        "u16" => {
                            members.insert(member_name, (offset, Type::U16));
                            offset += member.size as usize;
                        },
                        member_type => {
                            // get the name of the struct 
                            let member_resolved_type = Type::find_struct_def(member_type, structs, usings)
                                .ok_or(format!("Error looking up member type `{member_type}` for struct `{s}`, that type is not in scope here."))?;
                            let size = if let Some(member_type) = struct_defs.get(&member_resolved_type) {
                                member_type.size
                            } else {
                                Type::resolve_struct_size(&member_resolved_type, struct_defs, structs, seen)?
                            };

                            let member_type = Type::parse(t, struct_defs, usings)?;
                            members.insert(member_name, (offset, member_type));
                            offset += size * member.size as usize;
                        }
                    }
                },
                t @ LexType::Ptr(_) => {
                    let member_type = Type::parse(t, structs, usings)?;
                    members.insert(member_name, (offset, member_type));
                    offset += member.size as usize;
                },
                LexType::Func(_) => return Err(format!("Struct `{}` directly defines a function as a type. You should use a function pointer here.", &member.name.name)),
                LexType::Gen(_) => return Err(format!("Struct `{}` directly defines a generic as a type. You must use a concrete type here.", &member.name.name)),
            };
        }

        struct_defs.insert(
            s.to_string(),
            StructDefinition {
                members,
                size: offset,
            },
        );

        Ok(offset)
    }

    pub fn parse_types<T>(
        in_t: &[LexType],
        out_t: &[LexType],
        struct_defs: &HashMap<String, T>,
        using_stack: &Stack<String>,
    ) -> Result<(Vec<Type>, Vec<Type>), String> {
        let in_parsed: Vec<Type> = in_t
            .iter()
            .map(|t| Type::parse(t, struct_defs, using_stack))
            .collect::<Result<Vec<Type>, String>>()?;
        let out_parsed: Vec<Type> = out_t
            .iter()
            .map(|t| Type::parse(t, struct_defs, using_stack))
            .collect::<Result<Vec<Type>, String>>()?;
        Ok((in_parsed, out_parsed))
    }

    pub fn parse_func_def<T>(
        t: &FuncType,
        struct_defs: &HashMap<String, T>,
        using_stack: &Stack<String>,
    ) -> Result<FunctionType, String> {
        let (in_t, out_t) = Type::parse_types(&t.i, &t.o, struct_defs, using_stack)?;
        Ok(FunctionType { in_t, out_t })
    }

    pub fn parse<T>(
        t: &LexType,
        struct_defs: &HashMap<String, T>,
        using_stack: &Stack<String>,
    ) -> Result<Type, String> {
        match t {
            LexType::Base(Identifier { name, .. }) => match name.as_str() {
                "u16" => Ok(Type::U16),
                "bool" => Ok(Type::Bool),
                s => {
                    if let Some(struct_name) = Type::find_struct_def(s, struct_defs, using_stack) {
                        Ok(Type::Struct(struct_name))
                    } else {
                        Err(format!("Type `{s}` was not parseable"))
                    }
                }
            },
            LexType::Ptr(underlying) => {
                Ok(Type::parse(underlying, struct_defs, using_stack)?.add_ref())
            }
            LexType::Gen(base) => Ok(Type::Generic(base.name.clone())),
            LexType::Func(f) => Ok(Type::Function(Type::parse_func_def(
                f,
                struct_defs,
                using_stack,
            )?)), // "u16" => Ok(Type::U16),
                  // s => {
                  //     if let Some(caps) = TYPE_POINTER_REGEX.captures(s) {
                  //         let base_type = Type::parse(&caps["base_type"], struct_defs, using_stack)?;
                  //         let pointer_layers = &caps["pointers"].len();
                  //         Ok(Type::Pointer(*pointer_layers as isize, Box::new(base_type)))
                  //     } else if let Some(caps) = TYPE_GENERICS_REGEX.captures(s) {
                  //         Ok(Type::Generic(caps["alias"].to_string()))
                  //     } else if let Some(struct_name) = Type::find_struct_def(s, struct_defs, using_stack) {
                  //         Ok(Type::Struct(struct_name))
                  //     } else {
                  //         Err(format!("Type `{s}` was not parseable"))
                  //     }
                  // }
        }
    }

    pub fn add_ref(&self) -> Type {
        match self {
            Type::Pointer(i, t) => Type::Pointer(i + 1, t.clone()),
            t => Type::Pointer(1, Box::new(t.clone())),
        }
    }

    pub fn de_ref(&self) -> Result<Type, String> {
        match self {
            Type::Pointer(1, t) => Ok(*t.clone()),
            Type::Pointer(i, t) if *i > 1 => Ok(Type::Pointer(i - 1, t.clone())),
            p => Err(format!("Type {p:?} was not matched")),
        }
    }
}

#[derive(Debug)]
pub struct Marker {
    pub id: String,
    pub offset: usize,
}

#[derive(Debug)]
pub struct ConstAllocation {
    // markers are offset from the start of the sequence
    // for example, a const value of `4` in vec![0, 1, 2, 3, 4]
    // would have an offset of 4
    pub markers: Vec<Marker>,
    pub seq: Vec<u16>,
}

// This allows for overlap of tail-substrings
// for example, "abcd" and "bcd" can share memory
#[derive(Debug)]
pub struct ConstAllocationArena {
    id: usize,
    pub allocs: Vec<ConstAllocation>,
}

impl ConstAllocation {

    // check if a and b overlap by searching at the start of "a"
    // returns start and end of index into b where a would be overlayed on
    // Example:
    // overlap_start([2, 3, 4, 5, 6], [0, 1, 2, 3]) == Some((2, 6))
    fn overlap_start(a: &Vec<u16>, b: &Vec<u16>) -> Option<(usize, usize)> {
        for i in 0..b.len() {
            let common_len = min(a.len(), b.len() - i);
            if a[0..common_len] == b[i..(common_len + i)] {
                return Some((i, i + a.len()))
            }
        }

        None
    }

    pub fn try_insert(&mut self, arr: &Vec<u16>, id: &str) -> bool {
        // we can insert if the overlap of two sequences is non empty
        // for example, here are the cases where insertion is allowed

        //    bcdef
        //   abc    
        //     cde
        //       efg

        if let Some((start, end)) = Self::overlap_start(arr, &self.seq) {
            let extend_len = end as isize - self.seq.len() as isize;
            if extend_len > 0 {
                // need to extend sequence at the end
                self.seq.extend_from_slice(&arr[(arr.len()-extend_len as usize)..])
            }

            // add marker
            self.markers.push(Marker { id: id.to_owned(), offset: start });
            return true;
        }

        if let Some((start, end)) = Self::overlap_start(&self.seq, arr) {
            let mut new_arr = arr.clone();
            let extend_len = end as isize - arr.len() as isize;
            if extend_len > 0 {
                // need to extend sequence at the end
                new_arr.extend_from_slice(&self.seq[(self.seq.len()-extend_len as usize)..])
            }
            
            println!("start {start}");

            // need to shift all markers by start
            for marker in &mut self.markers {
                marker.offset = marker.offset + start;
            }
        

            self.markers.push(Marker { id: id.to_owned(), offset: 0 });

            // swap vectors
            self.seq.clear();
            self.seq.extend(new_arr);
            return true;
        }

        false
    }
}


impl ConstAllocationArena {
    pub fn new() -> Self {
        ConstAllocationArena { id: 0, allocs: vec![] }
    }

    pub fn allocate(&mut self, arr: &Vec<u16>) -> String {
        self.id += 1;
        let id = format!("{}", self.id);
        // check each allocation to see if we can add arr to it.
        for alloc in &mut self.allocs {
            println!("{:?}", alloc.markers);
            if alloc.try_insert(&arr, &id) {
                return id
            }
        }
        
        // otherwise, add a new allocation
        self.allocs.push(ConstAllocation { markers: vec![Marker{ id: id.clone(), offset: 0}], seq: arr.clone() });
        id
    }
}

#[derive(Debug)]
pub struct GlobalState {
    pub const_allocs: ConstAllocationArena,
    pub struct_defs: HashMap<String, StructDefinition>,
    pub function_signatures: HashMap<String, FunctionType>,
    pub function_dependencies: DependencyGraph,
    pub globals: HashMap<String, (Type, isize, isize)>, // name -> (label, type, num, init)
    pub inlines: HashMap<String, Statement>,            // name -> statement
}

pub struct FunctionState {
    // [a b c d] means that `d` is at the top of the ret stack
    pub current_bindings: Vec<(String, Type)>,
    pub function_out_stack: Vec<Type>,
    pub function_out_label: String,
    pub function_let_bindings: isize,
    pub function_name: String,
}

pub(crate) use gen;
pub(crate) use ptr;
pub(crate) use tasm;
pub(crate) use u16;
pub(crate) use bool;
