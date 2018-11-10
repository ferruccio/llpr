use std::io::{Read, Seek, SeekFrom};
use errors::*;

type Result<T> = ::std::result::Result<T, PdfError>;

pub type PdfString = Vec<u8>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PdfName(pub String);

#[derive(Debug, PartialEq)]
pub enum PdfToken {
    Ident(String),
    Integer(i64),
    Real(f64),
    Name(PdfName),
    Str(PdfString),
    BeginArray,
    EndArray,
    BeginDictionary,
    EndDictionary
}

pub struct TokenReader<S> where S: Read + Seek {
    source: S
}

impl<S> TokenReader<S> where S: Read + Seek {
    pub fn new(source: S) -> TokenReader<S> {
        TokenReader{ source: source }
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        Ok(self.source.seek(pos)?)
    }

    fn backup(&mut self) {
        let _ = self.seek(SeekFrom::Current(-1));
    }

    fn getch(&mut self) -> Result<char> {
        let mut buffer = [0];
        if self.source.read(&mut buffer)? == 0 {
            Err(PdfError::EndOfFile)
        } else {
            Ok(buffer[0] as char)
        }
    }

    fn skip_whitespace(&mut self) -> Result<()> {
        let whitespace = [' ', '\t', '\n', '\r', '\x0c'];
        let mut in_comment = false;
        loop {
            let ch = self.getch()?;
            if in_comment {
                if ch == '\n' {
                    in_comment = false;
                }
            } else {
                if ch == '%' {
                    in_comment = true;
                } else {
                    if !whitespace.contains(&ch) {
                        self.backup();
                        return Ok(());
                    }
                }
            }
        }
    }

    pub fn next(&mut self) -> Result<PdfToken> {
        let syntax_error = Err(PdfError::InvalidPdf("syntax error"));
        self.skip_whitespace()?;
        match self.getch()? {
            ch @ 'A'...'Z' | ch @ 'a'...'z' => self.identifier(ch),
            ch @ '+' | ch @ '-' | ch @ '.' | ch @ '0'...'9' => self.numeric(ch),
            '/' => self.name(),
            '[' => Ok(PdfToken::BeginArray),
            ']' => Ok(PdfToken::EndArray),
            '(' => self.string(),
            '<' => match self.getch()? {
                '<' => Ok(PdfToken::BeginDictionary),
                _ => {
                    self.backup();
                    self.hex_string()
                }
            },
            '>' => match self.getch()? {
                '>' => Ok(PdfToken::EndDictionary),
                _ => syntax_error
            },
            _ => syntax_error
        }
    }

    fn identifier(&mut self, first: char) -> Result<PdfToken> {
        let mut ident = first.to_string();
        loop {
            match self.getch()? {
                ch @ 'A'...'Z' | ch @ 'a'...'z' => ident.push(ch),
                _ => {
                    self.backup();
                    return Ok(PdfToken::Ident(ident));
                }
            }
        }
    }

    fn numeric(&mut self, first: char) -> Result<PdfToken> {
        let mut number = first.to_string();
        let mut decimal = first == '.';
        loop {
            match self.getch()? {
                ch @ '0'...'9' => number.push(ch),
                '.' => {
                    number.push('.');
                    decimal = true;
                }
                _ => {
                    self.backup();
                    if decimal {
                        return Ok(PdfToken::Real(number.parse()?));
                    } else {
                        return Ok(PdfToken::Integer(number.parse()?));
                    }
                }
            }
        }
    }

    fn name(&mut self) -> Result<PdfToken> {
        let mut name = "".to_owned();
        loop {
            match self.getch()? {
                ch @ 'A'...'Z' | ch @ 'a'...'z' => name.push(ch),
                _ => {
                    self.backup();
                    return Ok(PdfToken::Name(PdfName(name)));
                }
            }
        }
    }

    fn string(&mut self) -> Result<PdfToken> {
        let mut nesting = 0;
        let mut string = vec![];
        loop {
            match self.getch()? {
                '(' => {
                    string.push('(' as u8);
                    nesting += 1;
                },
                ')' => {
                    if nesting == 0 {
                        return Ok(PdfToken::Str(string));
                    }
                    string.push(')' as u8);
                    nesting -= 1;
                },
                '\\' => match self.getch()? {
                    'n' => string.push(0x0a),
                    'r' => string.push(0x0d),
                    't' => string.push(0x09),
                    'b' => string.push(0x08),
                    'f' => string.push(0x0c),
                    '(' => string.push('(' as u8),
                    ')' => string.push(')' as u8),
                    ch @ '0'...'7' => {
                        let mut octal = ch as u8 - '0' as u8;
                        let mut digits = 1;
                        loop {
                            match self.getch()? {
                                ch @ '0'...'7' => {
                                    octal = (octal << 3) | (ch as u8 - '0' as u8);
                                    digits += 1;
                                    if digits == 3 {
                                        string.push(octal);
                                        break;
                                    }
                                }
                                _ => {
                                    string.push(octal);
                                    self.backup();
                                    break;
                                }
                            }
                        }
                    },
                    _ => {}
                }
                ch @ _ => string.push(ch as u8)
            }
        }
    }

