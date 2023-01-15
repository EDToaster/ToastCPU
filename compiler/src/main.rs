use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
extern crate argparse;
use argparse::{ArgumentParser, Store};
use crate::emit::module::emit_module;

lrlex_mod!("tl.l");
lrpar_mod!("tl.y");

mod emit;

#[derive(Debug)]
struct ArgParseError(i32);

impl Error for ArgParseError {}
impl Display for ArgParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered error while parsing arguments: {}", self.0)
    }
}

fn get_args() -> Result<(String, String), ArgParseError> {
    let mut input: String = "".to_string();
    let mut output: String = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Compile ToastLang to ToastASM");
        ap.refer(&mut input)
            .add_option(&["-i", "--input-file"], Store, "Input .tl file");
        ap.refer(&mut output)
            .add_option(&["-o", "--output-file"], Store, "Output file location");
        ap.parse_args()
    }
    .map(|_| (input, output))
    .map_err(|i| ArgParseError(i))
}

fn main() -> Result<(), String> {
    let (input, output) = get_args().map_err(|e| e.to_string())?;
    let program_text = fs::read_to_string(input).map_err(|e| e.to_string())?;

    let lexer_def = tl_l::lexerdef();

    let lexer = lexer_def.lexer(&*program_text);
    let (res, errs) = tl_y::parse(&lexer);
    for e in errs {
        println!("{}", e.pp(&lexer, &tl_y::token_epp));
        return Err(e.to_string())
    }

    let r = res.ok_or("Some parser thing went wrong!")?
        .map_err(|_| "Some parser thing went wrong!")?;

    let tasm = emit_module(&r);
    fs::write(output, tasm?).expect("Unable to write file");

    Ok(())
}
