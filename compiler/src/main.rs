use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;

extern crate argparse;
extern crate core;

use argparse::{ArgumentParser, Collect, Store, StoreTrue};
use regex::Regex;

use tlc::{is_verbose, DiagnosticKind};

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
    only_preprocess: bool,
}

fn get_args() -> Result<Options, ArgParseError> {
    let mut input: String = "".to_string();
    let mut output: String = "".to_string();
    let mut include_paths: Vec<String> = vec![];
    let mut verbose: bool = false;
    let mut only_preprocess: bool = false;
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
        ap.refer(&mut only_preprocess)
            .add_option(&["-p", "--only-preprocess"], StoreTrue, "Emit the preprocessed code, and do not compile to tasm");
        ap.parse_args()
    }
    .map(|_| Options {
        input,
        output,
        include_paths,
        verbose,
        only_preprocess,
    })
    .map_err(ArgParseError)
}

fn run() -> Result<(), String> {
    let options = get_args().map_err(|e| e.to_string())?;
    tlc::set_verbose(options.verbose);

    if is_verbose() {
        println!("{options:?}");
    }

    let program_text = fs::read_to_string(&options.input).map_err(|e| e.to_string())?;
    // Run the `#include` preprocessor via the frontend library.
    let program_text = tlc::preprocess_source(&program_text, &options.include_paths)
        .map_err(|d| d.message)?;

    if options.only_preprocess {
        fs::write(options.output, program_text).expect("Unable to write file");
        return Ok(());
    }

    // Lex + parse via the frontend library, then reproduce the historical
    // print-and-exit CLI behavior on top of the returned diagnostics.
    let (ast, diagnostics) = tlc::parse(&program_text);
    for d in &diagnostics {
        println!("Error found: {}", d.message);
        if d.kind == DiagnosticKind::Lex {
            println!("Token: \"{}\"", &program_text[d.start..d.end]);
        }
    }

    let r = ast.ok_or("Some parser thing went wrong!")?;

    let tasm = tlc::emit::module::emit_root_module(&r).map_err(|(span, s)| {
        format!(
            "Error here: `\n{}\n`\n\n{s}",
            &program_text[span.start()..span.end()]
        )
    })?;

    let newline_nuke = Regex::new(r"(\n|\r\n)\s*(\n|\r\n)").unwrap();
    let tasm = newline_nuke.replace_all(&tasm, "\n").to_string();
    fs::write(options.output, tasm).expect("Unable to write file");

    Ok(())
}

fn main() -> Result<(), ()> {
    let res = run();
    match res {
        Err(e) => {
            println!("{e}");
            Err(())
        }
        _ => Ok(()),
    }
}
