use std::collections::{HashMap, VecDeque};

use crate::abi::error::{bail, format_err, ParseError, Result};
use crate::abi::struct_def::{FieldType, StructFieldType};
use crate::abi::{
    param_type::Reader, Abi, Constructor, Event, EventParam, Function, Param, ParamType, SolStruct,
    StateMutability,
};

/// A parser that turns a "human readable abi" into a `Abi`
pub struct AbiParser {
    /// solidity structs
    pub structs: HashMap<String, SolStruct>,
    /// solidity structs as tuples
    pub struct_tuples: HashMap<String, Vec<ParamType>>,
}

impl AbiParser {
    /// Parses a "human readable abi" string
    ///
    /// # Example
    ///
    /// ```
    ///  # use ethers::abi::AbiParser;
    /// let abi = AbiParser::default().parse_str(r#"[
    ///         function setValue(string)
    ///         function getValue() external view returns (string)
    ///         event ValueChanged(address indexed author, string oldValue, string newValue)
    ///     ]"#).unwrap();
    /// ```
    pub fn parse_str(&mut self, s: &str) -> Result<Abi> {
        self.parse(
            &s.trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .lines()
                .collect::<Vec<_>>(),
        )
    }

    /// Parses a "human readable abi" string vector
    ///
    /// # Example
    /// ```
    /// use ethers::abi::AbiParser;
    ///
    /// let abi = AbiParser::default().parse(&[
    ///     "function x() external view returns (uint256)",
    /// ]).unwrap();
    /// ```
    pub fn parse(&mut self, input: &[&str]) -> Result<Abi> {
        // parse struct first
        let mut abi = Abi {
            constructor: None,
            functions: HashMap::new(),
            events: HashMap::new(),
            receive: false,
            fallback: false,
        };

        let (structs, types): (Vec<_>, Vec<_>) = input
            .iter()
            .map(|s| escape_quotes(s))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .partition(|s| s.starts_with("struct"));

        for sol in structs {
            let s = SolStruct::parse(sol)?;
            if self.structs.contains_key(s.name()) {
                bail!("Duplicate struct declaration for struct `{}`", s.name())
            }
            self.structs.insert(s.name().to_string(), s);
        }
        self.substitute_structs()?;

        for mut line in types {
            line = line.trim_start();
            if line.starts_with("function") {
                let function = self.parse_function(line)?;
                abi.functions
                    .entry(function.name.clone())
                    .or_default()
                    .push(function);
            } else if line.starts_with("event") {
                let event = self.parse_event(line)?;
                abi.events
                    .entry(event.name.clone())
                    .or_default()
                    .push(event);
            } else if line.starts_with("constructor") {
                abi.constructor = Some(self.parse_constructor(line)?);
            } else {
                bail!("Illegal abi `{}`", line)
            }
        }
        Ok(abi)
    }

