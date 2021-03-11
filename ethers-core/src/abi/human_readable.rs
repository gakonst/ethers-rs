use std::collections::HashMap;

use thiserror::Error;

use super::{
    Abi, Event, EventParam, Function, Param, param_type::Reader, ParamType, StateMutability,
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

    for line in input {
        if line.contains("function") {
            let function = parse_function(&line)?;
            abi.functions
                .entry(function.name.clone())
                .or_default()
                .push(function);
        } else if line.contains("event") {
            let event = parse_event(&line)?;
            abi.events
                .entry(event.name.clone())
                .or_default()
                .push(event);
        } else if line.starts_with("struct") {
            panic!("Got tuple");
        } else {
            panic!("unknown sig")
        }
    }

    Ok(abi)
}

/// Parses an identifier like event or function name
fn parse_identifier(input: &mut &str) -> Result<String, ParseError> {
    let mut chars = input.trim_start().chars();
    let mut name = String::new();
    let c = chars.next().ok_or(ParseError::ParseError(super::Error::InvalidData))?;
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
fn parse_event2(mut event: &str) -> Result<Event, ParseError> {
    event = event.trim();
    if !event.starts_with("event ") {
        return Err(ParseError::ParseError(super::Error::InvalidData));
    }
    event = &event[6..];

    let name = parse_identifier(&mut event)?;
    if name.is_empty() {
        return Err(ParseError::ParseError(super::Error::InvalidName(event.to_owned())));
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
                let inputs = parse_event_args(event)?;
                return Ok(Event {
                    name,
                    anonymous,
                    inputs,
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

/// Returns the event parameters
fn parse_event_args(mut input: &str) -> Result<Vec<EventParam>, ParseError> {
    input = input.trim().strip_suffix(')').ok_or(ParseError::ParseError(super::Error::InvalidData))?;

    let mut params = Vec::new();

    if input.is_empty() {
        return Ok(params);
    }

    for arg in input.split(',') {
        let mut iter = arg.trim().rsplitn(3, is_whitespace);
        let name = iter.next().ok_or(ParseError::ParseError(super::Error::InvalidData))?;
        let mid = iter.next().ok_or(ParseError::ParseError(super::Error::InvalidData))?;
        let mut kind = None;
        let mut indexed = false;
        if mid == "indexed" {
            indexed = true;
            kind = iter.next().map(Reader::read);
        } else {
            kind = Some(Reader::read(mid));
            if iter.next().is_some() {
                return Err(ParseError::ParseError(super::Error::InvalidData));
            }
        }
        params.push(
            EventParam {
                name: name.to_owned(),
                kind: kind.ok_or(ParseError::ParseError(super::Error::InvalidData))??,
                indexed,
            }
        )
    }
    Ok(params)
}


fn parse_function2(mut input: &str) -> Result<Function, ParseError> {
    input = input.trim();
    if !input.starts_with("function ") {
        return Err(ParseError::ParseError(super::Error::InvalidData));
    }
    input = &input[9..];
    let name = parse_identifier(&mut input)?;
    if name.is_empty() {
        return Err(ParseError::ParseError(super::Error::InvalidName(input.to_owned())));
    }

    let mut iter = input.split(" returns");

    let parens = iter.next().ok_or(ParseError::ParseError(super::Error::InvalidData))?.trim_end();

    let mut parens_iter = parens.rsplitn(2, ')');
    let mut modifiers = parens_iter.next();

    let input_params = if let Some(args) = parens_iter.next() {
        args
    } else {
        modifiers.take().ok_or(ParseError::ParseError(super::Error::InvalidData))?
    }.trim_start().strip_prefix('(').ok_or(ParseError::ParseError(super::Error::InvalidData))?;


    let inputs = input_params.split(',').filter(|s|!s.is_empty()).map(parse_param2).collect::<Result<Vec<_>,_>>()?;

    let outputs = if let Some(params) = iter.next() {
        let params = params.trim().strip_prefix('(').and_then(|s|s.strip_suffix(')')).ok_or(ParseError::ParseError(super::Error::InvalidData))?;
        params.split(',').filter(|s|!s.is_empty()).map(parse_param2).collect::<Result<Vec<_>,_>>()?
    } else {
        Vec::new()
    };

    let state_mutability = if let Some(modifiers) = modifiers {
        if modifiers.contains("pure") {
            StateMutability::Pure
        } else if modifiers.contains("view") {
            StateMutability::View
        } else if modifiers.contains("payable") {
            StateMutability::Payable
        } else {
            StateMutability::NonPayable
        }
    } else {
        StateMutability::NonPayable
    };

    #[allow(deprecated)]
        Ok(Function {
        name,
        inputs,
        outputs,
        state_mutability,
        constant: false,
    })
}

fn parse2(input: &[&str]) -> Result<Abi, ParseError> {
    let mut abi = Abi {
        constructor: None,
        functions: HashMap::new(),
        events: HashMap::new(),
        receive: false,
        fallback: false,
    };

    for line in input {
        let line = line.trim_start();
        if line.starts_with("function") {
            let function = parse_function2(&line)?;
            abi.functions
                .entry(function.name.clone())
                .or_default()
                .push(function);
        } else if line.starts_with("event") {
            let event = parse_event2(line)?;
            abi.events
                .entry(event.name.clone())
                .or_default()
                .push(event);
        } else if line.starts_with("constructor") {} else {
            return Err(ParseError::ParseError(super::Error::InvalidData));
        }
    }

    Ok(abi)
}


fn is_first_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' )
}

fn parse_event(event: &str) -> Result<Event, ParseError> {
    let split: Vec<&str> = event.split("event ").collect();
    let split: Vec<&str> = split[1].split('(').collect();
    let name = split[0].trim_end();
    let rest = split[1];

    let args = rest.replace(")", "");
    let anonymous = rest.contains("anonymous");

    let inputs = if args.contains(',') {
        let args: Vec<&str> = args.split(", ").collect();
        args.iter()
            .map(|arg| parse_event_arg(arg))
            .collect::<Result<Vec<EventParam>, _>>()?
    } else {
        vec![]
    };

    Ok(Event {
        name: name.to_owned(),
        anonymous,
        inputs,
    })
}

// Parses an event's argument as indexed if neded
fn parse_event_arg(param: &str) -> Result<EventParam, ParseError> {
    let tokens: Vec<&str> = param.split(' ').collect();
    let kind: ParamType = Reader::read(tokens[0])?;
    let (name, indexed) = if tokens.len() == 2 {
        (tokens[1], false)
    } else {
        (tokens[2], true)
    };

    Ok(EventParam {
        name: name.to_owned(),
        kind,
        indexed,
    })
}

fn parse_function(fn_string: &str) -> Result<Function, ParseError> {
    let fn_string = fn_string.to_owned();
    let delim = if fn_string.starts_with("function ") {
        "function "
    } else {
        " "
    };
    let split: Vec<&str> = fn_string.split(delim).collect();
    let split: Vec<&str> = split[1].split('(').collect();

    // function name is the first char
    let fn_name = split[0];

    // internal args
    let args: Vec<&str> = split[1].split(')').collect();
    let args: Vec<&str> = args[0].split(", ").collect();

    let inputs = args
        .into_iter()
        .filter(|x| !x.is_empty())
        .filter(|x| !x.contains("returns"))
        .map(|x| parse_param(x))
        .collect::<Result<Vec<Param>, _>>()?;

    // return value
    let outputs: Vec<Param> = if split.len() > 2 {
        let ret = split[2].strip_suffix(")").expect("no right paren");
        let ret: Vec<&str> = ret.split(", ").collect();

        ret.into_iter()
            // remove modifiers etc
            .filter(|x| !x.is_empty())
            .map(|x| parse_param(x))
            .collect::<Result<Vec<Param>, _>>()?
    } else {
        vec![]
    };

    #[allow(deprecated)]
        Ok(Function {
        name: fn_name.to_owned(),
        inputs,
        outputs,
        // this doesn't really matter
        state_mutability: StateMutability::NonPayable,
        constant: false,
    })
}

fn parse_param2(param: &str) -> Result<Param, ParseError> {
    let mut iter = param.trim().rsplitn(3, is_whitespace);

    let name = iter.next().ok_or(ParseError::ParseError(super::Error::InvalidData))?;

    if let Some(ty) = iter.last() {
        if name == "memory" || name == "calldata" {
            Ok(Param { name: "".to_owned(), kind: Reader::read(ty)? })
        } else {
            Ok(Param { name: name.to_owned(), kind: Reader::read(ty)? })
        }
    } else {
        Ok(Param { name: "".to_owned(), kind: Reader::read(name)? })
    }
}

// address x
fn parse_param(param: &str) -> Result<Param, ParseError> {
    let mut param = param
        .split(' ')
        .filter(|x| !x.contains("memory") || !x.contains("calldata"));

    let kind = param.next().ok_or(ParseError::Kind)?;
    let kind: ParamType = Reader::read(kind).unwrap();

    // strip memory/calldata from the name
    // e.g. uint256[] memory x
    let mut name = param.next().unwrap_or_default();
    if name == "memory" || name == "calldata" {
        name = param.next().unwrap_or_default();
    }

    Ok(Param {
        name: name.to_owned(),
        kind,
    })
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

    #[test]
    fn parses_approve2() {
        let fn_str = "function approve(address _spender, uint256 value) external returns(bool)";
        let parsed = parse_function2(fn_str).unwrap();
        assert_eq!(parsed.name, "approve");
        assert_eq!(parsed.inputs[0].name, "_spender");
        assert_eq!(parsed.inputs[0].kind, ParamType::Address, );
        assert_eq!(parsed.inputs[1].name, "value");
        assert_eq!(parsed.inputs[1].kind, ParamType::Uint(256), );
        assert_eq!(parsed.outputs[0].name, "");
        assert_eq!(parsed.outputs[0].kind, ParamType::Bool);
    }

    #[test]
    fn parses_approve() {
        let fn_str = "function approve(address _spender, uint256 value) external returns(bool)";
        let parsed = parse_function(fn_str).unwrap();
        assert_eq!(parsed.name, "approve");
        assert_eq!(parsed.inputs[0].name, "_spender");
        assert_eq!(parsed.inputs[0].kind, ParamType::Address, );
        assert_eq!(parsed.inputs[1].name, "value");
        assert_eq!(parsed.inputs[1].kind, ParamType::Uint(256), );
        assert_eq!(parsed.outputs[0].name, "");
        assert_eq!(parsed.outputs[0].kind, ParamType::Bool);
    }

    #[test]
    fn parses_function_arguments_return2() {
        let fn_str = "function foo(uint32[] memory x) external view returns (address)";
        let parsed = parse_function2(fn_str).unwrap();
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
    fn parses_function_empty2() {
        let fn_str = "function foo()";
        let parsed = parse_function2(fn_str).unwrap();
        assert_eq!(parsed.name, "foo");
        assert!(parsed.inputs.is_empty());
        assert!(parsed.outputs.is_empty());
    }

    #[test]
    fn parses_function_payable() {
        let fn_str = "function foo() public payable";
        let parsed = parse_function2(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::Payable);
    }

    #[test]
    fn parses_function_view() {
        let fn_str = "function foo() external view";
        let parsed = parse_function2(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::View);
    }

    #[test]
    fn parses_function_pure() {
        let fn_str = "function foo()  pure";
        let parsed = parse_function2(fn_str).unwrap();
        assert_eq!(parsed.state_mutability, StateMutability::Pure);
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
    fn parses_event2() {
        assert_eq!(
            parse_event2(&mut "event Foo (address indexed x, uint y, bytes32[] z)").unwrap(),
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
    fn parses_event() {
        assert_eq!(
            parse_event("event Foo (address indexed x, uint y, bytes32[] z)").unwrap(),
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
    fn parses_anonymous_event2() {
        assert_eq!(
            parse_event2(&mut "event Foo() anonymous").unwrap(),
            Event {
                anonymous: true,
                name: "Foo".to_owned(),
                inputs: vec![],
            }
        );
    }

    #[test]
    fn parses_anonymous_event() {
        assert_eq!(
            parse_event("event Foo() anonymous").unwrap(),
            Event {
                anonymous: true,
                name: "Foo".to_owned(),
                inputs: vec![],
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
    fn can_parse_functions2() {
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
                parse_function2(x).unwrap();
            });
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
            "\"function getValue() external view (string)\"",
        ])
            .unwrap();
    }
}