    fn hex_string(&mut self) -> Result<PdfToken> {
        let mut value = 0u8;
        let mut hex = 0u8;
        let mut first = false;
        let mut string = vec![];
        loop {
            match self.getch()? {
                ' ' | '\t' | '\n' | '\r' | '\x0c' => {}
                ch @ '0'...'9' => {
                    hex = ch as u8 - '0' as u8;
                    first = !first;
                }
                ch @ 'A'...'F' => {
                    hex = 10 + (ch as u8 - 'A' as u8);
                    first = !first;
                }
                ch @ 'a'...'f' => {
                    hex = 10 + (ch as u8 - 'a' as u8);
                    first = !first;
                }
                '>' => {
                    if first {
                        string.push(hex << 4);
                    }
                    return Ok(PdfToken::Str(string));
                }
                _ => return Err(PdfError::InvalidPdf("unexpected character in hex string"))
            }
            if first {
                value = hex;
            } else {
                string.push((value << 4) | hex);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Read, Seek, Cursor};

    fn tokens(source: &'static str) -> impl Read + Seek {
        Cursor::new(source.as_bytes())
    }

    #[test]
    fn identifiers() {
        let mut tr = TokenReader::new(tokens("ident Ident\nIDENT "));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Ident("ident".to_owned()));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Ident("Ident".to_owned()));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Ident("IDENT".to_owned()));
    }

    #[test]
    fn integers() {
        let mut tr = TokenReader::new(tokens("0 15 -24\t\t+212 %blah blah blah\n12345 "));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Integer(0));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Integer(15));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Integer(-24));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Integer(212));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Integer(12345));
    }

    #[test]
    fn reals() {
        let mut tr = TokenReader::new(tokens("0.0 2030.0 3.1415926 -32. .5 "));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Real(0.0));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Real(2030.0));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Real(3.1415926));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Real(-32.0));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Real(0.5));
    }

    #[test]
    fn names() {
        let mut tr = TokenReader::new(tokens("/Name /size /TEST /AnotherName "));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Name(PdfName("Name".to_owned())));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Name(PdfName("size".to_owned())));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Name(PdfName("TEST".to_owned())));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Name(PdfName("AnotherName".to_owned())));
    }

    #[test]
    fn strings() {
        let mut tr = TokenReader::new(tokens(r###"
(This is a string)
(Strings may contain newlines
and such.)
(Strings may contain balanced parentheses ( ) and
special characters ( * ! & } ^ % and so on ).)
(The following is an empty string.)
()
(It has zero (0) length.)
(String with escapes: \n \r \t \b \f \( \) \\ \0 \10 \100 \1234)
"###));

        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![84, 104, 105, 115, 32, 105, 115, 32,
            97, 32, 115, 116, 114, 105, 110, 103]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![83, 116, 114, 105, 110, 103, 115, 32,
            109, 97, 121, 32, 99, 111, 110, 116, 97, 105, 110, 32, 110, 101, 119,
            108, 105, 110, 101, 115, 10, 97, 110, 100, 32, 115, 117, 99, 104, 46]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![83, 116, 114, 105, 110, 103, 115, 32,
            109, 97, 121, 32, 99, 111, 110, 116, 97, 105, 110, 32, 98, 97, 108,
            97, 110, 99, 101, 100, 32, 112, 97, 114, 101, 110, 116, 104, 101, 115,
            101, 115, 32, 40, 32, 41, 32, 97, 110, 100, 10, 115, 112, 101, 99, 105,
            97, 108, 32, 99, 104, 97, 114, 97, 99, 116, 101, 114, 115, 32, 40, 32,
            42, 32, 33, 32, 38, 32, 125, 32, 94, 32, 37, 32, 97, 110, 100, 32, 115,
            111, 32, 111, 110, 32, 41, 46]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![84, 104, 101, 32, 102, 111, 108, 108,
            111, 119, 105, 110, 103, 32, 105, 115, 32, 97, 110, 32, 101, 109, 112,
            116, 121, 32, 115, 116, 114, 105, 110, 103, 46]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![73, 116, 32, 104, 97, 115, 32, 122, 101,
            114, 111, 32, 40, 48, 41, 32, 108, 101, 110, 103, 116, 104, 46]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![83, 116, 114, 105, 110, 103, 32, 119,
            105, 116, 104, 32, 101, 115, 99, 97, 112, 101, 115, 58, 32, 10, 32,
            13, 32, 9, 32, 8, 32, 12, 32, 40, 32, 41, 32, 32, 0, 32, 8, 32, 64,
            32, 83, 52]));
    }

    #[test]
    fn hex_strings() {
        let mut tr = TokenReader::new(tokens("<><a> <12AbCd> <deadbeef> <CAFEBABE> "));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![0xa0]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![0x12, 0xab, 0xcd]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![0xde, 0xad, 0xbe, 0xef]));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::Str(vec![0xca, 0xfe, 0xba, 0xbe]));
    }

    #[test]
    fn structures() {
        let mut tr = TokenReader::new(tokens("<< >> [ ] "));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::BeginDictionary);
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::EndDictionary);
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::BeginArray);
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::EndArray);
    }

    #[test]
    fn seek() {
        let mut tr = TokenReader::new(tokens("<< >> [ ] "));
        let _ = tr.seek(SeekFrom::Start(6));
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::BeginArray);
        let tok = tr.next().unwrap();
        assert_eq!(tok, PdfToken::EndArray);
    }
}