    /// Substitutes any other struct references within structs with tuples
    fn substitute_structs(&mut self) -> Result<()> {
        let mut unresolved = self.structs.keys().collect::<VecDeque<_>>();
        let mut sequential_retries = 0;
        while let Some(name) = unresolved.pop_front() {
            let mut resolved = true;
            let sol = &self.structs[name];
            let mut tuple = Vec::with_capacity(sol.fields().len());
            for field in sol.fields() {
                match field.r#type() {
                    FieldType::Elementary(param) => tuple.push(param.clone()),
                    FieldType::Struct(ty) => {
                        if let Some(param) = self.struct_tuples.get(ty.name()).cloned() {
                            tuple.push(ParamType::Tuple(param))
                        } else {
                            resolved = false;
                            break;
                        }
                    }
                    FieldType::StructArray(ty) => {
                        if let Some(param) = self.struct_tuples.get(ty.name()).cloned() {
                            tuple.push(ParamType::Array(Box::new(ParamType::Tuple(param))))
                        } else {
                            resolved = false;
                            break;
                        }
                    }
                    FieldType::FixedStructArray(ty, size) => {
                        if let Some(param) = self.struct_tuples.get(ty.name()).cloned() {
                            tuple.push(ParamType::FixedArray(
                                Box::new(ParamType::Tuple(param)),
                                *size,
                            ))
                        } else {
                            resolved = false;
                            break;
                        }
                    }
                    FieldType::Mapping(_) => {
                        bail!(
                            "mappings are not allowed as params in public functions of struct `{}`",
                            sol.name()
                        )
                    }
                }
            }
            if resolved {
                sequential_retries = 0;
                self.struct_tuples.insert(sol.name().to_string(), tuple);
            } else {
                sequential_retries += 1;
                if sequential_retries > unresolved.len() {
                    bail!("No struct definition found for struct `{}`", name)
                }
                unresolved.push_back(name);
            }
        }
        Ok(())
    }

    /// Link additional structs for parsing
    pub fn with_structs(structs: Vec<SolStruct>) -> Self {
        Self {
            structs: structs
                .into_iter()
                .map(|s| (s.name().to_string(), s))
                .collect(),
            struct_tuples: HashMap::new(),
        }
    }

    /// Parses a solidity event declaration from `event <name> (args*) anonymous?`
    pub fn parse_event(&self, s: &str) -> Result<Event> {
        let mut event = s.trim();
        if !event.starts_with("event ") {
            bail!("Not an event `{}`", s)
        }
        event = &event[5..];

        let name = parse_identifier(&mut event)?;

        let mut chars = event.chars();

        loop {
            match chars.next() {
                None => bail!("Expected event"),
                Some('(') => {
                    event = chars.as_str().trim();
                    let mut anonymous = false;
                    if event.ends_with("anonymous") {
                        anonymous = true;
                        event = event[..event.len() - 9].trim_end();
                    }
                    event = event
                        .trim()
                        .strip_suffix(')')
                        .ok_or_else(|| format_err!("Expected closing `)` in `{}`", s))?;

                    let inputs = if event.is_empty() {
                        Vec::new()
                    } else {
                        event
                            .split(',')
                            .map(|e| self.parse_event_arg(e))
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    return Ok(Event {
                        name,
                        inputs,
                        anonymous,
                    });
                }
                Some(' ') | Some('\t') => {
                    continue;
                }
                Some(c) => {
                    bail!("Illegal char `{}` at `{}`", c, s)
                }
            }
        }
    }

    /// Parse a single event param
    fn parse_event_arg(&self, input: &str) -> Result<EventParam> {
        let mut iter = input.trim().rsplitn(3, is_whitespace);
        let mut indexed = false;
        let mut name = iter
            .next()
            .ok_or_else(|| format_err!("Empty event param at `{}`", input))?;

        let type_str;
        if let Some(mid) = iter.next() {
            if let Some(ty) = iter.next() {
                if mid != "indexed" {
                    bail!("Expected indexed keyword at `{}`", input)
                }
                indexed = true;
                type_str = ty;
            } else {
                if name == "indexed" {
                    indexed = true;
                    name = "";
                }
                type_str = mid;
            }
        } else {
            type_str = name;
            name = "";
        }

        Ok(EventParam {
            name: name.to_string(),
            indexed,
            kind: self.parse_type(type_str)?,
        })
    }

    pub fn parse_function(&mut self, s: &str) -> Result<Function> {
        let mut input = s.trim();
        if !input.starts_with("function ") {
            bail!("Not a function `{}`", input)
        }
        input = &input[8..];
        let name = parse_identifier(&mut input)?;

        let mut iter = input.split(" returns");

        let parens = iter
            .next()
            .ok_or_else(|| format_err!("Invalid function declaration at `{}`", s))?
            .trim_end();

        let mut parens_iter = parens.rsplitn(2, ')');
        let mut modifiers = parens_iter.next();

        let input_params = if let Some(args) = parens_iter.next() {
            args
        } else {
            modifiers
                .take()
                .ok_or(ParseError::ParseError(super::Error::InvalidData))?
        }
        .trim_start()
        .strip_prefix('(')
        .ok_or(ParseError::ParseError(super::Error::InvalidData))?;

        let inputs = self.parse_params(input_params)?;

        let outputs = if let Some(params) = iter.next() {
            let params = params
                .trim()
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
                .ok_or_else(|| format_err!("Expected parentheses at `{}`", s))?;
            self.parse_params(params)?
        } else {
            Vec::new()
        };

        let state_mutability = modifiers.map(detect_state_mutability).unwrap_or_default();

        #[allow(deprecated)]
        Ok(Function {
            name,
            inputs,
            outputs,
            state_mutability,
            constant: false,
        })
    }

    fn parse_params(&self, s: &str) -> Result<Vec<Param>> {
        s.split(',')
            .filter(|s| !s.is_empty())
            .map(|s| self.parse_param(s))
            .collect::<Result<Vec<_>, _>>()
    }

    fn parse_type(&self, type_str: &str) -> Result<ParamType> {
        if let Ok(kind) = Reader::read(type_str) {
            Ok(kind)
        } else {
            // try struct instead
            if let Ok(field) = StructFieldType::parse(type_str) {
                let struct_ty = field
                    .as_struct()
                    .ok_or_else(|| format_err!("Expected struct type `{}`", type_str))?;
                let tuple = self
                    .struct_tuples
                    .get(struct_ty.name())
                    .cloned()
                    .map(ParamType::Tuple)
                    .ok_or_else(|| format_err!("Unknown struct `{}`", struct_ty.name()))?;

                match field {
                    FieldType::Struct(_) => Ok(tuple),
                    FieldType::StructArray(_) => Ok(ParamType::Array(Box::new(tuple))),
                    FieldType::FixedStructArray(_, size) => {
                        Ok(ParamType::FixedArray(Box::new(tuple), size))
                    }
                    _ => bail!("Expected struct type"),
                }
            } else {
                bail!("Failed determine event type `{}`", type_str)
            }
        }
    }

    pub fn parse_constructor(&self, s: &str) -> Result<Constructor> {
        let mut input = s.trim();
        if !input.starts_with("constructor") {
            bail!("Not a constructor `{}`", input)
        }
        input = input[11..]
            .trim_start()
            .strip_prefix('(')
            .ok_or_else(|| format_err!("Expected leading `(` in `{}`", s))?;

        let params = input
            .rsplitn(2, ')')
            .last()
            .ok_or_else(|| format_err!("Expected closing `)` in `{}`", s))?;

        let inputs = self.parse_params(params)?;

        Ok(Constructor { inputs })
    }

    fn parse_param(&self, param: &str) -> Result<Param> {
        let mut iter = param.trim().rsplitn(3, is_whitespace);

        let mut name = iter
            .next()
            .ok_or(ParseError::ParseError(super::Error::InvalidData))?;

        let type_str;
        if let Some(ty) = iter.last() {
            if name == "memory" || name == "calldata" {
                name = "";
            }
            type_str = ty;
        } else {
            type_str = name;
            name = "";
        }

        Ok(Param {
            name: name.to_string(),
            kind: self.parse_type(type_str)?,
        })
    }
}

