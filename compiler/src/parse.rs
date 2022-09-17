use std::{str::FromStr, num::ParseIntError};

pub trait ParseWithPrefix {
    type Err;
    fn parse_with_prefix(s: &str) -> Result<Self, Self::Err> where Self: Sized;
}

impl ParseWithPrefix for i32 {
    type Err = ParseIntError;

    fn parse_with_prefix(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            i32::from_str_radix(&s[2..], 16)
        } else if s.starts_with("0b") {
            i32::from_str_radix(&s[2..], 2)
        } else {
            i32::from_str(s)
        }
    }
}