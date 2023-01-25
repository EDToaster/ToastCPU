use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref COUNTER: Arc<Mutex<usize>> =
        Arc::new(Mutex::new(0));
}

pub fn generate_label(base_label: &str) -> String {
    let mut counter = COUNTER.lock().unwrap();
    *counter += 1;
    format!("{base_label}_{}", *counter)
}

pub fn generate_label_with_context(base_label: &str, context: &str) -> String {
    generate_label(&format!("{base_label}_{context}"))
}