impl Default for AbiParser {
    fn default() -> Self {
        Self::with_structs(Vec::new())
    }
}

/// Parses a "human readable abi" string vector
///
/// ```
/// use ethers::abi::parse_abi;
///
/// let abi = parse_abi(&[
///     "function x() external view returns (uint256)",
/// ]).unwrap();
/// ```
pub fn parse(input: &[&str]) -> Result<Abi> {
    AbiParser::default().parse(input)
}

/// Parses a "human readable abi" string
///
/// See also `AbiParser::parse_str`
pub fn parse_str(input: &str) -> Result<Abi> {
    AbiParser::default().parse_str(input)
}

/// Parses an identifier like event or function name
pub(crate) fn parse_identifier(input: &mut &str) -> Result<String> {
    let mut chars = input.trim_start().chars();
    let mut name = String::new();
    let c = chars
        .next()
        .ok_or_else(|| format_err!("Empty identifier in `{}`", input))?;
    if is_first_ident_char(c) {
        name.push(c);
        loop {
            match chars.clone().next() {
                Some(c) if is_ident_char(c) => {
                    chars.next();
                    name.push(c);
                }
                _ => break,
            }
        }
    }
    if name.is_empty() {
        return Err(ParseError::ParseError(super::Error::InvalidName(
            input.to_string(),
        )));
    }
    *input = chars.as_str();
    Ok(name)
}

fn detect_state_mutability(s: &str) -> StateMutability {
    if s.contains("pure") {
        StateMutability::Pure
    } else if s.contains("view") {
        StateMutability::View
    } else if s.contains("payable") {
        StateMutability::Payable
    } else {
        StateMutability::NonPayable
    }
}

