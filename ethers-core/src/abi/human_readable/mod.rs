use ethabi::AbiError;
use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::abi::{
    error::{bail, format_err, ParseError, Result},
    struct_def::{FieldType, StructFieldType},
    Abi, Constructor, Event, EventParam, Function, HumanReadableParser, Param, ParamType,
    SolStruct, StateMutability,
};
pub mod lexer;

/// A parser that turns a "human readable abi" into a `Abi`
pub struct AbiParser {
    /// solidity structs
    pub structs: HashMap<String, SolStruct>,
    /// solidity structs as tuples
    pub struct_tuples: HashMap<String, Vec<ParamType>>,
    /// (function name, param name) -> struct which are the identifying properties we get the name
    /// from ethabi.
    pub function_params: HashMap<(String, String), String>,
    /// (event name, idx) -> struct which are the identifying properties we get the name
    /// from ethabi.
    ///
    /// Note: we need to map the index of the event here because events can contain nameless inputs
    pub event_params: HashMap<(String, usize), String>,
    /// (function name) -> `Vec<structs>` all structs the function returns
    pub outputs: HashMap<String, Vec<String>>,
}

impl AbiParser {
    /// Parses a "human readable abi" string
    ///
    /// # Example
    ///
    /// ```
    ///  # use ethers_core::abi::AbiParser;
    /// let abi = AbiParser::default().parse_str(r#"[
    ///         function setValue(string)
    ///         function getValue() external view returns (string)
    ///         event ValueChanged(address indexed author, string oldValue, string newValue)
    ///     ]"#).unwrap();
    /// ```
    pub fn parse_str(&mut self, s: &str) -> Result<Abi> {
        self.parse(
            &s.trim().trim_start_matches('[').trim_end_matches(']').lines().collect::<Vec<_>>(),
        )
    }

