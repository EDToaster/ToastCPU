//! Stack-state probes for editor tooling (typed holes & stack inspection).
//!
//! The type / stack-effect checker knows the stack at every point but normally
//! discards it. When probe recording is active (see [`record_probes`]), the
//! checker snapshots the stack at each statement boundary and at every `??`
//! hole, so the language server can show "what is on the stack here".
//!
//! Recording is controlled by a thread-local, so we don't have to thread an
//! extra parameter through the entire emitter. The checker runs synchronously
//! to completion within [`record_probes`] (no `.await` occurs mid-check), so
//! this is safe under the language server's async runtime — thread-locals are
//! per-thread and each analysis owns its recorder for the duration of the call.

use std::cell::RefCell;

use lrpar::Span;

use crate::emit::types::Type;

/// What kind of point a [`StackProbe`] was taken at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeKind {
    /// A `??` hole (the stack the programmer is asking about).
    Hole,
    /// Just after a statement (the stack is the state once it has run).
    Statement,
}

/// A snapshot of the type stack at a source location.
#[derive(Debug, Clone)]
pub struct StackProbe {
    pub span: Span,
    pub kind: ProbeKind,
    /// Stack contents, bottom-to-top.
    pub stack: Vec<Type>,
    /// The enclosing function's name, for context.
    pub function: String,
}

thread_local! {
    static RECORDER: RefCell<Option<Vec<StackProbe>>> = const { RefCell::new(None) };
}

/// Run `f` with probe recording enabled, returning `f`'s result together with
/// every probe gathered during it — including when `f` returns an error partway
/// through (useful for the common in-progress program that doesn't yet check).
pub fn record_probes<R>(f: impl FnOnce() -> R) -> (R, Vec<StackProbe>) {
    RECORDER.with(|r| *r.borrow_mut() = Some(Vec::new()));
    let result = f();
    let probes = RECORDER.with(|r| r.borrow_mut().take().unwrap_or_default());
    (result, probes)
}

/// Record a probe if recording is active; a no-op otherwise (e.g. the compiler).
pub(crate) fn push_probe(span: Span, kind: ProbeKind, stack: Vec<Type>, function: &str) {
    RECORDER.with(|r| {
        if let Some(v) = r.borrow_mut().as_mut() {
            v.push(StackProbe {
                span,
                kind,
                stack,
                function: function.to_string(),
            });
        }
    });
}
