use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::is_verbose;

lazy_static! {
    static ref INCLUDE_REGEX: Regex = Regex::new(r"#include<(?P<path>[a-zA-Z0-9_/-]+)>").unwrap();
}

pub fn find_first_in_include_path(
    path: &str,
    include_paths: &Vec<String>,
) -> Result<String, String> {
    let file_path = format!("{path}.tl");

    let mut checked_paths: Vec<String> = vec![];

    for include_path in include_paths {
        let mut check_path = PathBuf::new();
        check_path.push(include_path);
        check_path.push(&file_path);
        checked_paths.push(check_path.as_path().to_string_lossy().parse().unwrap());
        if let Ok(text) = fs::read_to_string(check_path) {
            return Ok(text);
        }
    }

    Err(format!(
        "Could not find include {path} in these places: {checked_paths:?}"
    ))
}

/// Like [`find_first_in_include_path`], but also returns the resolved file path
/// (so callers can track which file each byte came from).
fn resolve_include(path: &str, include_paths: &[String]) -> Result<(String, String), String> {
    let file_path = format!("{path}.tl");
    let mut checked_paths: Vec<String> = vec![];

    for include_path in include_paths {
        let mut check_path = PathBuf::new();
        check_path.push(include_path);
        check_path.push(&file_path);
        checked_paths.push(check_path.to_string_lossy().to_string());
        if let Ok(text) = fs::read_to_string(&check_path) {
            return Ok((check_path.to_string_lossy().to_string(), text));
        }
    }

    Err(format!(
        "Could not find include {path} in these places: {checked_paths:?}"
    ))
}

/// One contiguous run of preprocessed output that came verbatim from a single
/// source file. `pp_start..pp_end` are byte offsets into the preprocessed text;
/// the byte at preprocessed offset `o` maps to `orig_start + (o - pp_start)` in
/// `file`.
#[derive(Debug, Clone, Copy)]
struct Segment {
    pp_start: usize,
    pp_end: usize,
    file: usize,
    orig_start: usize,
}

/// A map from offsets in preprocessed text back to `(original file, offset)`.
///
/// Because `#include`s are expanded in place, an offset in the preprocessed
/// program does not correspond to the same offset in the file the user is
/// editing. This lets a consumer (the language server) translate a diagnostic's
/// preprocessed span back to a real file and position.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    files: Vec<String>,
    /// Sorted by `pp_start` (segments are appended in output order).
    segments: Vec<Segment>,
}

impl SourceMap {
    fn intern(&mut self, path: &str) -> usize {
        if let Some(i) = self.files.iter().position(|f| f == path) {
            return i;
        }
        self.files.push(path.to_string());
        self.files.len() - 1
    }

    fn push_segment(&mut self, out: &mut String, file: usize, orig_start: usize, s: &str) {
        if s.is_empty() {
            return;
        }
        let pp_start = out.len();
        out.push_str(s);
        self.segments.push(Segment {
            pp_start,
            pp_end: out.len(),
            file,
            orig_start,
        });
    }

    /// Map a preprocessed byte range `[start, end)` back to
    /// `(original file path, orig_start, orig_end)`. The result is clamped to a
    /// single segment (a range that crosses a file boundary is truncated to the
    /// file containing `start`). Returns `None` if `start` is out of range.
    pub fn map_range(&self, start: usize, end: usize) -> Option<(&str, usize, usize)> {
        let idx = self.segments.partition_point(|s| s.pp_end <= start);
        let seg = self.segments.get(idx)?;
        if start < seg.pp_start {
            return None;
        }
        let ostart = seg.orig_start + (start - seg.pp_start);
        let clamped_end = end.min(seg.pp_end);
        let oend = seg.orig_start + clamped_end.saturating_sub(seg.pp_start);
        Some((&self.files[seg.file], ostart, oend.max(ostart)))
    }
}

/// An error raised while preprocessing, with an optional span (in the *root*
/// source) of the `#include` directive that caused it.
#[derive(Debug, Clone)]
pub struct PreprocessError {
    pub message: String,
    /// `(start, end)` byte offsets of the offending include in the root source,
    /// when the failure originated from a root-level directive.
    pub root_span: Option<(usize, usize)>,
}

/// Preprocess `source` (whose path is `root_path`) while building a
/// [`SourceMap`] that maps preprocessed offsets back to original files.
///
/// This mirrors [`preprocess`] (expanding `#include<...>` in place, each unique
/// include once) but records provenance instead of discarding it.
pub fn preprocess_mapped(
    source: &str,
    root_path: &str,
    include_paths: &[String],
) -> Result<(String, SourceMap), PreprocessError> {
    let mut out = String::new();
    let mut map = SourceMap::default();
    let mut included: HashSet<String> = HashSet::new();
    let root = map.intern(root_path);
    expand_mapped(source, root, true, include_paths, &mut out, &mut map, &mut included)?;
    Ok((out, map))
}

fn expand_mapped(
    text: &str,
    file: usize,
    is_root: bool,
    include_paths: &[String],
    out: &mut String,
    map: &mut SourceMap,
    included: &mut HashSet<String>,
) -> Result<(), PreprocessError> {
    let mut last = 0usize;
    for m in INCLUDE_REGEX.captures_iter(text) {
        let whole = m.get(0).unwrap();
        let range = whole.range();

        // Text before this directive comes verbatim from `file`.
        map.push_segment(out, file, last, &text[last..range.start]);

        let path = m.name("path").unwrap().as_str().trim();
        if !included.contains(path) {
            included.insert(path.to_string());
            let (resolved_path, inc_text) =
                resolve_include(path, include_paths).map_err(|message| PreprocessError {
                    message,
                    root_span: if is_root {
                        Some((range.start, range.end))
                    } else {
                        None
                    },
                })?;
            let inc_id = map.intern(&resolved_path);
            expand_mapped(&inc_text, inc_id, false, include_paths, out, map, included).map_err(
                |mut e| {
                    // Attribute a nested failure to the root-level include that
                    // pulled it in, so the LSP can point at a real location.
                    if is_root && e.root_span.is_none() {
                        e.root_span = Some((range.start, range.end));
                    }
                    e
                },
            )?;
        }

        last = range.end;
    }
    // Trailing text after the last directive.
    map.push_segment(out, file, last, &text[last..]);
    Ok(())
}

pub fn preprocess(
    input: &str,
    include_paths: &Vec<String>,
    included_files: &mut HashSet<String>,
) -> Result<String, String> {
    let mut prog = input.to_string();

    for m in INCLUDE_REGEX
        .captures_iter(&prog.clone())
        .collect::<Vec<Captures>>()
        .iter()
        .rev()
    {
        let text = m.get(0).unwrap();
        let range = text.range(); // range in the original text
        let path = m["path"].trim();
        if included_files.contains(path) {
            // todo: update our bookmarks
            prog.replace_range(range, "");
            if is_verbose() {
                println!("Path {path} already included, skipping");
            }
            continue;
        }
        if is_verbose() {
            println!("Including path {path}");
        }
        included_files.insert(path.to_string());
        let text = preprocess(
            &find_first_in_include_path(path, include_paths)?,
            include_paths,
            included_files,
        )?;
        // todo: update our bookmarks
        prog.replace_range(range, text.as_str());
    }

    Ok(prog)
}
