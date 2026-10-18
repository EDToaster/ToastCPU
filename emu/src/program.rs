//! Loading `.mif` ROM images, plus the debug info the assembler leaves in them.
//!
//! The assembler writes one line per word (the gap before `{` is tabs):
//!
//! ```text
//! 0000 : 2C47; -- . imov [imov! isr .fn_isr]    {[Label(name='reset')]}
//! ```
//!
//! Everything after `--` is the source the word came from, and the trailing
//! `{...}` lists the labels that point at this address. The debugger uses both.

use std::collections::{BTreeMap, HashMap};

use regex::Regex;

use crate::hardware::def::ROM_SIZE;

/// Fill value for ROM words the image doesn't define (`halt`).
const EMPTY_WORD: u16 = 0x7000;

#[derive(Default, Debug)]
pub struct DebugInfo {
    /// Source text for each address that had one.
    pub source: HashMap<u16, String>,
    /// Labels pointing at each address.
    pub labels: BTreeMap<u16, Vec<String>>,
    /// Reverse lookup, label name (without leading `.`) -> address.
    pub by_name: HashMap<String, u16>,
}

impl DebugInfo {
    /// Resolve user input to an address: hex (`0x1A`, `1a`, `$1a`) or a label
    /// name (with or without the leading `.`).
    pub fn resolve(&self, input: &str) -> Option<u16> {
        let s = input.trim();
        let name = s.trim_start_matches('.');
        if let Some(&addr) = self.by_name.get(name) {
            return Some(addr);
        }
        let hex = s
            .trim_start_matches("0x")
            .trim_start_matches("0X")
            .trim_start_matches('$');
        u16::from_str_radix(hex, 16).ok()
    }

    /// The nearest label at or before `addr`, with the offset from it,
    /// e.g. `fn_main+3`.
    pub fn symbolize(&self, addr: u16) -> Option<String> {
        self.symbolize_within(addr, u16::MAX)
    }

    /// Like [`Self::symbolize`], but only if the label is at most `max_offset`
    /// words back (so plain data values don't get bogus names).
    pub fn symbolize_within(&self, addr: u16, max_offset: u16) -> Option<String> {
        let (&base, names) = self.labels.range(..=addr).next_back()?;
        if addr - base > max_offset {
            return None;
        }
        let name = names.first()?;
        Some(if base == addr {
            name.clone()
        } else {
            format!("{name}+{}", addr - base)
        })
    }
}

pub struct Program {
    pub rom: Vec<u16>,
    pub debug: DebugInfo,
}

impl Program {
    pub fn parse(text: &str) -> Program {
        // assume HEX radix with a depth of 32k and width of 16
        let line_matcher = Regex::new(r"^([0-9a-fA-F]{4}) : ([0-9a-fA-F]{4});(?:\s*--\s?(.*))?$").unwrap();
        let label_matcher = Regex::new(r"Label\(name='([^']+)'\)").unwrap();

        let mut rom = vec![EMPTY_WORD; ROM_SIZE];
        let mut debug = DebugInfo::default();

        for line in text.lines() {
            let Some(caps) = line_matcher.captures(line.trim_end()) else {
                continue;
            };
            let addr = u16::from_str_radix(&caps[1], 16).unwrap();
            let val = u16::from_str_radix(&caps[2], 16).unwrap();
            if let Some(slot) = rom.get_mut(addr as usize) {
                *slot = val;
            }

            let Some(comment) = caps.get(3).map(|m| m.as_str()) else {
                continue;
            };
            let (src, meta) = match comment.find('{') {
                Some(i) => (&comment[..i], &comment[i..]),
                None => (comment, ""),
            };
            let src = src.trim();
            if !src.is_empty() {
                debug.source.insert(addr, src.to_string());
            }
            for label in label_matcher.captures_iter(meta) {
                let name = label[1].to_string();
                debug.by_name.entry(name.clone()).or_insert(addr);
                debug.labels.entry(addr).or_default().push(name);
            }
        }

        Program { rom, debug }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "DEPTH = 32768;\nBEGIN\n\
0000 : 2C47; -- . imov [imov! isr .fn_isr]\t\t\t{[Label(name='reset')]}\n\
0001 : 3C02; -- ' imoh\t\t\t{}\n\
0010 : 7000; --   halt\t\t\t{[Label(name='reset_ret'), Label(name='other')]}\n\
0011 : 5D60;\n";

    #[test]
    fn parses_words_and_debug_info() {
        let p = Program::parse(SAMPLE);
        assert_eq!(p.rom[0], 0x2C47);
        assert_eq!(p.rom[0x11], 0x5D60);
        assert_eq!(p.rom[0x12], EMPTY_WORD);
        assert_eq!(p.debug.source[&0], ". imov [imov! isr .fn_isr]");
        assert_eq!(p.debug.labels[&0x10], ["reset_ret", "other"]);
        assert_eq!(p.debug.resolve(".other"), Some(0x10));
        assert_eq!(p.debug.resolve("0x1f"), Some(0x1F));
        assert_eq!(p.debug.symbolize(0x13).as_deref(), Some("reset_ret+3"));
        assert_eq!(p.debug.symbolize(0x0).as_deref(), Some("reset"));
        assert_eq!(p.debug.symbolize_within(0x40, 8), None);
    }
}
