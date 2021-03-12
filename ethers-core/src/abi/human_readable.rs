use std::collections::HashMap;

use thiserror::Error;

use super::{
    param_type::Reader, Abi, Constructor, Event, EventParam, Function, Param, StateMutability,
};

/// Parses a "human readable abi" string vector
///
/// ```
/// use ethers::abi::parse_abi;
///
/// let abi = parse_abi(&[
///     "function x() external view returns (uint256)",
/// ]).unwrap();
/// ```
pub fn parse(input: &[&str]) -> Result<Abi, ParseError> {
    let mut abi = Abi {
        constructor: None,
        functions: HashMap::new(),
        events: HashMap::new(),
        receive: false,
        fallback: false,
    };

    for mut line in input.iter().map(|s| escape_quotes(s)) {
        line = line.trim_start();
        if line.starts_with("function") {
            let function = parse_function(&line)?;
            abi.functions
                .entry(function.name.clone())
                .or_default()
                .push(function);
        } else if line.starts_with("event") {
            let event = parse_event(line)?;
            abi.events
                .entry(event.name.clone())
                .or_default()
                .push(event);
        } else if line.starts_with("constructor") {
            abi.constructor = Some(parse_constructor(line)?);
        } else {
            return Err(ParseError::ParseError(super::Error::InvalidData));
        }
    }

    Ok(abi)
}

/// Parses an identifier like event or function name
fn parse_identifier(input: &mut &str) -> Result<String, ParseError> {
    let mut chars = input.trim_start().chars();
    let mut name = String::new();
    let c = chars
        .next()
        .ok_or(ParseError::ParseError(super::Error::InvalidData))?;
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
    *input = chars.as_str();
    Ok(name)
}

