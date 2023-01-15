use core::fmt;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::iter;
use lazy_static::lazy_static;
use regex::Regex;
use crate::tl_y::Identifier;

macro_rules! tasm {
    ($prog:ident; $($params:expr),*; $asm:literal) => {
        $prog.push_str(&*format!($asm, $($params),*));
    }
}
pub(crate) use tasm;

pub trait TypeSize {
    fn type_size(&self) -> isize;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StructType {

}

impl TypeSize for StructType {
    fn type_size(&self) -> isize {
        todo!()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum Type {
    U16,
    Pointer(isize, Box<Type>),
    Struct(StructType),
    Generic(String),
}

lazy_static! {
    static ref TYPE_POINTER_REGEX: Regex = Regex::new(r"(?P<base_type>\$?[^*]+)(?P<pointers>\*+)").unwrap();
    static ref TYPE_GENERICS_REGEX: Regex = Regex::new(r"(?P<alias>\$?[^*]+)").unwrap();
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::U16 => { write!(f, "u16") }
            Type::Pointer(i, t) => { write!(f, "{:?}{}", t, iter::repeat('*').take(*i as usize).collect::<String>()) }
            Type::Struct(_) => { write!(f, "struct") }
            Type::Generic(s) => { write!(f, "{s}")}
        }
    }
}

impl TypeSize for Type {
    fn type_size(&self) -> isize {
        match self {
            Type::U16 => 1,
            Type::Pointer(..) => 1,
            Type::Struct(s) => s.type_size(),
            Type::Generic(s) => 1,
        }
    }
}

impl TypeSize for Vec<Type> {
    fn type_size(&self) -> isize {
        self.iter().map(|t| t.type_size()).sum()
    }
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
            _ => Ok((t1.clone(), t2.clone()))
        }
    }

    pub fn parse(s: &str) -> Result<Type, ()> {
        match s {
            "u16" => Ok(Type::U16),
            s => {
                if let Some(caps) = TYPE_POINTER_REGEX.captures(s) {
                    let base_type = Type::parse(&caps["base_type"])?;
                    let pointer_layers = &caps["pointers"].len();
                    Ok(Type::Pointer(*pointer_layers as isize, Box::new(base_type)))
                } else if let Some(caps) = TYPE_GENERICS_REGEX.captures(s) {
                    Ok(Type::Generic((&caps["alias"]).to_string()))
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn add_ref(&self) -> Type {
        match self {
            Type::Pointer(i, t) => {
                Type::Pointer(i + 1, t.clone())
            }
            t => {
                Type::Pointer(1, Box::new(t.clone()))
            }
        }
    }

    pub fn de_ref(&self) -> Result<Type, String> {
        match self {
            Type::Pointer(1, t) => {
                Ok(*t.clone())
            }
            Type::Pointer(i, t) if *i > 1 => {
                Ok(Type::Pointer(i - 1, t.clone()))
            }
            p => { Err(format!("Type {p:?} was not matched").to_string()) }
        }
    }
}


pub struct GlobalState {
    pub string_allocs_counter: isize,
    pub string_allocs: HashMap<Vec<u16>, String>,
    pub function_signatures: HashMap<String, (Vec<Type>, Vec<Type>)>,
    pub globals: HashMap<String, (String, Type)>, // name -> label
}

pub struct FunctionState {
    // [a b c d] means that `d` is at the top of the ret stack
    pub current_bindings: Vec<(String, Type)>,
    pub stack_view: Vec<Type>,
}

pub fn parse_types(in_t: &Vec<Identifier>, out_t: &Vec<Identifier>) -> Result<(Vec<Type>, Vec<Type>), ()> {
    let in_parsed: Vec<Type> = in_t.iter().map(|t| Type::parse(&t.name)).collect::<Result<Vec<Type>, ()>>()?;
    let out_parsed: Vec<Type> = out_t.iter().map(|t| Type::parse(&t.name)).collect::<Result<Vec<Type>, ()>>()?;
    Ok((in_parsed, out_parsed))
}