    /// Parses a "human readable abi" string vector
    ///
    /// # Example
    /// ```
    /// use ethers_core::abi::AbiParser;
    ///
    /// let abi = AbiParser::default().parse(&[
    ///     "function x() external view returns (uint256)",
    /// ]).unwrap();
    /// ```
    pub fn parse(&mut self, input: &[&str]) -> Result<Abi> {
        // parse struct first
        let mut abi = Abi {
            constructor: None,
            functions: BTreeMap::new(),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
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
            if line.starts_with("event") {
                let event = self.parse_event(line)?;
                abi.events.entry(event.name.clone()).or_default().push(event);
            } else if let Some(err) = line.strip_prefix("error") {
                // an error is essentially a function without outputs, so we parse as function here
                let function = match self.parse_function(err) {
                    Ok(function) => function,
                    Err(_) => bail!("Illegal abi `{}`, expected error", line),
                };
                if !function.outputs.is_empty() {
                    bail!("Illegal abi `{}`, expected error", line);
                }
                let error = AbiError { name: function.name, inputs: function.inputs };
                abi.errors.entry(error.name.clone()).or_default().push(error);
            } else if line.starts_with("constructor") {
                let inputs = self
                    .constructor_inputs(line)?
                    .into_iter()
                    .map(|(input, struct_name)| {
                        if let Some(struct_name) = struct_name {
                            // keep track of the user defined struct of that param
                            self.function_params.insert(
                                ("constructor".to_string(), input.name.clone()),
                                struct_name,
                            );
                        }
                        input
                    })
                    .collect();

                abi.constructor = Some(Constructor { inputs });
            } else {
                // function may have shorthand declaration, so it won't start with "function"
                let function = match self.parse_function(line) {
                    Ok(function) => function,
                    Err(_) => bail!("Illegal abi `{}`, expected function", line),
                };
                abi.functions.entry(function.name.clone()).or_default().push(function);
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
                            tuple.push(ty.as_param(ParamType::Tuple(param)))
                        } else {
                            resolved = false;
                            break
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
            structs: structs.into_iter().map(|s| (s.name().to_string(), s)).collect(),
            struct_tuples: HashMap::new(),
            function_params: Default::default(),
            event_params: Default::default(),
            outputs: Default::default(),
        }
    }

    /// Parses a solidity event declaration from `event <name> (args*) anonymous?`
    pub fn parse_event(&mut self, s: &str) -> Result<Event> {
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
                            .into_iter()
                            .enumerate()
                            .map(|(idx, (input, struct_name))| {
                                if let Some(struct_name) = struct_name {
                                    // keep track of the user defined struct of that param
                                    self.event_params.insert((name.clone(), idx), struct_name);
                                }
                                input
                            })
                            .collect()
                    };

                    let event = Event { name, inputs, anonymous };
                    return Ok(event)
                }
                Some(' ') | Some('\t') => continue,
                Some(c) => {
                    bail!("Illegal char `{}` at `{}`", c, s)
                }
            }
        }
    }

    /// Parse a single event param
    ///
    /// See [`Self::parse_type`]
    fn parse_event_arg(&self, input: &str) -> Result<(EventParam, Option<String>)> {
        let mut iter = input.trim().rsplitn(3, is_whitespace);
        let mut indexed = false;
        let mut name =
            iter.next().ok_or_else(|| format_err!("Empty event param at `{}`", input))?;

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

        let (kind, user_ty) = self.parse_type(type_str)?;
        Ok((EventParam { name: name.to_string(), indexed, kind }, user_ty))
    }

    /// Returns the parsed function from the input string
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_core::abi::AbiParser;
    /// let f = AbiParser::default()
    ///     .parse_function("bar(uint256 x, uint256 y, address addr)").unwrap();
    /// ```
    pub fn parse_function(&mut self, s: &str) -> Result<Function> {
        let mut input = s.trim();
        let shorthand = !input.starts_with("function ");

        if !shorthand {
            input = &input[8..];
        }

        let name = parse_identifier(&mut input)?;
        input = input
            .strip_prefix('(')
            .ok_or_else(|| format_err!("Expected input args parentheses at `{}`", s))?;

        let (input_args_modifiers, output_args) = match input.rsplit_once('(') {
            Some((first, second)) => (first, Some(second)),
            None => (input, None),
        };

        let mut input_args_modifiers_iter = input_args_modifiers
            .trim_end()
            .strip_suffix(" returns")
            .unwrap_or(input_args_modifiers)
            .splitn(2, ')');

        let input_args = match input_args_modifiers_iter
            .next()
            .ok_or_else(|| format_err!("Expected input args parentheses at `{}`", s))?
        {
            "" => None,
            input_params_args => Some(input_params_args),
        };
        let modifiers = match input_args_modifiers_iter
            .next()
            .ok_or_else(|| format_err!("Expected input args parentheses at `{}`", s))?
        {
            "" => None,
            modifiers => Some(modifiers),
        };

        let inputs = if let Some(params) = input_args {
            self.parse_params(params)?
                .into_iter()
                .map(|(input, struct_name)| {
                    if let Some(struct_name) = struct_name {
                        // keep track of the user defined struct of that param
                        self.function_params
                            .insert((name.clone(), input.name.clone()), struct_name);
                    }
                    input
                })
                .collect()
        } else {
            Vec::new()
        };

        let outputs = if let Some(params) = output_args {
            let params = params
                .trim()
                .strip_suffix(')')
                .ok_or_else(|| format_err!("Expected output args parentheses at `{}`", s))?;
            let output_params = self.parse_params(params)?;
            let mut outputs = Vec::with_capacity(output_params.len());
            let mut output_types = Vec::new();

            for (output, struct_name) in output_params {
                if let Some(struct_name) = struct_name {
                    // keep track of the user defined struct of that param
                    output_types.push(struct_name);
                }
                outputs.push(output);
            }
            self.outputs.insert(name.clone(), output_types);
            outputs
        } else {
            Vec::new()
        };

        let state_mutability = modifiers.map(detect_state_mutability).unwrap_or_default();

        Ok(
            #[allow(deprecated)]
            Function { name, inputs, outputs, state_mutability, constant: None },
        )
    }

    fn parse_params(&self, s: &str) -> Result<Vec<(Param, Option<String>)>> {
        s.split(',')
            .filter(|s| !s.is_empty())
            .map(|s| self.parse_param(s))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Returns the `ethabi` `ParamType` for the function parameter and the aliased struct type, if
    /// it is a user defined struct
    ///
    /// **NOTE**: the  `ethabi` Reader treats unknown identifiers as `UInt(8)`, because solc uses
    /// the _name_ of a solidity enum for the value of the `type` of the ABI, but only in sol
    /// libraries. If the enum is defined in a contract the value of the `type` is `uint8`
    ///
    /// # Example ABI for an enum in a __contract__
    /// ```json
    /// {
    ///   "internalType": "enum ContractTest.TestEnum",
    ///   "name": "test",
    ///   "type": "uint8"
    /// }
    /// ```
    ///
    /// # Example ABI for an enum in a __library__
    /// ```json
    /// {
    ///   "internalType": "enum ContractTest.TestEnum",
    ///   "name": "test",
    ///   "type": "ContractTest.TestEnum"
    /// }
    /// ```
    ///
    /// See <https://github.com/rust-ethereum/ethabi/issues/254>
    ///
    /// Therefore, we need to double-check if the `ethabi::Reader` parsed an `uint8`, and ignore the
    /// type if `type_str` is not uint8. However can lead to some problems if a function param is
    /// array of custom types for example, like `Foo[]`, which the `Reader` would identify as
    /// `uint8[]`. Therefor if the `Reader` returns an `uint8` we also check that the input string
    /// contains a `uint8`. This however can still lead to false detection of `uint8` and is only
    /// solvable with a more sophisticated parser: <https://github.com/gakonst/ethers-rs/issues/474>
    fn parse_type(&self, type_str: &str) -> Result<(ParamType, Option<String>)> {
        if let Ok(kind) = HumanReadableParser::parse_type(type_str) {
            Ok((kind, None))
        } else {
            // try struct instead
            self.parse_struct_type(type_str)
        }
    }

    /// Attempts to parse the `type_str` as a `struct`, resolving all fields of the struct into a
    /// `ParamType::Tuple`
    fn parse_struct_type(&self, type_str: &str) -> Result<(ParamType, Option<String>)> {
        if let Ok(field) = StructFieldType::parse(type_str) {
            let struct_ty = field
                .as_struct()
                .ok_or_else(|| format_err!("Expected struct type `{}`", type_str))?;
            let name = struct_ty.name();
            let tuple = self
                .struct_tuples
                .get(name)
                .cloned()
                .map(ParamType::Tuple)
                .ok_or_else(|| format_err!("Unknown struct `{}`", struct_ty.name()))?;

            if let Some(field) = field.as_struct() {
                Ok((field.as_param(tuple), Some(name.to_string())))
            } else {
                bail!("Expected struct type")
            }
        } else {
            bail!("Failed determine event type `{}`", type_str)
        }
    }

    pub fn parse_constructor(&self, s: &str) -> Result<Constructor> {
        let inputs = self.constructor_inputs(s)?.into_iter().map(|s| s.0).collect();
        Ok(Constructor { inputs })
    }

    fn constructor_inputs(&self, s: &str) -> Result<Vec<(Param, Option<String>)>> {
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

        self.parse_params(params)
    }

    fn parse_param(&self, param: &str) -> Result<(Param, Option<String>)> {
        let mut iter = param.trim().rsplitn(3, is_whitespace);

        let mut name =
            iter.next().ok_or_else(|| ParseError::ParseError(super::Error::InvalidData))?;

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
        let (kind, user_struct) = self.parse_type(type_str)?;
        Ok((Param { name: name.to_string(), kind, internal_type: None }, user_struct))
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
/// use ethers_core::abi::parse_abi;
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
    let c = chars.next().ok_or_else(|| format_err!("Empty identifier in `{}`", input))?;
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
        return Err(ParseError::ParseError(super::Error::InvalidName(input.to_string())))
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
        assert_eq!(parsed.inputs[0].kind, ParamType::Array(Box::new(ParamType::Uint(32))));
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
                    EventParam { name: "x".to_string(), kind: ParamType::Address, indexed: true },
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
            AbiParser::default().parse_event("event Foo() anonymous").unwrap(),
            Event { anonymous: true, name: "Foo".to_string(), inputs: vec![] }
        );
    }

    #[test]
    fn parses_unnamed_event() {
        assert_eq!(
            AbiParser::default().parse_event("event Foo(address)").unwrap(),
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
            AbiParser::default().parse_event("event Foo(address indexed)").unwrap(),
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
            AbiParser::default().parse_event_arg("address indexed x").unwrap().0,
            EventParam { name: "x".to_string(), kind: ParamType::Address, indexed: true }
        );

        assert_eq!(
            AbiParser::default().parse_event_arg("address x").unwrap().0,
            EventParam { name: "x".to_string(), kind: ParamType::Address, indexed: false }
        );
    }

    #[test]
    fn can_parse_functions() {
        [
            "function foo(uint256[] memory x) external view returns (address)",
            "function bar(uint256[] memory x) returns(address)",
            "function bar(uint256[] memory x, uint32 y) returns (address, uint256)",
            "function foo(address[] memory, bytes memory) returns (bytes memory)",
            "function bar(uint256[] memory x)",
            "function bar()",
            "bar(uint256[] memory x)(address)",
            "bar(uint256[] memory x, uint32 y)(address, uint256)",
            "foo(address[] memory, bytes memory)(bytes memory)",
            "bar(uint256[] memory x)()",
            "bar()()",
            "bar(uint256)",
            "bar()",
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
            "foo(uint256[] memory x)()",
            "call(Voter memory voter)(address, uint256)",
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
                        Box::new(ParamType::Tuple(vec![ParamType::Int(256), ParamType::Address])),
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
