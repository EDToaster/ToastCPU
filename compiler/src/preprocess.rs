use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use lazy_static::lazy_static;
use regex::{Captures, Regex};

lazy_static!(
    static ref INCLUDE_REGEX: Regex = Regex::new(r"#include<(?P<path>[a-zA-Z0-9_/-]+)>").unwrap();
);

pub fn find_first_in_include_path(path: &str, include_paths: &Vec<String>) -> Result<String, String> {
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

    Err(format!("Could not find include {path} in these places: {checked_paths:?}"))
}

pub fn preprocess(input: &str, include_paths: &Vec<String>, included_files: &mut HashSet<String>) -> Result<String, String> {
    let mut prog = input.to_string();

    for m in INCLUDE_REGEX.captures_iter(&prog.clone()).collect::<Vec<Captures>>().iter().rev() {
        let text = m.get(0).unwrap();
        let range = text.range(); // range in the original text
        let path = m["path"].trim();
        if included_files.contains(path) {
            println!("Path {path} already included, skipping");
            continue;
        }
        println!("Including path {path}");
        included_files.insert(path.to_string());
        let text = preprocess(&find_first_in_include_path(path, include_paths)?, include_paths, included_files)?;
        prog.replace_range(range, text.as_str());
    }

    Ok(prog)
}