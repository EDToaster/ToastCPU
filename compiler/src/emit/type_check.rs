use std::borrow::Borrow;
use std::collections::HashMap;
use lrpar::Span;
use crate::emit::types::Type;
use crate::util::gss::Stack;

fn resolve_generic_type(out_t: &Type, span: &Span, generics: &HashMap<String, Type>) -> Result<Type, (Span, String)> {
    match out_t {
        Type::Pointer(i, t) => {
            Ok(Type::Pointer(
                *i,
                Box::new(
                    resolve_generic_type(t.borrow(), span, generics)?
                )))
        }
        Type::Generic(label) => {
            Ok(generics
                .get(label)
                .ok_or((span.clone(), format!("Output generic {label:?} has no corresponding input type.")))?
                .clone())
        }
        _ => { Ok(out_t.clone()) }
    }
}

fn type_matches(stack_t: &Type, span: &Span, sig_t: &Type, generics: &mut HashMap<String, Type>) -> Result<bool, (Span, String)> {
    // unwrap pointers
    let (stack_t, sig_t) = Type::baseline_pointers(stack_t, sig_t).map_err(|s| (span.clone(), s))?;
    match sig_t {
        Type::Generic(label) => {
            // check if label is in the generics map
            match generics.get(label.as_str()) {
                None => {
                    // add the generic to map
                    generics.insert(label, stack_t);
                    Ok(true)
                }
                Some(g) => {
                    Ok(*g == stack_t)
                }
            }
        }
        _ => { Ok(stack_t == sig_t) }
    }
}

pub fn check_and_apply_multiple_stack_transitions(s: &str, span: &Span, stack_view: &mut Stack<Type>, rules: &Vec<(Vec<Type>, Vec<Type>)>) -> Result<(), (Span, String)> {
    for (in_t, out_t) in rules {
        match check_and_apply_stack_transition(s, span, stack_view, in_t, out_t) {
            Ok(_) => { return Ok(()) }
            _ => {}
        }
    }

    Err((span.clone(), format!("Cannot invoke statement `{s}` as it requires one of {rules:?} to match. Current stack is {:?}.", stack_view)))
}

pub fn check_and_apply_stack_transition(s: &str, span: &Span, stack_view: &mut Stack<Type>, in_t: &Vec<Type>, out_t: &Vec<Type>) -> Result<Vec<Type>, (Span, String)> {
    let length = stack_view.len;
    if length < in_t.len() {
        // we don't have enough stack params to call this function
        return Err((span.clone(), format!("Cannot invoke statement `{s}` as it requires types {in_t:?} at the top of the stack. Current stack is {:?}.", stack_view)));
    }

    let mut generics: HashMap<String, Type> = HashMap::new();
    let new_length = length-in_t.len();
    let stack_view_slice = stack_view.peek_n(in_t.len());

    for (i, t) in stack_view_slice.iter().enumerate() {
        if !type_matches(t, span, &in_t[i], &mut generics)? {
            return Err((span.clone(), format!("Cannot invoke statement `{s}` as it requires types {in_t:?} at the top of the stack. Current stack is {:?}.", stack_view)))
        }
    }

    // passed input type check, remove elements from stack and resolve output
    let mut dropped = vec![];
    for i in 0..in_t.len() {
        dropped.push(stack_view.pop().unwrap());
    }

    dropped.reverse();

    for t in out_t.iter() {
        stack_view.push(resolve_generic_type(t, span, &generics)?);
    }

    Ok(dropped)
}