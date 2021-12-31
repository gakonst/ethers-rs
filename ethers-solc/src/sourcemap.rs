use std::{fmt, iter::Peekable, str::CharIndices};

type Spanned<Token, Loc, Error> = Result<(Token, Loc), Error>;

macro_rules! syntax_err {
    ($msg:expr) => {{
        Err(SyntaxError::new($msg))
    }};
    ($msg:expr, $($tt:tt)*) => {{
        Err(SyntaxError::new(format!($msg, $($tt)*)))
    }};
}

/// An error that can happen during source map parsing.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct SyntaxError(String);

impl SyntaxError {
    pub fn new(s: impl Into<String>) -> Self {
        SyntaxError(s.into())
    }
}

#[derive(PartialEq)]
enum Token<'a> {
    Number(&'a str),
    Semicolon,
    Colon,
    /// `i` which represents an instruction that goes into a function
    In,
    /// `o` which represents an instruction that returns from a function
    Out,
    /// `-` regular jump
    Regular,
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(s) => write!(f, "NUMBER({:?})", s),
            Token::Semicolon => write!(f, "SEMICOLON"),
            Token::Colon => write!(f, "COLON"),
            Token::In => write!(f, "JMP(i)"),
            Token::Out => write!(f, "JMP(o)"),
            Token::Regular => write!(f, "JMP(-)"),
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(_) => write!(f, "number"),
            Token::Semicolon => write!(f, "`;`"),
            Token::Colon => write!(f, "`:`"),
            Token::In => write!(f, "jmp-in"),
            Token::Out => write!(f, "jmp-out"),
            Token::Regular => write!(f, "jmp"),
        }
    }
}

struct TokenStream<'input> {
    input: &'input str,
    chars: Peekable<CharIndices<'input>>,
}

impl<'input> TokenStream<'input> {
    pub fn new(input: &'input str) -> TokenStream<'input> {
        TokenStream { chars: input.char_indices().peekable(), input }
    }

    fn number(
        &mut self,
        start: usize,
        mut end: usize,
    ) -> Option<Spanned<Token<'input>, usize, SyntaxError>> {
        loop {
            if let Some((_, ch)) = self.chars.peek().cloned() {
                if !ch.is_ascii_digit() {
                    break
                }
                self.chars.next();
                end += 1;
            } else {
                end = self.input.len();
                break
            }
        }
        Some(Ok((Token::Number(&self.input[start..end]), start)))
    }
}

impl<'input> Iterator for TokenStream<'input> {
    type Item = Spanned<Token<'input>, usize, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.next()? {
            (i, ';') => Some(Ok((Token::Semicolon, i))),
            (i, ':') => Some(Ok((Token::Colon, i))),
            (i, 'i') => Some(Ok((Token::In, i))),
            (i, 'o') => Some(Ok((Token::Out, i))),
            (start, '-') => match self.chars.peek() {
                Some((_, ch)) if ch.is_ascii_digit() => {
                    self.chars.next();
                    self.number(start, start + 2)
                }
                _ => Some(Ok((Token::Regular, start))),
            },
            (start, ch) if ch.is_ascii_digit() => self.number(start, start + 1),
            (i, c) => Some(syntax_err!("Unexpected input {} at {}", c, i)),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Jump {
    /// A jump instruction that goes into a function
    In,
    /// A jump  represents an instruction that returns from a function
    Out,
    /// A regular jump instruction
    Regular,
}

/// Represents a whole source map as list of `SourceElement`s
///
/// See also https://docs.soliditylang.org/en/v0.8.10/internals/source_mappings.html
pub type SourceMap = Vec<SourceElement>;

/// Represents a single element in the source map
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceElement {
    /// The byte-offset to the start of the range in the source file
    pub offset: usize,
    /// The length of the source range in bytes
    pub length: usize,
    /// the source index
    ///
    /// Note: In the case of instructions that are not associated with any particular source file,
    /// the source mapping assigns an integer identifier of -1. This may happen for bytecode
    /// sections stemming from compiler-generated inline assembly statements.
    /// This case is represented as a `None` value
    pub index: Option<u32>,
    /// Jump instruction
    pub jump: Jump,
    /// “modifier depth”. This depth is increased whenever the placeholder statement (_) is entered
    /// in a modifier and decreased when it is left again.
    pub modifier_depth: usize,
}

#[derive(Default)]
struct SourceElementBuilder {
    pub offset: Option<usize>,
    pub length: Option<usize>,
    pub index: Option<Option<u32>>,
    pub jump: Option<Jump>,
    pub modifier_depth: Option<usize>,
}

impl SourceElementBuilder {
    fn finish(self, prev: Option<SourceElement>) -> Result<SourceElement, SyntaxError> {
        let element = if let Some(prev) = prev {
            SourceElement {
                offset: self.offset.unwrap_or(prev.offset),
                length: self.length.unwrap_or(prev.length),
                index: self.index.unwrap_or(prev.index),
                jump: self.jump.unwrap_or(prev.jump),
                modifier_depth: self.modifier_depth.unwrap_or(prev.modifier_depth),
            }
        } else {
            SourceElement {
                offset: self.offset.ok_or_else(|| SyntaxError::new("No previous offset"))?,
                length: self.length.ok_or_else(|| SyntaxError::new("No previous length"))?,
                index: self.index.ok_or_else(|| SyntaxError::new("No previous index"))?,
                jump: self.jump.ok_or_else(|| SyntaxError::new("No previous jump"))?,
                modifier_depth: self.modifier_depth.unwrap_or_default(),
            }
        };
        Ok(element)
    }

