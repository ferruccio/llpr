use errors::*;
use pdf_source::Source;
use pdf_types::*;

type Result<T> = ::std::result::Result<T, PdfError>;

fn pdf_name(name: &str) -> Option<PdfName> {
    NAMES.get(name).cloned()
}

fn pdf_keyword(keyword: &str) -> Option<PdfKeyword> {
    KEYWORDS.get(keyword).cloned()
}

pub fn next_token(source: &mut Box<Source>) -> Result<PdfToken> {
    let syntax_error = Err(PdfError::InvalidPdf("syntax error"));
    skip_whitespace(source)?;
    match source.getch()? {
        ch @ 'A'...'Z' | ch @ 'a'...'z' => keyword(source, ch),
        ch @ '+' | ch @ '-' | ch @ '.' | ch @ '0'...'9' => number(source, ch),
        '/' => name_or_symbol(source),
        '[' => Ok(PdfToken::BeginArray),
        ']' => Ok(PdfToken::EndArray),
        '(' => string(source),
        '<' => match source.getch()? {
            '<' => Ok(PdfToken::BeginDictionary),
            _ => {
                source.backup();
                hex_string(source)
            }
        },
        '>' => match source.getch()? {
            '>' => Ok(PdfToken::EndDictionary),
            _ => syntax_error,
        },
        _ => syntax_error,
    }
}

fn skip_whitespace(source: &mut Box<Source>) -> Result<()> {
    let whitespace = [' ', '\t', '\n', '\r', '\x0c'];
    let mut in_comment = false;
    loop {
        let ch = source.getch()?;
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
        } else {
            if ch == '%' {
                in_comment = true;
            } else {
                if !whitespace.contains(&ch) {
                    source.backup();
                    return Ok(());
                }
            }
        }
    }
}

fn keyword(source: &mut Box<Source>, first: char) -> Result<PdfToken> {
    let mut keyword = first.to_string();
    loop {
        match source.getch()? {
            ch @ 'A'...'Z' | ch @ 'a'...'z' => keyword.push(ch),
            _ => {
                source.backup();
                return match pdf_keyword(&keyword) {
                    Some(keyword) => Ok(PdfToken::Keyword(keyword)),
                    None => Ok(PdfToken::Keyword(PdfKeyword::Unknown)),
                };
            }
        }
    }
}

fn number(source: &mut Box<Source>, first: char) -> Result<PdfToken> {
    let mut number = first.to_string();
    let mut decimal = first == '.';
    loop {
        match source.getch()? {
            ch @ '0'...'9' => number.push(ch),
            '.' => {
                number.push('.');
                decimal = true;
            }
            _ => {
                source.backup();
                if decimal {
                    return Ok(PdfToken::Real(number.parse()?));
                } else {
                    return Ok(PdfToken::Integer(number.parse()?));
                }
            }
        }
    }
}

fn nybble(ch: char) -> Result<u8> {
    match ch {
        ch @ '0'...'9' => Ok(ch as u8 - b'0'),
        ch @ 'A'...'F' => Ok(10 + (ch as u8 - b'A')),
        ch @ 'a'...'f' => Ok(10 + (ch as u8 - b'a')),
        _ => Err(PdfError::InvalidPdf("invalid hex character")),
    }
}

fn name_or_symbol(source: &mut Box<Source>) -> Result<PdfToken> {
    let mut name = "".to_owned();
    loop {
        match source.getch()? {
            ' ' | '\t' | '\n' | '\r' | '\x0c' => {
                source.backup();
                return match pdf_name(&name) {
                    Some(name) => Ok(PdfToken::Name(name)),
                    None => Ok(PdfToken::Symbol(name.as_bytes().to_vec())),
                };
            }
            '#' => {
                let hi = nybble(source.getch()?)?;
                let lo = nybble(source.getch()?)?;
                name.push((hi << 4 | lo) as char);
            }
            ch @ _ => name.push(ch),
        }
    }
}

fn string(source: &mut Box<Source>) -> Result<PdfToken> {
    let mut nesting = 0;
    let mut string = vec![];
    loop {
        match source.getch()? {
            '(' => {
                string.push(b'(');
                nesting += 1;
            }
            ')' => {
                if nesting == 0 {
                    return Ok(PdfToken::Str(string));
                }
                string.push(b')');
                nesting -= 1;
            }
            '\\' => match source.getch()? {
                'n' => string.push(b'\n'),
                'r' => string.push(b'\r'),
                't' => string.push(b'\t'),
                'b' => string.push(0x08),
                'f' => string.push(0x0c),
                '(' => string.push(b'('),
                ')' => string.push(b')'),
                ch @ '0'...'7' => string.push(octal_escape(source, ch)?),
                _ => {}
            },
            ch @ _ => string.push(ch as u8),
        }
    }
}

fn octal_escape(source: &mut Box<Source>, first: char) -> Result<u8> {
    let mut octal = first as u8 - b'0';
    let mut digits = 1;
    loop {
        match source.getch()? {
            ch @ '0'...'7' => {
                octal = (octal << 3) | (ch as u8 - b'0');
                digits += 1;
                if digits == 3 {
                    return Ok(octal);
                }
            }
            _ => {
                source.backup();
                return Ok(octal);
            }
        }
    }
}