/// Parses a solidity event declaration from `event <name> (args*) anonymous?`
fn parse_event(mut event: &str) -> Result<Event, ParseError> {
    event = event.trim();
    if !event.starts_with("event ") {
        return Err(ParseError::ParseError(super::Error::InvalidData));
    }
    event = &event[5..];

    let name = parse_identifier(&mut event)?;
    if name.is_empty() {
        return Err(ParseError::ParseError(super::Error::InvalidName(
            event.to_owned(),
        )));
    }

    let mut chars = event.chars();

    loop {
        match chars.next() {
            None => return Err(ParseError::ParseError(super::Error::InvalidData)),
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
                    .ok_or(ParseError::ParseError(super::Error::InvalidData))?;

                let inputs = if event.is_empty() {
                    Vec::new()
                } else {
                    event
                        .split(',')
                        .map(parse_event_arg)
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
            _ => {
                return Err(ParseError::ParseError(super::Error::InvalidData));
            }
        }
    }
}

/// Parse a single event param
fn parse_event_arg(input: &str) -> Result<EventParam, ParseError> {
    let mut iter = input.trim().rsplitn(3, is_whitespace);
    let mut indexed = false;
    let mut name = iter
        .next()
        .ok_or(ParseError::ParseError(super::Error::InvalidData))?;

    if let Some(mid) = iter.next() {
        let kind;
        if let Some(ty) = iter.next() {
            if mid != "indexed" {
                return Err(ParseError::ParseError(super::Error::InvalidData));
            }
            indexed = true;
            kind = Reader::read(ty)?;
        } else {
            if name == "indexed" {
                indexed = true;
                name = "";
            }
            kind = Reader::read(mid)?;
        }
        Ok(EventParam {
            name: name.to_owned(),
            kind,
            indexed,
        })
    } else {
        Ok(EventParam {
            name: "".to_owned(),
            indexed,
            kind: Reader::read(name)?,
        })
    }
}

fn parse_function(mut input: &str) -> Result<Function, ParseError> {
    input = input.trim();
    if !input.starts_with("function ") {
        return Err(ParseError::ParseError(super::Error::InvalidData));
    }
    input = &input[8..];
    let name = parse_identifier(&mut input)?;
    if name.is_empty() {
        return Err(ParseError::ParseError(super::Error::InvalidName(
            input.to_owned(),
        )));
    }

    let mut iter = input.split(" returns");

    let parens = iter
        .next()
        .ok_or(ParseError::ParseError(super::Error::InvalidData))?
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

    let inputs = input_params
        .split(',')
        .filter(|s| !s.is_empty())
        .map(parse_param)
        .collect::<Result<Vec<_>, _>>()?;

    let outputs = if let Some(params) = iter.next() {
        let params = params
            .trim()
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .ok_or(ParseError::ParseError(super::Error::InvalidData))?;
        params
            .split(',')
            .filter(|s| !s.is_empty())
            .map(parse_param)
            .collect::<Result<Vec<_>, _>>()?
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

fn parse_constructor(mut input: &str) -> Result<Constructor, ParseError> {
    input = input.trim();
    if !input.starts_with("constructor") {
        return Err(ParseError::ParseError(super::Error::InvalidData));
    }
    input = input[11..]
        .trim_start()
        .strip_prefix('(')
        .ok_or(ParseError::ParseError(super::Error::InvalidData))?;

    let params = input
        .rsplitn(2, ')')
        .last()
        .ok_or(ParseError::ParseError(super::Error::InvalidData))?;

    let inputs = params
        .split(',')
        .filter(|s| !s.is_empty())
        .map(parse_param)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Constructor { inputs })
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

fn parse_param(param: &str) -> Result<Param, ParseError> {
    let mut iter = param.trim().rsplitn(3, is_whitespace);

    let name = iter
        .next()
        .ok_or(ParseError::ParseError(super::Error::InvalidData))?;

    if let Some(ty) = iter.last() {
        if name == "memory" || name == "calldata" {
            Ok(Param {
                name: "".to_owned(),
                kind: Reader::read(ty)?,
            })
        } else {
            Ok(Param {
                name: name.to_owned(),
                kind: Reader::read(ty)?,
            })
        }
    } else {
        Ok(Param {
            name: "".to_owned(),
            kind: Reader::read(name)?,
        })
    }
}

fn is_first_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

fn escape_quotes(input: &str) -> &str {
    input.trim_matches(is_whitespace).trim_matches('\"')
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("expected data type")]
    Kind,

    #[error(transparent)]
    ParseError(#[from] super::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::ParamType;

    #[test]
    fn parses_approve() {
        let fn_str = "function approve(address _spender, uint256 value) external returns(bool)";
        let parsed = parse_function(fn_str).unwrap();
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
        let parsed = parse_function(fn_str).unwrap();
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
        let parsed = parse_function(fn_str).unwrap();
        assert_eq!(parsed.name, "foo");
        assert!(parsed.inputs.is_empty());
        assert!(parsed.outputs.is_empty());
    }

    #[test]
    fn parses_function_payable() {
        let fn_str = "function foo() public payable";
        let parsed = parse_function(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::Payable);
    }

    #[test]
    fn parses_function_view() {
        let fn_str = "function foo() external view";
        let parsed = parse_function(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::View);
    }

    #[test]
    fn parses_function_pure() {
        let fn_str = "function foo()  pure";
        let parsed = parse_function(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::Pure);
    }

    #[test]
    fn parses_event() {
        assert_eq!(
            parse_event(&mut "event Foo (address indexed x, uint y, bytes32[] z)").unwrap(),
            Event {
                anonymous: false,
                name: "Foo".to_owned(),
                inputs: vec![
                    EventParam {
                        name: "x".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    EventParam {
                        name: "y".to_owned(),
                        kind: ParamType::Uint(256),
                        indexed: false,
                    },
                    EventParam {
                        name: "z".to_owned(),
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
            parse_event(&mut "event Foo() anonymous").unwrap(),
            Event {
                anonymous: true,
                name: "Foo".to_owned(),
                inputs: vec![],
            }
        );
    }

    #[test]
    fn parses_unnamed_event() {
        assert_eq!(
            parse_event(&mut "event Foo(address)").unwrap(),
            Event {
                anonymous: false,
                name: "Foo".to_owned(),
                inputs: vec![EventParam {
                    name: "".to_owned(),
                    kind: ParamType::Address,
                    indexed: false,
                }],
            }
        );
    }

    #[test]
    fn parses_unnamed_indexed_event() {
        assert_eq!(
            parse_event(&mut "event Foo(address indexed)").unwrap(),
            Event {
                anonymous: false,
                name: "Foo".to_owned(),
                inputs: vec![EventParam {
                    name: "".to_owned(),
                    kind: ParamType::Address,
                    indexed: true,
                }],
            }
        );
    }

    #[test]
    fn parse_event_input() {
        assert_eq!(
            parse_event_arg("address indexed x").unwrap(),
            EventParam {
                name: "x".to_owned(),
                kind: ParamType::Address,
                indexed: true,
            }
        );

        assert_eq!(
            parse_event_arg("address x").unwrap(),
            EventParam {
                name: "x".to_owned(),
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
            parse_function(x).unwrap();
        });
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
            parse_param(x).unwrap();
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
}