pub(crate) fn is_first_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

pub(crate) fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
}

pub(crate) fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

fn escape_quotes(input: &str) -> &str {
    input.trim_matches(is_whitespace).trim_matches('\"')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_approve() {
        let fn_str = "function approve(address _spender, uint256 value) external returns(bool)";
        let parsed = AbiParser::default().parse_function(fn_str).unwrap();
        assert_eq!(parsed.name, "approve");
        assert_eq!(parsed.inputs[0].name, "_spender");
        assert_eq!(parsed.inputs[0].kind, ParamType::Address,);
        assert_eq!(parsed.inputs[1].name, "value");
        assert_eq!(parsed.inputs[1].kind, ParamType::Uint(256),);
        assert_eq!(parsed.outputs[0].name, "");
        assert_eq!(parsed.outputs[0].kind, ParamType::Bool);
    }

    #[test]
    fn parses_function_arguments_return() {
        let fn_str = "function foo(uint32[] memory x) external view returns (address)";
        let parsed = AbiParser::default().parse_function(fn_str).unwrap();
        assert_eq!(parsed.name, "foo");
        assert_eq!(parsed.inputs[0].name, "x");
        assert_eq!(
            parsed.inputs[0].kind,
            ParamType::Array(Box::new(ParamType::Uint(32)))
        );
        assert_eq!(parsed.outputs[0].name, "");
        assert_eq!(parsed.outputs[0].kind, ParamType::Address);
    }

    #[test]
    fn parses_function_empty() {
        let fn_str = "function foo()";
        let parsed = AbiParser::default().parse_function(fn_str).unwrap();
        assert_eq!(parsed.name, "foo");
        assert!(parsed.inputs.is_empty());
        assert!(parsed.outputs.is_empty());
    }

    #[test]
    fn parses_function_payable() {
        let fn_str = "function foo() public payable";
        let parsed = AbiParser::default().parse_function(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::Payable);
    }

    #[test]
    fn parses_function_view() {
        let fn_str = "function foo() external view";
        let parsed = AbiParser::default().parse_function(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::View);
    }

    #[test]
    fn parses_function_pure() {
        let fn_str = "function foo()  pure";
        let parsed = AbiParser::default().parse_function(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::Pure);
    }

    #[test]
    fn parses_event() {
        assert_eq!(
            AbiParser::default()
                .parse_event("event Foo (address indexed x, uint y, bytes32[] z)")
                .unwrap(),
            Event {
                anonymous: false,
                name: "Foo".to_string(),
                inputs: vec![
                    EventParam {
                        name: "x".to_string(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    EventParam {
                        name: "y".to_string(),
                        kind: ParamType::Uint(256),
                        indexed: false,
                    },
                    EventParam {
                        name: "z".to_string(),
                        kind: ParamType::Array(Box::new(ParamType::FixedBytes(32))),
                        indexed: false,
                    },
                ],
            }
        );
    }

    #[test]
    fn parses_anonymous_event() {
        assert_eq!(
            AbiParser::default()
                .parse_event("event Foo() anonymous")
                .unwrap(),
            Event {
                anonymous: true,
                name: "Foo".to_string(),
                inputs: vec![],
            }
        );
    }

    #[test]
    fn parses_unnamed_event() {
        assert_eq!(
            AbiParser::default()
                .parse_event("event Foo(address)")
                .unwrap(),
            Event {
                anonymous: false,
                name: "Foo".to_string(),
                inputs: vec![EventParam {
                    name: "".to_string(),
                    kind: ParamType::Address,
                    indexed: false,
                }],
            }
        );
    }

    #[test]
    fn parses_unnamed_indexed_event() {
        assert_eq!(
            AbiParser::default()
                .parse_event("event Foo(address indexed)")
                .unwrap(),
            Event {
                anonymous: false,
                name: "Foo".to_string(),
                inputs: vec![EventParam {
                    name: "".to_string(),
                    kind: ParamType::Address,
                    indexed: true,
                }],
            }
        );
    }

    #[test]
    fn parse_event_input() {
        assert_eq!(
            AbiParser::default()
                .parse_event_arg("address indexed x")
                .unwrap(),
            EventParam {
                name: "x".to_string(),
                kind: ParamType::Address,
                indexed: true,
            }
        );

        assert_eq!(
            AbiParser::default().parse_event_arg("address x").unwrap(),
            EventParam {
                name: "x".to_string(),
                kind: ParamType::Address,
                indexed: false,
            }
        );
    }

    #[test]
    fn can_parse_functions() {
        [
            "function foo(uint256[] memory x) external view returns (address)",
            "function bar(uint256[] memory x) returns (address)",
            "function bar(uint256[] memory x, uint32 y) returns (address, uint256)",
            "function foo(address[] memory, bytes memory) returns (bytes memory)",
            "function bar(uint256[] memory x)",
            "function bar()",
        ]
        .iter()
        .for_each(|x| {
            AbiParser::default().parse_function(x).unwrap();
        });
    }

    #[test]
    fn can_parse_structs_and_functions() {
        let abi = &[
            "struct Demo {bytes  x; address payable d;}",
            "struct Voter {  uint weight;  bool voted;  address delegate; uint vote; }",
            "event FireEvent(Voter v, NestedVoter2 n)",
            "function foo(uint256[] memory x) external view returns (address)",
            "function call(Voter memory voter) returns (address, uint256)",
            "struct NestedVoter {  Voter voter;  bool voted;  address delegate; uint vote; }",
            "struct NestedVoter2 {  NestedVoter[] voter;  Voter[10] votes;  address delegate; uint vote; }",
        ];
        parse(abi).unwrap();
    }

    #[test]
    fn can_parse_params() {
        [
            "address x",
            "address",
            "bytes memory y",
            "bytes memory",
            "bytes32[] memory",
            "bytes32[] memory z",
        ]
        .iter()
        .for_each(|x| {
            AbiParser::default().parse_param(x).unwrap();
        });
    }

    #[test]
    fn can_read_backslashes() {
        parse(&[
            "\"function setValue(string)\"",
            "\"function getValue() external view returns(string)\"",
        ])
        .unwrap();
    }

    #[test]
    fn can_substitute_structs() {
        let abi = parse(&[
            "struct MyStruct {int y; address _addr;}",
            "event FireEvent(MyStruct m, address indexed newOwner)",
        ])
        .unwrap();
        assert_eq!(
            abi.events["FireEvent"][0].inputs.clone(),
            vec![
                EventParam {
                    name: "m".to_string(),
                    kind: ParamType::Tuple(vec![ParamType::Int(256), ParamType::Address]),
                    indexed: false
                },
                EventParam {
                    name: "newOwner".to_string(),
                    kind: ParamType::Address,
                    indexed: true
                },
            ]
        );
    }

    #[test]
    fn can_substitute_array_structs() {
        let abi = parse(&[
            "struct MyStruct {int y; address _addr;}",
            "event FireEvent(MyStruct[] m, MyStruct[10] m2)",
        ])
        .unwrap();

        assert_eq!(
            abi.events["FireEvent"][0].inputs.clone(),
            vec![
                EventParam {
                    name: "m".to_string(),
                    kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
                        ParamType::Int(256),
                        ParamType::Address
                    ]))),
                    indexed: false
                },
                EventParam {
                    name: "m2".to_string(),
                    kind: ParamType::FixedArray(
                        Box::new(ParamType::Tuple(vec![
                            ParamType::Int(256),
                            ParamType::Address
                        ])),
                        10
                    ),
                    indexed: false
                },
            ]
        );
    }

    #[test]
    fn can_substitute_nested_array_structs() {
        let abi = parse(&[
            "struct MyStruct {int y; address _addr;}",
            "event FireEvent(MyStruct[] m, MyStructWrapper w)",
            "struct MyStructWrapper {MyStruct y; int y; address _addr;}",
        ])
        .unwrap();

        assert_eq!(
            abi.events["FireEvent"][0].inputs.clone(),
            vec![
                EventParam {
                    name: "m".to_string(),
                    kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
                        ParamType::Int(256),
                        ParamType::Address
                    ]))),
                    indexed: false
                },
                EventParam {
                    name: "w".to_string(),
                    kind: ParamType::Tuple(vec![
                        ParamType::Tuple(vec![ParamType::Int(256), ParamType::Address]),
                        ParamType::Int(256),
                        ParamType::Address
                    ]),
                    indexed: false
                },
            ]
        );
    }
}
