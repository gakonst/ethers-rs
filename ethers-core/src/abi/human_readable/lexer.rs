
use ethabi::{Event, EventParam, ParamType};
use std::{fmt, iter::Peekable, str::CharIndices};
use unicode_xid::UnicodeXID;

pub type Spanned<Token, Loc, Error> = Result<(Loc, Token, Loc), Error>;

macro_rules! unrecognised {
    ($l:ident,$r:ident,$t:expr) => {
        return Err(LexerError::UnrecognisedToken($l, $r, $t))
    };
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Token<'input> {
    Identifier(&'input str),

    // Punctuation
    OpenParenthesis,
    CloseParenthesis,
    Comma,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Point,

    Struct,
    Event,
    Error,
    Enum,
    Function,
    Tuple,

    Memory,
    Storage,
    Calldata,

    Public,
    Private,
    Internal,
    External,

    Constant,

    Type,
    Pure,
    View,
    Payable,
    Returns,
    Anonymous,
    Receive,
    Fallback,
    Abstract,
    Virtual,
    Override,

    Constructor,
    Indexed,

    Uint(u16),
    Int(u16),
    Bytes(u8),
    // prior to 0.8.0 `byte` used to be an alias for `bytes1`
    Byte,
    DynamicBytes,
    Bool,
    Address,
    String,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(id) => write!(f, "{}", id),
            Token::Uint(w) => write!(f, "uint{}", w),
            Token::Int(w) => write!(f, "int{}", w),
            Token::Bytes(w) => write!(f, "bytes{}", w),
            Token::Byte => write!(f, "byte"),
            Token::DynamicBytes => write!(f, "bytes"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::OpenParenthesis => write!(f, "("),
            Token::CloseParenthesis => write!(f, ")"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::Point => write!(f, "."),
            Token::Tuple => write!(f, "tuple"),
            Token::Bool => write!(f, "bool"),
            Token::Address => write!(f, "address"),
            Token::String => write!(f, "string"),
            Token::Function => write!(f, "function"),
            Token::Struct => write!(f, "struct"),
            Token::Event => write!(f, "event"),
            Token::Error => write!(f, "error"),
            Token::Enum => write!(f, "enum"),
            Token::Type => write!(f, "type"),
            Token::Memory => write!(f, "memory"),
            Token::Storage => write!(f, "storage"),
            Token::Calldata => write!(f, "calldata"),
            Token::Public => write!(f, "public"),
            Token::Private => write!(f, "private"),
            Token::Internal => write!(f, "internal"),
            Token::External => write!(f, "external"),
            Token::Constant => write!(f, "constant"),
            Token::Pure => write!(f, "pure"),
            Token::View => write!(f, "view"),
            Token::Payable => write!(f, "payable"),
            Token::Returns => write!(f, "returns"),
            Token::Anonymous => write!(f, "anonymous"),
            Token::Constructor => write!(f, "constructor"),
            Token::Indexed => write!(f, "indexed"),
            Token::Receive => write!(f, "receive"),
            Token::Fallback => write!(f, "fallback"),
            Token::Abstract => write!(f, "abstract"),
            Token::Virtual => write!(f, "virtual"),
            Token::Override => write!(f, "override"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum LexerError {
    #[error("UnrecognisedToken {0}:{1} `{2}`")]
    UnrecognisedToken(usize, usize, String),
    #[error("end of file")]
    EndOfFile,
}

pub(crate) struct HumanReadableLexer<'input> {
    input: &'input str,
    chars: Peekable<CharIndices<'input>>,
}

// === impl HumanReadableLexer ===

impl<'input> HumanReadableLexer<'input> {
    /// Creates a new instance of the lexer
    pub fn new(input: &'input str) -> Self {
        Self { chars: input.char_indices().peekable(), input }
    }

    fn next_token(&mut self) -> Option<Spanned<Token<'input>, usize, LexerError>> {
        loop {
            match self.chars.next() {
                Some((start, ch)) if UnicodeXID::is_xid_start(ch) || ch == '_' => {
                    let end;
                    loop {
                        if let Some((i, ch)) = self.chars.peek() {
                            if !UnicodeXID::is_xid_continue(*ch) && *ch != '$' {
                                end = *i;
                                break
                            }
                            self.chars.next();
                        } else {
                            end = self.input.len();
                            break
                        }
                    }
                    let id = &self.input[start..end];

                    return if let Some(w) = keyword(id) {
                        Some(Ok((start, w, end)))
                    } else {
                        Some(Ok((start, Token::Identifier(id), end)))
                    }
                }
                Some((i, '(')) => return Some(Ok((i, Token::OpenParenthesis, i + 1))),
                Some((i, ')')) => return Some(Ok((i, Token::CloseParenthesis, i + 1))),
                Some((i, ';')) => return Some(Ok((i, Token::Semicolon, i + 1))),
                Some((i, ',')) => return Some(Ok((i, Token::Comma, i + 1))),
                Some((i, '.')) => return Some(Ok((i, Token::Point, i + 1))),
                Some((i, '[')) => return Some(Ok((i, Token::OpenBracket, i + 1))),
                Some((i, ']')) => return Some(Ok((i, Token::CloseBracket, i + 1))),
                Some((_, ch)) if ch.is_whitespace() => (),
                Some((start, _)) => {
                    let mut end;
                    loop {
                        if let Some((i, ch)) = self.chars.next() {
                            end = i;
                            if ch.is_whitespace() {
                                break
                            }
                        } else {
                            end = self.input.len();
                            break
                        }
                    }

                    return Some(Err(LexerError::UnrecognisedToken(
                        start,
                        end,
                        self.input[start..end].to_owned(),
                    )))
                }
                None => return None,
            }
        }
    }
}

impl<'input> Iterator for HumanReadableLexer<'input> {
    type Item = Spanned<Token<'input>, usize, LexerError>;

    /// Return the next token
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

pub struct HumanReadableParser<'input> {
    lexer: Peekable<HumanReadableLexer<'input>>,
}

// === impl HumanReadableParser ===

impl<'input> HumanReadableParser<'input> {
    /// Creates a new instance of the lexer
    pub fn new(input: &'input str) -> Self {
        let lexer = HumanReadableLexer::new(input);
        Self { lexer: lexer.peekable() }
    }

    pub fn parse_event(&mut self) -> Result<Event, LexerError> {
        let (l, token, r) = self.next_spanned()?;
        let name = match token {
            Token::Event => {
                let (_, next, _) = self.lexer.peek().cloned().ok_or(LexerError::EndOfFile)??;
                if let Token::Identifier(name) = next {
                    self.next();
                    name
                } else {
                    ""
                }
            }
            Token::Identifier(name) => name,
            t => unrecognised!(l, r, t.to_string()),
        };

        self.take_open_parenthesis()?;
        let inputs = self.take_event_params()?;
        self.take_close_parenthesis()?;
        let event = Event { name: name.to_string(), inputs, anonymous: self.take_anonymous() };

        Ok(event)
    }

    fn take_anonymous(&mut self) -> bool {
        if self.peek_next(Token::Anonymous) {
            self.next();
            true
        } else {
            false
        }
    }

    /// Parses all event params
    fn take_event_params(&mut self) -> Result<Vec<EventParam>, LexerError> {
        let params = Vec::new();
        loop {
            if self.peek_next(Token::CloseBracket) {
                break
            }
            let (l, token, r) = self.next_spanned()?;
            match token {
                t => unrecognised!(l, r, t.to_string()),
            }
        }

        Ok(params)
    }

    /// Parses a list of parameter types
    fn take_params(&mut self) -> Result<Vec<ParamType>, LexerError> {
        todo!()
    }

    fn take_open_parenthesis(&mut self) -> Result<(), LexerError> {
        self.take_next_exact(Token::OpenParenthesis)
    }

    fn take_close_parenthesis(&mut self) -> Result<(), LexerError> {
        self.take_next_exact(Token::CloseParenthesis)
    }

    fn take_next_exact(&mut self, token: Token) -> Result<(), LexerError> {
        let (l, next, r) = self.next_spanned()?;
        if next != token {
            unrecognised!(l, r, next.to_string())
        }
        Ok(())
    }

    /// Returns true if the next token is the given `token`
    fn peek_next(&mut self, token: Token) -> bool {
        if let Some(Ok(next)) = self.lexer.peek() {
            next.1 == token
        } else {
            false
        }
    }

    fn next_param(&mut self) {}

    fn next_spanned(&mut self) -> Spanned<Token<'input>, usize, LexerError> {
        self.next().ok_or(LexerError::EndOfFile)?
    }

    fn next(&mut self) -> Option<Spanned<Token<'input>, usize, LexerError>> {
        self.lexer.next()
    }
}

fn keyword(id: &str) -> Option<Token> {
    let token = match id {
        "address" => Token::Address,
        "anonymous" => Token::Anonymous,
        "bool" => Token::Bool,
        "bytes1" => Token::Bytes(1),
        "bytes2" => Token::Bytes(2),
        "bytes3" => Token::Bytes(3),
        "bytes4" => Token::Bytes(4),
        "bytes5" => Token::Bytes(5),
        "bytes6" => Token::Bytes(6),
        "bytes7" => Token::Bytes(7),
        "bytes8" => Token::Bytes(8),
        "bytes9" => Token::Bytes(9),
        "bytes10" => Token::Bytes(10),
        "bytes11" => Token::Bytes(11),
        "bytes12" => Token::Bytes(12),
        "bytes13" => Token::Bytes(13),
        "bytes14" => Token::Bytes(14),
        "bytes15" => Token::Bytes(15),
        "bytes16" => Token::Bytes(16),
        "bytes17" => Token::Bytes(17),
        "bytes18" => Token::Bytes(18),
        "bytes19" => Token::Bytes(19),
        "bytes20" => Token::Bytes(20),
        "bytes21" => Token::Bytes(21),
        "bytes22" => Token::Bytes(22),
        "bytes23" => Token::Bytes(23),
        "bytes24" => Token::Bytes(24),
        "bytes25" => Token::Bytes(25),
        "bytes26" => Token::Bytes(26),
        "bytes27" => Token::Bytes(27),
        "bytes28" => Token::Bytes(28),
        "bytes29" => Token::Bytes(29),
        "bytes30" => Token::Bytes(30),
        "bytes31" => Token::Bytes(31),
        "bytes32" => Token::Bytes(32),
        "bytes" => Token::DynamicBytes,
        "byte" => Token::Byte,
        "calldata" => Token::Calldata,
        "constant" => Token::Constant,
        "constructor" => Token::Constructor,
        "enum" => Token::Enum,
        "event" => Token::Event,
        "error" => Token::Error,
        "external" => Token::External,
        "function" => Token::Function,
        "indexed" => Token::Indexed,
        "tuple" => Token::Tuple,
        "int8" => Token::Int(8),
        "int16" => Token::Int(16),
        "int24" => Token::Int(24),
        "int32" => Token::Int(32),
        "int40" => Token::Int(40),
        "int48" => Token::Int(48),
        "int56" => Token::Int(56),
        "int64" => Token::Int(64),
        "int72" => Token::Int(72),
        "int80" => Token::Int(80),
        "int88" => Token::Int(88),
        "int96" => Token::Int(96),
        "int104" => Token::Int(104),
        "int112" => Token::Int(112),
        "int120" => Token::Int(120),
        "int128" => Token::Int(128),
        "int136" => Token::Int(136),
        "int144" => Token::Int(144),
        "int152" => Token::Int(152),
        "int160" => Token::Int(160),
        "int168" => Token::Int(168),
        "int176" => Token::Int(176),
        "int184" => Token::Int(184),
        "int192" => Token::Int(192),
        "int200" => Token::Int(200),
        "int208" => Token::Int(208),
        "int216" => Token::Int(216),
        "int224" => Token::Int(224),
        "int232" => Token::Int(232),
        "int240" => Token::Int(240),
        "int248" => Token::Int(248),
        "int256" => Token::Int(256),
        "internal" => Token::Internal,
        "int" => Token::Int(256),
        "memory" => Token::Memory,
        "payable" => Token::Payable,
        "private" => Token::Private,
        "public" => Token::Public,
        "pure" => Token::Pure,
        "returns" => Token::Returns,
        "storage" => Token::Storage,
        "string" => Token::String,
        "struct" => Token::Struct,
        "type" => Token::Type,
        "uint8" => Token::Uint(8),
        "uint16" => Token::Uint(16),
        "uint24" => Token::Uint(24),
        "uint32" => Token::Uint(32),
        "uint40" => Token::Uint(40),
        "uint48" => Token::Uint(48),
        "uint56" => Token::Uint(56),
        "uint64" => Token::Uint(64),
        "uint72" => Token::Uint(72),
        "uint80" => Token::Uint(80),
        "uint88" => Token::Uint(88),
        "uint96" => Token::Uint(96),
        "uint104" => Token::Uint(104),
        "uint112" => Token::Uint(112),
        "uint120" => Token::Uint(120),
        "uint128" => Token::Uint(128),
        "uint136" => Token::Uint(136),
        "uint144" => Token::Uint(144),
        "uint152" => Token::Uint(152),
        "uint160" => Token::Uint(160),
        "uint168" => Token::Uint(168),
        "uint176" => Token::Uint(176),
        "uint184" => Token::Uint(184),
        "uint192" => Token::Uint(192),
        "uint200" => Token::Uint(200),
        "uint208" => Token::Uint(208),
        "uint216" => Token::Uint(216),
        "uint224" => Token::Uint(224),
        "uint232" => Token::Uint(232),
        "uint240" => Token::Uint(240),
        "uint248" => Token::Uint(248),
        "uint256" => Token::Uint(256),
        "uint" => Token::Uint(256),
        "view" => Token::View,
        "receive" => Token::Receive,
        "fallback" => Token::Fallback,
        "abstract" => Token::Abstract,
        "virtual" => Token::Virtual,
        "override" => Token::Override,
        _ => return None,
    };
    Some(token)
}
