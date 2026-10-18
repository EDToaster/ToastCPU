use std::{error::Error, fmt::{Display, Formatter}};

use argparse::{ArgumentParser, Store, StoreTrue};

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
    pub window: bool,
    pub scale: usize,
    pub fps: usize,
    pub debug: bool,
    pub paused: bool,
    pub breakpoints: String,
}

pub fn get_args() -> Result<Options, ArgParseError> {
    let mut opts = Options {
        mif_file: String::new(),
        jit_mode: false,
        window: false,
        scale: 2,
        fps: 60,
        debug: false,
        paused: false,
        breakpoints: String::new(),
    };
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Emulate toastcpu");
        ap.refer(&mut opts.jit_mode)
            .add_option(&["-j", "--jit"], StoreTrue, "Just in time compiled mode");
        ap.refer(&mut opts.window).add_option(
            &["-w", "--window"],
            StoreTrue,
            "Render the VGA screen in a native window (pixel-accurate FPGA font) instead of the terminal",
        );
        ap.refer(&mut opts.scale)
            .add_option(&["--scale"], Store, "Window mode: initial integer upscale of the 600x480 screen (default 2)");
        ap.refer(&mut opts.fps)
            .add_option(&["--fps"], Store, "Window mode: frame rate cap, 0 for uncapped (default 60)");
        ap.refer(&mut opts.debug).add_option(
            &["-d", "--debug"],
            StoreTrue,
            "Window mode: open with the debugger sidebar expanded (F1 toggles it)",
        );
        ap.refer(&mut opts.paused)
            .add_option(&["--paused"], StoreTrue, "Window mode: start with the CPU paused at 0x0000");
        ap.refer(&mut opts.breakpoints).add_option(
            &["-b", "--break"],
            Store,
            "Window mode: comma-separated breakpoints, as hex addresses or labels (e.g. fn_main,0x1A)",
        );
        ap.refer(&mut opts.mif_file)
            .add_argument("MIF_FILE", Store, "The rom file")
            .required();
        ap.parse_args()
    }
    .map(|_| opts)
    .map_err(ArgParseError)
}
