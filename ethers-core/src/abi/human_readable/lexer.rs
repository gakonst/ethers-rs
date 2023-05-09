use ethabi::{
    AbiError, Constructor, Event, EventParam, Function, Param, ParamType, StateMutability,
};
use std::{fmt, iter::Peekable, str::CharIndices};
use unicode_xid::UnicodeXID;

pub type Spanned<Token, Loc, Error> = Result<(Loc, Token, Loc), Error>;

macro_rules! unrecognised {
    ($l:ident,$r:ident,$t:expr) => {
        return Err(LexerError::UnrecognisedToken($l, $r, $t))
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Token<'input> {
    Identifier(&'input str),
    Number(&'input str),
    HexNumber(&'input str),
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

    Uint(usize),
    Int(usize),
    Bytes(usize),
    // prior to 0.8.0 `byte` used to be an alias for `bytes1`
    Byte,
    DynamicBytes,
    Bool,
    Address,
    String,
}

// === impl Token ===

impl<'input> Token<'input> {
    fn into_param_type(self) -> Option<ParamType> {
        let param = match self {
            Token::Uint(size) => ParamType::Uint(size),
            Token::Int(size) => ParamType::Int(size),
            Token::Bytes(size) => ParamType::FixedBytes(size),
            Token::Byte => ParamType::FixedBytes(1),
            Token::DynamicBytes => ParamType::Bytes,
            Token::Bool => ParamType::Bool,
            Token::Address => ParamType::Address,
            Token::String => ParamType::String,
            Token::Tuple => ParamType::Tuple(vec![]),
            _ => return None,
        };

        Some(param)
    }
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(id) => write!(f, "{id}"),
            Token::Number(num) => write!(f, "{num}"),
            Token::HexNumber(num) => write!(f, "0x{num}"),
            Token::Uint(w) => write!(f, "uint{w}"),
            Token::Int(w) => write!(f, "int{w}"),
            Token::Bytes(w) => write!(f, "bytes{w}"),
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

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum LexerError {
    #[error("UnrecognisedToken {0}:{1} `{2}`")]
    UnrecognisedToken(usize, usize, String),
    #[error("Expected token `{2}` at {0}:{1} ")]
    ExpectedToken(usize, usize, String),
    #[error("EndofFileInHex {0}:{1}")]
    EndofFileInHex(usize, usize),
    #[error("MissingNumber {0}:{1}")]
    MissingNumber(usize, usize),
    #[error("end of file but expected `{0}`")]
    EndOfFileExpectedToken(String),
    #[error("end of file")]
    EndOfFile,
}

#[derive(Clone, Debug)]
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
                Some((start, ch)) if ch.is_ascii_digit() => {
                    let mut end = start + 1;
                    if ch == '0' {
                        if let Some((_, 'x')) = self.chars.peek() {
                            // hex number
                            self.chars.next();

                            let mut end = match self.chars.next() {
                                Some((end, ch)) if ch.is_ascii_hexdigit() => end,
                                Some((_, _)) => {
                                    return Some(Err(LexerError::MissingNumber(start, start + 1)))
                                }
                                None => {
                                    return Some(Err(LexerError::EndofFileInHex(
                                        start,
                                        self.input.len(),
                                    )))
                                }
                            };

                            while let Some((i, ch)) = self.chars.peek() {
                                if !ch.is_ascii_hexdigit() && *ch != '_' {
                                    break
                                }
                                end = *i;
                                self.chars.next();
                            }

                            return Some(Ok((
                                start,
                                Token::HexNumber(&self.input[start..=end]),
                                end + 1,
                            )))
                        }
                    }

                    loop {
                        if let Some((i, ch)) = self.chars.peek().cloned() {
                            if !ch.is_ascii_digit() {
                                break
                            }
                            self.chars.next();
                            end = i + 1;
                        } else {
                            end = self.input.len();
                            break
                        }
                    }
                    return Some(Ok((start, Token::Number(&self.input[start..end]), end + 1)))
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

#[derive(Clone, Debug)]
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

    /// Parses the input into a [ParamType]
    pub fn parse_type(input: &'input str) -> Result<ParamType, LexerError> {
        Self::new(input).take_param()
    }

    /// Parses a [Function] from a human readable form
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_core::abi::HumanReadableParser;
    /// let mut fun = HumanReadableParser::parse_function("function get(address author, string oldValue, string newValue)").unwrap();
    /// ```
    pub fn parse_function(input: &'input str) -> Result<Function, LexerError> {
        Self::new(input).take_function()
    }

    /// Parses a [Function] from a human readable form
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_core::abi::HumanReadableParser;
    /// let err = HumanReadableParser::parse_error("error MyError(address author, string oldValue, string newValue)").unwrap();
    /// ```
    pub fn parse_error(input: &'input str) -> Result<AbiError, LexerError> {
        Self::new(input).take_error()
    }

    /// Parses a [Constructor] from a human readable form
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_core::abi::HumanReadableParser;
    /// let mut constructor = HumanReadableParser::parse_constructor("constructor(address author, string oldValue, string newValue)").unwrap();
    /// ```
    pub fn parse_constructor(input: &'input str) -> Result<Constructor, LexerError> {
        Self::new(input).take_constructor()
    }

    /// Parses an [Event] from a human readable form
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_core::abi::HumanReadableParser;
    /// let mut event = HumanReadableParser::parse_event("event ValueChanged(address indexed author, string oldValue, string newValue)").unwrap();
    /// ```
    pub fn parse_event(input: &'input str) -> Result<Event, LexerError> {
        Self::new(input).take_event()
    }

    /// Returns the next `Error` and consumes the underlying tokens
    pub fn take_error(&mut self) -> Result<AbiError, LexerError> {
        let name = self.take_identifier(Token::Error)?;
        self.take_open_parenthesis()?;
        let inputs = self.take_function_params()?;
        self.take_close_parenthesis()?;
        Ok(AbiError { name: name.to_string(), inputs })
    }

    /// Returns the next `Constructor` and consumes the underlying tokens
    pub fn take_constructor(&mut self) -> Result<Constructor, LexerError> {
        self.take_next_exact(Token::Constructor)?;
        self.take_open_parenthesis()?;
        let inputs = self.take_function_params()?;
        self.take_close_parenthesis()?;
        Ok(Constructor { inputs })
    }

    /// Returns the next `Function` and consumes the underlying tokens
    pub fn take_function(&mut self) -> Result<Function, LexerError> {
        let name = self.take_identifier(Token::Function)?;

        self.take_open_parenthesis()?;
        let inputs = self.take_function_params()?;
        self.take_close_parenthesis()?;

        let mut state_mutability = Default::default();
        let mut outputs = vec![];
        if self.peek().is_some() {
            let _visibility = self.take_visibility();
            if let Some(mutability) = self.take_state_mutability() {
                state_mutability = mutability;
            }
            if self.peek_next(Token::Virtual) {
                self.next();
            }
            if self.peek_next(Token::Override) {
                self.next();
            }
            if self.peek_next(Token::Returns) {
                self.next();
            }

            if self.peek_next(Token::OpenParenthesis) {
                self.take_open_parenthesis()?;
                outputs = self.take_function_params()?;
                self.take_close_parenthesis()?;
            }
        }

        Ok(
            #[allow(deprecated)]
            Function { name: name.to_string(), inputs, outputs, constant: None, state_mutability },
        )
    }

    pub fn take_event(&mut self) -> Result<Event, LexerError> {
        let name = self.take_identifier(Token::Event)?;
        self.take_open_parenthesis()?;
        let inputs = self.take_event_params()?;
        self.take_close_parenthesis()?;
        let event = Event { name: name.to_string(), inputs, anonymous: self.take_anonymous() };

        Ok(event)
    }

    /// Returns an identifier, optionally prefixed with a token like `function? <name>`
    fn take_identifier(&mut self, prefixed: Token) -> Result<&'input str, LexerError> {
        let (l, token, r) = self.next_spanned()?;
        let name = match token {
            i if i == prefixed => {
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
        Ok(name)
    }

    fn take_name_opt(&mut self) -> Result<Option<&'input str>, LexerError> {
        if let (_, Token::Identifier(name), _) = self.peek_some()? {
            self.next();
            Ok(Some(name))
        } else {
            Ok(None)
        }
    }

    fn take_visibility(&mut self) -> Option<Visibility> {
        match self.lexer.peek() {
            Some(Ok((_, Token::Internal, _))) => {
                self.next();
                Some(Visibility::Internal)
            }
            Some(Ok((_, Token::External, _))) => {
                self.next();
                Some(Visibility::External)
            }
            Some(Ok((_, Token::Private, _))) => {
                self.next();
                Some(Visibility::Private)
            }
            Some(Ok((_, Token::Public, _))) => {
                self.next();
                Some(Visibility::Public)
            }
            _ => None,
        }
    }

    fn take_state_mutability(&mut self) -> Option<StateMutability> {
        match self.lexer.peek() {
            Some(Ok((_, Token::View, _))) => {
                self.next();
                Some(StateMutability::View)
            }
            Some(Ok((_, Token::Pure, _))) => {
                self.next();
                Some(StateMutability::Pure)
            }
            Some(Ok((_, Token::Payable, _))) => {
                self.next();
                Some(StateMutability::Payable)
            }
            _ => None,
        }
    }

    fn take_data_location(&mut self) -> Option<DataLocation> {
        match self.lexer.peek() {
            Some(Ok((_, Token::Memory, _))) => {
                self.next();
                Some(DataLocation::Memory)
            }
            Some(Ok((_, Token::Storage, _))) => {
                self.next();
                Some(DataLocation::Storage)
            }
            Some(Ok((_, Token::Calldata, _))) => {
                self.next();
                Some(DataLocation::Calldata)
            }
            _ => None,
        }
    }

    fn take_anonymous(&mut self) -> bool {
        if self.peek_next(Token::Anonymous) {
            self.next();
            true
        } else {
            false
        }
    }

    /// Takes comma separated values via `f` until the `token` is parsed
    fn take_csv_until<T, F>(&mut self, token: Token, f: F) -> Result<Vec<T>, LexerError>
    where
        F: Fn(&mut Self) -> Result<T, LexerError>,
    {
        let mut params = Vec::new();

        if self.peek_next(token) {
            return Ok(params)
        }

        loop {
            params.push(f(self)?);

            let (l, next, r) = match self.peek() {
                Some(next) => next?,
                _ => break,
            };

            match next {
                i if i == token => break,
                Token::Comma => {
                    self.next_spanned()?;
                }
                t => unrecognised!(l, r, t.to_string()),
            }
        }
        Ok(params)
    }

    /// Parses all function input params
    fn take_function_params(&mut self) -> Result<Vec<Param>, LexerError> {
        self.take_csv_until(Token::CloseParenthesis, |s| s.take_input_param())
    }

    fn take_input_param(&mut self) -> Result<Param, LexerError> {
        let kind = self.take_param()?;
        let _location = self.take_data_location();
        let name = self.take_name_opt()?.unwrap_or("");
        Ok(Param { name: name.to_string(), kind, internal_type: None })
    }

    /// Parses all event params
    fn take_event_params(&mut self) -> Result<Vec<EventParam>, LexerError> {
        self.take_csv_until(Token::CloseParenthesis, |s| s.take_event_param())
    }

    fn take_event_param(&mut self) -> Result<EventParam, LexerError> {
        let kind = self.take_param()?;
        let mut name = "";
        let mut indexed = false;

        loop {
            let (_, token, _) = self.peek_some()?;
            match token {
                Token::Indexed => {
                    indexed = true;
                    self.next();
                }
                Token::Identifier(id) => {
                    name = id;
                    self.next();
                    break
                }
                _ => break,
            };
        }
        Ok(EventParam { name: name.to_string(), kind, indexed })
    }

    /// Parses a list of parameter types
    fn take_params(&mut self) -> Result<Vec<ParamType>, LexerError> {
        let mut params = Vec::new();

        if self.peek_next(Token::CloseParenthesis) {
            return Ok(params)
        }
        loop {
            params.push(self.take_param()?);

            let (l, next, r) = match self.peek() {
                Some(next) => next?,
                _ => break,
            };
            match next {
                Token::Comma => {
                    self.next_spanned()?;
                }
                Token::CloseParenthesis => break,
                t => unrecognised!(l, r, t.to_string()),
            }
        }

        Ok(params)
    }

    fn take_param(&mut self) -> Result<ParamType, LexerError> {
        let (l, token, r) = self.next_spanned()?;
        let kind = match token {
            Token::OpenParenthesis => {
                let ty = self.take_params()?;
                self.take_next_exact(Token::CloseParenthesis)?;
                ParamType::Tuple(ty)
            }
            t => t
                .into_param_type()
                .ok_or_else(|| LexerError::UnrecognisedToken(l, r, t.to_string()))?,
        };
        self.take_array_tail(kind)
    }

    fn take_array_tail(&mut self, kind: ParamType) -> Result<ParamType, LexerError> {
        let (_, token, _) = match self.peek() {
            Some(next) => next?,
            _ => return Ok(kind),
        };

        match token {
            Token::OpenBracket => {
                self.next_spanned()?;
                let (_, token, _) = self.peek_some()?;
                let kind = if let Token::Number(size) = token {
                    self.next_spanned()?;
                    ParamType::FixedArray(Box::new(kind), size.parse().unwrap())
                } else {
                    ParamType::Array(Box::new(kind))
                };
                self.take_next_exact(Token::CloseBracket)?;
                self.take_array_tail(kind)
            }
            _ => Ok(kind),
        }
    }

    fn take_open_parenthesis(&mut self) -> Result<(), LexerError> {
        self.take_next_exact(Token::OpenParenthesis)
    }

    fn take_close_parenthesis(&mut self) -> Result<(), LexerError> {
        self.take_next_exact(Token::CloseParenthesis)
    }

    fn take_next_exact(&mut self, token: Token) -> Result<(), LexerError> {
        let (l, next, r) = self.next_spanned().map_err(|err| match err {
            LexerError::UnrecognisedToken(l, r, _) => {
                LexerError::ExpectedToken(l, r, token.to_string())
            }
            LexerError::EndOfFile => LexerError::EndOfFileExpectedToken(token.to_string()),
            err => err,
        })?;
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

    fn next_spanned(&mut self) -> Spanned<Token<'input>, usize, LexerError> {
        self.next().ok_or(LexerError::EndOfFile)?
    }

    fn next(&mut self) -> Option<Spanned<Token<'input>, usize, LexerError>> {
        self.lexer.next()
    }

    fn peek(&mut self) -> Option<Spanned<Token<'input>, usize, LexerError>> {
        self.lexer.peek().cloned()
    }

    fn peek_some(&mut self) -> Spanned<Token<'input>, usize, LexerError> {
        self.lexer.peek().cloned().ok_or(LexerError::EndOfFile)?
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Visibility {
    Internal,
    External,
    Private,
    Public,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DataLocation {
    Memory,
    Storage,
    Calldata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error() {
        let f = AbiError {
            name: "MyError".to_string(),
            inputs: vec![
                Param { name: "author".to_string(), kind: ParamType::Address, internal_type: None },
                Param {
                    name: "oldValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
                Param {
                    name: "newValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
            ],
        };
        let parsed = HumanReadableParser::parse_error(
            "error MyError(address author, string oldValue, string newValue)",
        )
        .unwrap();
        assert_eq!(f, parsed);
    }

    #[test]
    fn parse_constructor() {
        let f = Constructor {
            inputs: vec![
                Param { name: "author".to_string(), kind: ParamType::Address, internal_type: None },
                Param {
                    name: "oldValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
                Param {
                    name: "newValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
            ],
        };
        let parsed = HumanReadableParser::parse_constructor(
            "constructor(address author, string oldValue, string newValue)",
        )
        .unwrap();
        assert_eq!(f, parsed);
    }

    #[test]
    fn test_parse_function() {
        #[allow(deprecated)]
        let f = Function {
            name: "get".to_string(),
            inputs: vec![
                Param { name: "author".to_string(), kind: ParamType::Address, internal_type: None },
                Param {
                    name: "oldValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
                Param {
                    name: "newValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
            ],
            outputs: vec![],
            constant: None,
            state_mutability: Default::default(),
        };
        let parsed = HumanReadableParser::parse_function(
            "function get(address author, string oldValue, string newValue)",
        )
        .unwrap();
        assert_eq!(f, parsed);

        let parsed = HumanReadableParser::parse_function(
            "get(address author, string oldValue, string newValue)",
        )
        .unwrap();
        assert_eq!(f, parsed);

        #[allow(deprecated)]
        let f = Function {
            name: "get".to_string(),
            inputs: vec![
                Param { name: "".to_string(), kind: ParamType::Address, internal_type: None },
                Param { name: "".to_string(), kind: ParamType::String, internal_type: None },
                Param { name: "".to_string(), kind: ParamType::String, internal_type: None },
            ],
            outputs: vec![],
            constant: None,
            state_mutability: Default::default(),
        };

        let parsed =
            HumanReadableParser::parse_function("get(address , string , string )").unwrap();
        assert_eq!(f, parsed);
    }

    #[test]
    fn test_parse_function_output() {
        #[allow(deprecated)]
        let f = Function {
            name: "get".to_string(),
            inputs: vec![
                Param { name: "author".to_string(), kind: ParamType::Address, internal_type: None },
                Param {
                    name: "oldValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
                Param {
                    name: "newValue".to_string(),
                    kind: ParamType::String,
                    internal_type: None,
                },
            ],
            outputs: vec![
                Param {
                    name: "result".to_string(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                },
                Param { name: "output".to_string(), kind: ParamType::Address, internal_type: None },
            ],
            constant: None,
            state_mutability: Default::default(),
        };
        let parsed = HumanReadableParser::parse_function(
            "function get(address author, string oldValue, string newValue) returns (uint256 result, address output)",
        )
        .unwrap();
        assert_eq!(f, parsed);

        let parsed = HumanReadableParser::parse_function(
            " get(address author, string oldValue, string newValue) returns (uint256 result, address output)",
        )
        .unwrap();
        assert_eq!(f, parsed);
        #[allow(deprecated)]
        let mut f = Function {
            name: "get".to_string(),
            inputs: vec![
                Param { name: "".to_string(), kind: ParamType::Address, internal_type: None },
                Param { name: "".to_string(), kind: ParamType::String, internal_type: None },
                Param { name: "".to_string(), kind: ParamType::String, internal_type: None },
            ],
            outputs: vec![
                Param { name: "".to_string(), kind: ParamType::Uint(256), internal_type: None },
                Param { name: "".to_string(), kind: ParamType::Address, internal_type: None },
            ],
            constant: None,
            state_mutability: Default::default(),
        };
        let parsed = HumanReadableParser::parse_function(
            "function get(address, string, string) (uint256, address)",
        )
        .unwrap();
        assert_eq!(f, parsed);

        f.state_mutability = StateMutability::View;
        let parsed = HumanReadableParser::parse_function(
            "function get(address, string memory, string calldata) public view (uint256, address)",
        )
        .unwrap();
        assert_eq!(f, parsed);
    }

    #[test]
    fn test_parse_param() {
        assert_eq!(HumanReadableParser::parse_type("address").unwrap(), ParamType::Address);
        assert_eq!(HumanReadableParser::parse_type("bytes").unwrap(), ParamType::Bytes);
        assert_eq!(HumanReadableParser::parse_type("bytes32").unwrap(), ParamType::FixedBytes(32));
        assert_eq!(HumanReadableParser::parse_type("bool").unwrap(), ParamType::Bool);
        assert_eq!(HumanReadableParser::parse_type("string").unwrap(), ParamType::String);
        assert_eq!(HumanReadableParser::parse_type("int").unwrap(), ParamType::Int(256));
        assert_eq!(HumanReadableParser::parse_type("uint").unwrap(), ParamType::Uint(256));
        assert_eq!(
            HumanReadableParser::parse_type(
                "
        int32"
            )
            .unwrap(),
            ParamType::Int(32)
        );
        assert_eq!(HumanReadableParser::parse_type("uint32").unwrap(), ParamType::Uint(32));
    }

    #[test]
    fn test_parse_array_param() {
        assert_eq!(
            HumanReadableParser::parse_type("address[]").unwrap(),
            ParamType::Array(Box::new(ParamType::Address))
        );
        assert_eq!(
            HumanReadableParser::parse_type("uint[]").unwrap(),
            ParamType::Array(Box::new(ParamType::Uint(256)))
        );
        assert_eq!(
            HumanReadableParser::parse_type("bytes[]").unwrap(),
            ParamType::Array(Box::new(ParamType::Bytes))
        );
        assert_eq!(
            HumanReadableParser::parse_type("bool[][]").unwrap(),
            ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Bool))))
        );
    }

    #[test]
    fn test_parse_fixed_array_param() {
        assert_eq!(
            HumanReadableParser::parse_type("address[2]").unwrap(),
            ParamType::FixedArray(Box::new(ParamType::Address), 2)
        );
        assert_eq!(
            HumanReadableParser::parse_type("bool[17]").unwrap(),
            ParamType::FixedArray(Box::new(ParamType::Bool), 17)
        );
        assert_eq!(
            HumanReadableParser::parse_type("bytes[45][3]").unwrap(),
            ParamType::FixedArray(
                Box::new(ParamType::FixedArray(Box::new(ParamType::Bytes), 45)),
                3
            )
        );
    }

    #[test]
    fn test_parse_mixed_arrays() {
        assert_eq!(
            HumanReadableParser::parse_type("bool[][3]").unwrap(),
            ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 3)
        );
        assert_eq!(
            HumanReadableParser::parse_type("bool[3][]").unwrap(),
            ParamType::Array(Box::new(ParamType::FixedArray(Box::new(ParamType::Bool), 3)))
        );
    }

    #[test]
    fn test_parse_struct_param() {
        assert_eq!(
            HumanReadableParser::parse_type("(address,bool)").unwrap(),
            ParamType::Tuple(vec![ParamType::Address, ParamType::Bool])
        );
        assert_eq!(
            HumanReadableParser::parse_type("(bool[3],uint256)").unwrap(),
            ParamType::Tuple(vec![
                ParamType::FixedArray(Box::new(ParamType::Bool), 3),
                ParamType::Uint(256)
            ])
        );
    }

    #[test]
    fn test_parse_nested_struct_param() {
        assert_eq!(
            HumanReadableParser::parse_type("(address,bool,(bool,uint256))").unwrap(),
            ParamType::Tuple(vec![
                ParamType::Address,
                ParamType::Bool,
                ParamType::Tuple(vec![ParamType::Bool, ParamType::Uint(256)])
            ])
        );
    }

    #[test]
    fn test_parse_complex_nested_struct_param() {
        assert_eq!(
            HumanReadableParser::parse_type(
                "(address,bool,(bool,uint256,(bool,uint256)),(bool,uint256))"
            )
            .unwrap(),
            ParamType::Tuple(vec![
                ParamType::Address,
                ParamType::Bool,
                ParamType::Tuple(vec![
                    ParamType::Bool,
                    ParamType::Uint(256),
                    ParamType::Tuple(vec![ParamType::Bool, ParamType::Uint(256)])
                ]),
                ParamType::Tuple(vec![ParamType::Bool, ParamType::Uint(256)])
            ])
        );
    }

    #[test]
    fn test_parse_nested_tuple_array_param() {
        assert_eq!(
            HumanReadableParser::parse_type("(uint256,bytes32)[]").unwrap(),
            ParamType::Array(Box::new(ParamType::Tuple(vec![
                ParamType::Uint(256),
                ParamType::FixedBytes(32)
            ])))
        )
    }

    #[test]
    fn test_parse_inner_tuple_array_param() {
        let abi = "((uint256,bytes32)[],address)";
        let read = HumanReadableParser::parse_type(abi).unwrap();

        let param = ParamType::Tuple(vec![
            ParamType::Array(Box::new(ParamType::Tuple(vec![
                ParamType::Uint(256),
                ParamType::FixedBytes(32),
            ]))),
            ParamType::Address,
        ]);
        assert_eq!(read, param);
    }

    #[test]
    fn test_parse_complex_tuple_array_param() {
        let abi = "((uint256,uint256)[],(uint256,(uint256,uint256))[])";
        let read = HumanReadableParser::parse_type(abi).unwrap();
        let param = ParamType::Tuple(vec![
            ParamType::Array(Box::new(ParamType::Tuple(vec![
                ParamType::Uint(256),
                ParamType::Uint(256),
            ]))),
            ParamType::Array(Box::new(ParamType::Tuple(vec![
                ParamType::Uint(256),
                ParamType::Tuple(vec![ParamType::Uint(256), ParamType::Uint(256)]),
            ]))),
        ]);
        assert_eq!(read, param);
    }

    #[test]
    fn test_parse_event() {
        let abi = "event ValueChanged(address indexed author, string oldValue, string newValue)";
        let event = HumanReadableParser::parse_event(abi).unwrap();

        assert_eq!(
            Event {
                name: "ValueChanged".to_string(),
                inputs: vec![
                    EventParam {
                        name: "author".to_string(),
                        kind: ParamType::Address,
                        indexed: true
                    },
                    EventParam {
                        name: "oldValue".to_string(),
                        kind: ParamType::String,
                        indexed: false
                    },
                    EventParam {
                        name: "newValue".to_string(),
                        kind: ParamType::String,
                        indexed: false
                    }
                ],
                anonymous: false
            },
            event
        );
    }

    #[test]
    fn parse_large_function() {
        let f = "function atomicMatch_(address[14] addrs, uint[18] uints, uint8[8] feeMethodsSidesKindsHowToCalls, bytes calldataBuy, bytes calldataSell, bytes replacementPatternBuy, bytes replacementPatternSell, bytes staticExtradataBuy, bytes staticExtradataSell, uint8[2] vs, bytes32[5] rssMetadata) public payable";

        let _fun = HumanReadableParser::parse_function(f).unwrap();
    }
}
