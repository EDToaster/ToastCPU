use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref COUNTER: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

pub fn generate_label(base_label: &str) -> String {
    let mut counter = COUNTER.lock().unwrap();
    *counter += 1;
    format!("{base_label}_{}", *counter)
}

pub fn generate_label_with_context(base_label: &str, context: &str) -> String {
    generate_label(&format!("{base_label}_{context}"))
}

fn replace_mod(identifier: &str) -> String {
    identifier.replace(':', "_submod_")
}

pub fn function_label(identifier: &str) -> String {
    replace_mod(identifier)
}

pub fn global_label(identifier: &str) -> String {
    format!("variable_alloc_{}", replace_mod(identifier))
}
