use lrlex::lrlex_mod;
use lrpar::{lrpar_mod, LexParseError, NonStreamingLexer};
use std::collections::HashSet;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
extern crate argparse;
extern crate core;

use crate::emit::module::emit_root_module;
use crate::preprocess::preprocess;
use argparse::{ArgumentParser, Collect, Store, StoreTrue};
use regex::Regex;

use once_cell::sync::OnceCell;

lrlex_mod!("tl.l");
lrpar_mod!("tl.y");

mod emit;
mod parser_util;
mod preprocess;
mod util;

static VERBOSE: OnceCell<bool> = OnceCell::new();

pub fn is_verbose() -> bool {
    *VERBOSE.get().unwrap_or(&false)
}

#[derive(Debug)]
struct ArgParseError(i32);

impl Error for ArgParseError {}
impl Display for ArgParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered error while parsing arguments: {}", self.0)
    }
}

#[derive(Debug)]
struct Options {
    input: String,
    output: String,
    include_paths: Vec<String>,
    verbose: bool,
}

fn get_args() -> Result<Options, ArgParseError> {
    let mut input: String = "".to_string();
    let mut output: String = "".to_string();
    let mut include_paths: Vec<String> = vec![];
    let mut verbose: bool = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Compile ToastLang to ToastASM");
        ap.refer(&mut input)
            .add_option(&["-i", "--input-file"], Store, "Input .tl file");
        ap.refer(&mut output)
            .add_option(&["-o", "--output-file"], Store, "Output file location");
        ap.refer(&mut include_paths).add_option(
            &["-I", "--include"],
            Collect,
            "Include file paths",
        );
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Verbose mode");
        ap.parse_args()
    }
    .map(|_| Options {
        input,
        output,
        include_paths,
        verbose,
    })
    .map_err(ArgParseError)
}

fn run() -> Result<(), String> {
    let options = get_args().map_err(|e| e.to_string())?;
    VERBOSE.set(options.verbose).expect("Cannot set VERBOSE flag twice. This is a bug inside the program, please contact the owner to resolve it.");

    if is_verbose() {
        println!("{options:?}");
    }

    let mut program_text = fs::read_to_string(options.input).map_err(|e| e.to_string())?;
    program_text = preprocess(&program_text, &options.include_paths, &mut HashSet::new())?;

    // if is_verbose() {
    //     println!("{program_text}");
    // }

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

    let r = res
        .ok_or("Some parser thing went wrong!")?
        .map_err(|_| "Some parser thing went wrong!")?;

    let tasm = emit_root_module(&r).map_err(|(span, s)| format!("Error here: `\n{}\n`\n\n{s}", &program_text[span.start()..span.end()]))?;

    let newline_nuke = Regex::new(r"(\n|\r\n)\s*(\n|\r\n)").unwrap();
    let tasm = newline_nuke
        .replace_all(&tasm, "\n")
        .to_string();
    fs::write(options.output, tasm).expect("Unable to write file");

    Ok(())
}

fn main() -> Result<(), ()> {
    let res = run();
    match res {
        Err(e) => { println!("{e}"); Err(()) }
        _ => Ok(()),
    }
}