fn hex_string(source: &mut Box<Source>) -> Result<PdfToken> {
    let mut value = 0u8;
    let mut hex = 0u8;
    let mut first = false;
    let mut string = vec![];
    loop {
        match source.getch()? {
            ' ' | '\t' | '\n' | '\r' | '\x0c' => {}
            '>' => {
                if first {
                    string.push(hex << 4);
                }
                return Ok(PdfToken::Str(string));
            }
            ch @ _ => {
                hex = nybble(ch)?;
                first = !first;
            }
        }
        if first {
            value = hex;
        } else {
            string.push((value << 4) | hex);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_source::ByteSource;

    fn next(source: &mut Box<Source>) -> PdfToken {
        next_token(source).unwrap()
    }

    #[test]
    fn keywords() {
        let mut source: Box<Source> = Box::new(ByteSource::new(b"trailer false\nwho_knows "));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Keyword(PdfKeyword::trailer));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Keyword(PdfKeyword::r#false));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Keyword(PdfKeyword::Unknown));
    }

    #[test]
    fn integers() {
        let mut source: Box<Source> =
            Box::new(ByteSource::new(b"0 15 -24\t\t+212 %blah blah blah\n12345 "));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Integer(0));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Integer(15));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Integer(-24));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Integer(212));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Integer(12345));
    }

    #[test]
    fn reals() {
        let mut source: Box<Source> = Box::new(ByteSource::new(b"0.0 2030.0 3.1415926 -32. .5 "));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Real(0.0));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Real(2030.0));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Real(3.1415926));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Real(-32.0));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Real(0.5));
    }

    #[test]
    fn names() {
        let mut source: Box<Source> = Box::new(ByteSource::new(b"/Root /Size "));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Name(PdfName::Root));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Name(PdfName::Size));
    }

    #[test]
    fn symbols() {
        let mut source: Box<Source> = Box::new(ByteSource::new(
            b"/Who /What /#57here /W#68#65#6e /And#20How ",
        ));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Symbol("Who".as_bytes().to_vec()));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Symbol("What".as_bytes().to_vec()));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Symbol("Where".as_bytes().to_vec()));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Symbol("When".as_bytes().to_vec()));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Symbol("And How".as_bytes().to_vec()));
    }

    #[test]
    fn strings() {
        let mut source: Box<Source> = Box::new(ByteSource::new(
            br###"
(This is a string)
(Strings may contain newlines
and such.)
(Strings may contain balanced parentheses ( ) and
special characters ( * ! & } ^ % and so on ).)
(The following is an empty string.)
()
(It has zero (0) length.)
(String with escapes: \n \r \t \b \f \( \) \\ \0 \10 \100 \1234)
"###,
        ));

        let tok = next(&mut source);
        assert_eq!(
            tok,
            PdfToken::Str(vec![
                84, 104, 105, 115, 32, 105, 115, 32, 97, 32, 115, 116, 114, 105, 110, 103
            ])
        );
        let tok = next(&mut source);
        assert_eq!(
            tok,
            PdfToken::Str(vec![
                83, 116, 114, 105, 110, 103, 115, 32, 109, 97, 121, 32, 99, 111, 110, 116, 97, 105,
                110, 32, 110, 101, 119, 108, 105, 110, 101, 115, 10, 97, 110, 100, 32, 115, 117,
                99, 104, 46
            ])
        );
        let tok = next(&mut source);
        assert_eq!(
            tok,
            PdfToken::Str(vec![
                83, 116, 114, 105, 110, 103, 115, 32, 109, 97, 121, 32, 99, 111, 110, 116, 97, 105,
                110, 32, 98, 97, 108, 97, 110, 99, 101, 100, 32, 112, 97, 114, 101, 110, 116, 104,
                101, 115, 101, 115, 32, 40, 32, 41, 32, 97, 110, 100, 10, 115, 112, 101, 99, 105,
                97, 108, 32, 99, 104, 97, 114, 97, 99, 116, 101, 114, 115, 32, 40, 32, 42, 32, 33,
                32, 38, 32, 125, 32, 94, 32, 37, 32, 97, 110, 100, 32, 115, 111, 32, 111, 110, 32,
                41, 46
            ])
        );
        let tok = next(&mut source);
        assert_eq!(
            tok,
            PdfToken::Str(vec![
                84, 104, 101, 32, 102, 111, 108, 108, 111, 119, 105, 110, 103, 32, 105, 115, 32,
                97, 110, 32, 101, 109, 112, 116, 121, 32, 115, 116, 114, 105, 110, 103, 46
            ])
        );
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Str(vec![]));
        let tok = next(&mut source);
        assert_eq!(
            tok,
            PdfToken::Str(vec![
                73, 116, 32, 104, 97, 115, 32, 122, 101, 114, 111, 32, 40, 48, 41, 32, 108, 101,
                110, 103, 116, 104, 46
            ])
        );
        let tok = next(&mut source);
        assert_eq!(
            tok,
            PdfToken::Str(vec![
                83, 116, 114, 105, 110, 103, 32, 119, 105, 116, 104, 32, 101, 115, 99, 97, 112,
                101, 115, 58, 32, 10, 32, 13, 32, 9, 32, 8, 32, 12, 32, 40, 32, 41, 32, 32, 0, 32,
                8, 32, 64, 32, 83, 52
            ])
        );
    }

    #[test]
    fn hex_strings() {
        let mut source: Box<Source> =
            Box::new(ByteSource::new(b"<><a> <12AbCd> <deadbeef> <CAFEBABE> "));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Str(vec![]));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Str(vec![0xa0]));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Str(vec![0x12, 0xab, 0xcd]));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Str(vec![0xde, 0xad, 0xbe, 0xef]));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::Str(vec![0xca, 0xfe, 0xba, 0xbe]));
    }

    #[test]
    fn structures() {
        let mut source: Box<Source> = Box::new(ByteSource::new(b"<< >> [ ] "));
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::BeginDictionary);
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::EndDictionary);
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::BeginArray);
        let tok = next(&mut source);
        assert_eq!(tok, PdfToken::EndArray);
    }

}
