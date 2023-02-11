use std::{error::Error, fmt::{Display, Formatter}};

use argparse::{ArgumentParser, StoreTrue, Store};


#[derive(Debug)]
pub struct ArgParseError(i32);

impl Error for ArgParseError {}
impl Display for ArgParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered error while parsing arguments: {}", self.0)
    }
}
#[derive(Debug)]
pub struct Options {
    pub mif_file: String,
    pub jit_mode: bool,
}

pub fn get_args() -> Result<Options, ArgParseError> {
    let mut mif_file: String = "".to_string();
    let mut jit_mode: bool = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Emulate toastcpu");
        ap.refer(&mut jit_mode)
            .add_option(&["-j", "--jit"], StoreTrue, "Just in time compiled mode");
        ap.refer(&mut mif_file)
            .add_argument("MIF_FILE", Store, "The rom file");
        ap.parse_args()
    }
    .map(|_| Options {
        mif_file,
        jit_mode,
    })
    .map_err(ArgParseError)
}
