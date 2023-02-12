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

pub fn function_label(identifier: &str) -> String {
    format!("fn_{identifier}")
}

pub fn global_label(identifier: &str) -> String {
    format!("variable_alloc_{identifier}")
}

pub fn const_label(identifier: &str) -> String {
    format!("const_{identifier}")
}