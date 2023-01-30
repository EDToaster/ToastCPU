use std::collections::HashSet;
use lrlex::lrlex_mod;
use lrpar::{LexParseError, lrpar_mod, NonStreamingLexer};

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
extern crate argparse;
extern crate core;

use argparse::{ArgumentParser, Collect, Store};
use regex::Regex;
use crate::emit::module::emit_root_module;
use crate::preprocess::preprocess;

lrlex_mod!("tl.l");
lrpar_mod!("tl.y");

mod util;
mod emit;
mod preprocess;

#[derive(Debug)]
struct ArgParseError(i32);

impl Error for ArgParseError {}
impl Display for ArgParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered error while parsing arguments: {}", self.0)
    }
}

fn get_args() -> Result<(String, String, Vec<String>), ArgParseError> {
    let mut input: String = "".to_string();
    let mut output: String = "".to_string();
    let mut include_paths: Vec<String> = vec![];
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Compile ToastLang to ToastASM");
        ap.refer(&mut input)
            .add_option(&["-i", "--input-file"], Store, "Input .tl file");
        ap.refer(&mut output)
            .add_option(&["-o", "--output-file"], Store, "Output file location");
        ap.refer(&mut include_paths)
            .add_option(&["-I", "--include"], Collect, "Include file paths");
        ap.parse_args()
    }
    .map(|_| (input, output, include_paths))
    .map_err(ArgParseError)
}

fn main() -> Result<(), String> {
    let (input, output, include_paths) = get_args().map_err(|e| e.to_string())?;

    println!("Options: {input:?}, {output:?}, {include_paths:?}");
    let mut program_text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    program_text = preprocess(&program_text, &include_paths, &mut HashSet::new())?;

    let lexer_def = tl_l::lexerdef();

    let lexer = lexer_def.lexer(&program_text);
    let (res, errs) = tl_y::parse(&lexer);
    for e in errs {
        println!("Error found: {}", e.pp(&lexer, &tl_y::token_epp));

        match e {
            LexParseError::LexError(e) => {
                println!("Token: \"{}\"", lexer.span_str(e.span()));
            }
            LexParseError::ParseError(_) => {}
        }
    }

    let r = res.ok_or("Some parser thing went wrong!")?
        .map_err(|_| "Some parser thing went wrong!")?;

    let newline_nuke = Regex::new(r"(\n|\r\n)\s*(\n|\r\n)").unwrap();
    let tasm = newline_nuke.replace_all(emit_root_module(&r)?.as_str(), "\n").to_string();
    fs::write(output, tasm).expect("Unable to write file");

    Ok(())
}