    fn set_jmp(&mut self, jmp: Jump, i: usize) -> Option<SyntaxError> {
        if self.jump.is_some() {
            return Some(SyntaxError::new(format!("Jump already set: {}", i)))
        }
        self.jump = Some(jmp);
        None
    }

    fn set_offset(&mut self, offset: usize, i: usize) -> Option<SyntaxError> {
        if self.offset.is_some() {
            return Some(SyntaxError::new(format!("Offset already set: {}", i)))
        }
        self.offset = Some(offset);
        None
    }

    fn set_length(&mut self, length: usize, i: usize) -> Option<SyntaxError> {
        if self.length.is_some() {
            return Some(SyntaxError::new(format!("Length already set: {}", i)))
        }
        self.length = Some(length);
        None
    }

    fn set_index(&mut self, index: Option<u32>, i: usize) -> Option<SyntaxError> {
        if self.index.is_some() {
            return Some(SyntaxError::new(format!("Index already set: {}", i)))
        }
        self.index = Some(index);
        None
    }

    fn set_modifier(&mut self, modifier_depth: usize, i: usize) -> Option<SyntaxError> {
        if self.modifier_depth.is_some() {
            return Some(SyntaxError::new(format!("Modifier depth already set: {}", i)))
        }
        self.modifier_depth = Some(modifier_depth);
        None
    }
}

pub struct Parser<'input> {
    stream: TokenStream<'input>,
    last_element: Option<SourceElement>,
}

impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        Self { stream: TokenStream::new(input), last_element: None }
    }
}

macro_rules! parse_number {
    ($num:expr, $t:ty, $pos:expr) => {
        match $num.parse::<$t>() {
            Ok(num) => num,
            Err(_) => {
                return Some(syntax_err!(
                    "Expected {} to be a `{}` at {}",
                    $num,
                    stringify!($t),
                    $pos
                ))
            }
        }
    };
}

macro_rules! bail_opt {
    ($opt:stmt) => {
        if let Some(err) = { $opt } {
            return Some(Err(err))
        }
    };
}

impl<'input> Iterator for Parser<'input> {
    type Item = Result<SourceElement, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = State::Offset;
        let mut builder = SourceElementBuilder::default();

        loop {
            match self.stream.next()? {
                Ok((token, pos)) => match token {
                    Token::Semicolon => break,
                    Token::Number(num) => match state {
                        State::Offset => {
                            bail_opt!(builder.set_offset(parse_number!(num, usize, pos), pos))
                        }
                        State::Length => {
                            bail_opt!(builder.set_length(parse_number!(num, usize, pos), pos))
                        }
                        State::Index => {
                            let index = match parse_number!(num, i32, pos) {
                                i if i < -1 => {
                                    return Some(syntax_err!(
                                        "Unexpected index identifier of `{}` at {}",
                                        i,
                                        pos
                                    ))
                                }
                                -1 => None,
                                i => Some(i as u32),
                            };
                            bail_opt!(builder.set_index(index, pos))
                        }
                        State::Modifier => {
                            bail_opt!(builder.set_modifier(parse_number!(num, usize, pos), pos))
                        }
                        State::Jmp => {
                            return Some(syntax_err!("Expected Jump found number at {}", pos))
                        }
                    },
                    Token::Colon => {
                        bail_opt!(state.advance(pos))
                    }
                    Token::In => {
                        bail_opt!(builder.set_jmp(Jump::In, pos))
                    }
                    Token::Out => {
                        bail_opt!(builder.set_jmp(Jump::Out, pos))
                    }
                    Token::Regular => {
                        bail_opt!(builder.set_jmp(Jump::Regular, pos))
                    }
                },
                Err(err) => return Some(Err(err)),
            }
        }

        let element = match builder.finish(self.last_element.take()) {
            Ok(element) => {
                self.last_element = Some(element.clone());
                Ok(element)
            }
            Err(err) => Err(err),
        };
        Some(element)
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
enum State {
    Offset,
    Length,
    Index,
    Jmp,
    Modifier,
}

impl State {
    fn advance(&mut self, i: usize) -> Option<SyntaxError> {
        match self {
            State::Offset => *self = State::Length,
            State::Length => *self = State::Index,
            State::Index => *self = State::Jmp,
            State::Jmp => *self = State::Modifier,
            State::Modifier => return Some(SyntaxError::new(format!("unexpected colon at {}", i))),
        }
        None
    }
}

/// Parses a source map
pub fn parse(input: &str) -> Result<SourceMap, SyntaxError> {
    Parser::new(input).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn tokenize(s: &str) -> Vec<Spanned<Token, usize, SyntaxError>> {
        TokenStream::new(s).collect()
    }

    #[test]
    fn can_parse_source_maps() {
        // all source maps from the compiler output test data
        let source_maps = include_str!("../test-data/out-source-maps.txt");

        for (line, s) in source_maps.lines().enumerate() {
            parse(s).unwrap_or_else(|_| panic!("Failed to parse line {}", line));
        }
    }
}
