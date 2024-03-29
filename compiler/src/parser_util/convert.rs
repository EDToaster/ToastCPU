fn multi(s: &str) -> (isize, &str) {
    if let Some(s) = s.strip_prefix('-') {
        (-1, s)
    } else {
        (1, s)
    }
}

// Any functions here are in scope for all the grammar actions above.
pub fn dec_int(s: &str) -> Result<isize, ()> {
    let (sign, s) = multi(s); 
    Ok(sign * s.replace('_', "").parse::<isize>().map_err(|_| ())?)
}

pub fn bin_int(s: &str) -> Result<isize, ()> {
    let (sign, s) = multi(s); 
    Ok(sign * isize::from_str_radix(&s[2..].replace('_', ""), 2).map_err(|_| ())?)
    
}

pub fn hex_int(s: &str) -> Result<isize, ()> {
    let (sign, s) = multi(s); 
    Ok(sign * isize::from_str_radix(&s[2..].replace('_', ""), 16).map_err(|_| ())?)
}

pub fn char_int(s: &str) -> Result<isize, ()> {
    let inner = s.trim_matches('\'');

    // \' single quote
    // \" double quote
    // \\ backslash
    // \n new line
    // \r carriage return
    // \t tab
    // \b backspace
    // \f form feed
    // \v vertical tab (Internet Explorer 9 and older treats '\v as 'v instead of a vertical tab ('\x0B). If cross-browser compatibility is a concern, use \x0B instead of \v.)
    // \0 null character (U+0000 NULL) (only if the next character is not a decimal digit; else it is an octal escape sequence)
    // \xFF character represented by the hexadecimal byte "FF"

    match inner {
        "\\'" => Ok('\''),
        "\\\"" => Ok('"'),
        "\\\\" => Ok('\\'),
        "\\n" => Ok('\n'),
        "\\r" => Ok('\r'),
        "\\t" => Ok('\t'),
        "\\0" => Ok('\0'),
        c => c.chars().next().ok_or(()),
    }
    .map(|e| e as isize)
}

pub fn char_int_single(c: char, escaped: bool) -> Result<u16, ()> {
    match (c, escaped) {
        ('n', true) => Ok('\n' as u16),
        ('r', true) => Ok('\r' as u16),
        ('t', true) => Ok('\t' as u16),
        ('0', true) => Ok('\0' as u16),
        (_, _) => Ok(c as u16),
    }
}

pub fn string_int_arr(s: &str) -> Result<Vec<u16>, ()> {
    let mut string = vec![];
    let mut escaped = false;

    for c in s[1..s.len() - 1].chars() {
        if escaped {
            string.push(char_int_single(c, true)?);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else {
            string.push(char_int_single(c, false)?);
        }
    }

    string.push(0);

    Ok(string)
}
