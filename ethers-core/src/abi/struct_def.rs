//! Solidity struct definition parsing support
use crate::abi::error::{bail, format_err, Result};
use crate::abi::human_readable::{is_whitespace, parse_identifier};
use crate::abi::{param_type::Reader, ParamType};

/// A field declaration inside a struct
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDeclaration {
    name: String,
    ty: FieldType,
}

impl FieldDeclaration {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn r#type(&self) -> &FieldType {
        &self.ty
    }
}

/// A field declaration inside a struct
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// Represents elementary types, see [`ParamType`]
    Elementary(ParamType),
    /// A non elementary type field, treated as user defined struct
    Struct(StructFieldType),
    // Array of user defined type
    StructArray(StructFieldType),
    // Array with fixed size of user defined type
    FixedStructArray(StructFieldType, usize),
    /// Mapping
    Mapping(Box<MappingType>),
}

impl FieldType {
    pub fn is_mapping(&self) -> bool {
        matches!(self, FieldType::Mapping(_))
    }

    pub(crate) fn as_struct(&self) -> Option<&StructFieldType> {
        match self {
            FieldType::Struct(s)
            | FieldType::StructArray(s)
            | FieldType::FixedStructArray(s, _) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappingType {
    /// key types can be elementary and `bytes` and `string`
    ///
    /// Valid `ParamType` variants are:
    ///     `Address`, `Bytes`, `Int`, `UInt`, `Bool`, `String`, `FixedBytes`,
    key_type: ParamType,
    /// The value type of this mapping
    value_type: FieldType,
}

/// Represents a elementary field declaration inside a struct with a : `int x`
#[derive(Debug, Clone, PartialEq)]
pub struct StructFieldDeclaration {
    /// The name of the field
    name: String,
    /// The type of the field
    ty: StructFieldType,
}

/// How the type of a struct field is referenced
#[derive(Debug, Clone, PartialEq)]
pub struct StructFieldType {
    /// The name of the struct
    name: String,
    /// All previous projections up until the name
    ///
    /// For `MostOuter.Outer.<name>` this is `vec!["MostOuter", "Outer"]`
    projections: Vec<String>,
}

impl StructFieldType {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Parse a struct field declaration
    ///
    /// The parsed field is either a `Struct`, `StructArray` or `FixedStructArray`
    pub fn parse(mut input: &str) -> Result<FieldType> {
        let mut projections = Vec::new();

        loop {
            let ty = parse_identifier(&mut input)?;
            let mut chars = input.chars();
            match chars.next() {
                None => {
                    return Ok(FieldType::Struct(StructFieldType {
                        name: ty,
                        projections,
                    }))
                }
                Some(' ') | Some('\t') | Some('[') => {
                    // array
                    let mut size = String::new();
                    loop {
                        match chars.next() {
                            None => bail!("Expected Array `{}`", input),
                            Some(' ') | Some('\t') => {
                                if !size.is_empty() {
                                    bail!(
                                        "Illegal whitespace in array size after `{}` in `{}`",
                                        size,
                                        input
                                    )
                                }
                            }
                            Some(']') => {
                                let ty = StructFieldType {
                                    name: ty,
                                    projections,
                                };

                                return if size.is_empty() {
                                    Ok(FieldType::StructArray(ty))
                                } else {
                                    let size = size.parse().map_err(|_| {
                                        format_err!("Illegal array size `{}` at `{}`", size, input)
                                    })?;
                                    Ok(FieldType::FixedStructArray(ty, size))
                                };
                            }
                            Some(c) => {
                                if c.is_numeric() {
                                    size.push(c);
                                } else {
                                    bail!("Illegal char `{}` inner array `{}`", c, input)
                                }
                            }
                        }
                    }
                }
                Some('.') => {
                    input = chars.as_str();
                    projections.push(ty);
                }
                Some(c) => {
                    bail!("Illegal char `{}` at `{}`", c, input)
                }
            }
        }
    }
}

/// Represents a solidity struct
#[derive(Debug, Clone, PartialEq)]
pub struct SolStruct {
    name: String,
    fields: Vec<FieldDeclaration>,
}

impl SolStruct {
    /// Parse a solidity struct definition
    ///
    /// # Example
    ///
    /// ```
    /// # use ethers::abi::SolStruct;
    /// let s = SolStruct::parse("struct MyStruct { uint x; uint y;}").unwrap();
    /// ```
    pub fn parse(s: &str) -> Result<Self> {
        let mut input = s.trim();
        if !input.starts_with("struct ") {
            bail!("Not a struct `{}`", input)
        }
        input = &input[6..];

        let name = parse_identifier(&mut input)?;

        let mut chars = input.chars();

        loop {
            match chars.next() {
                None => bail!("Expected struct"),
                Some('{') => {
                    // strip opening and trailing curly bracket
                    input = chars
                        .as_str()
                        .trim()
                        .strip_suffix('}')
                        .ok_or_else(|| format_err!("Expected closing `}}` in `{}`", s))?
                        .trim_end();

                    let fields = if input.is_empty() {
                        Vec::new()
                    } else {
                        input
                            .split(';')
                            .filter(|s| !s.is_empty())
                            .map(parse_struct_field)
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    return Ok(SolStruct { name, fields });
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

    /// Name of this struct
    pub fn name(&self) -> &str {
        &self.name
    }

    /// All the fields of this struct
    pub fn fields(&self) -> &Vec<FieldDeclaration> {
        &self.fields
    }
}

/// Strips the identifier of field declaration from the input and returns it
fn strip_field_identifier(input: &mut &str) -> Result<String> {
    let mut iter = input.trim_end().rsplitn(2, is_whitespace);
    let name = iter
        .next()
        .ok_or_else(|| format_err!("Expected field identifier"))
        .map(|mut s| parse_identifier(&mut s))??;
    *input = iter
        .next()
        .ok_or_else(|| format_err!("Expected field type in `{}`", input))?
        .trim_end();
    Ok(name)
}

/// Parses a field definition such as `<type> <storageLocation>? <name>`
fn parse_struct_field(s: &str) -> Result<FieldDeclaration> {
    let mut input = s.trim_start();

    if !input.starts_with("mapping") {
        // strip potential defaults
        input = input
            .split('=')
            .next()
            .ok_or_else(|| format_err!("Expected field definition `{}`", s))?
            .trim_end();
    }
    let name = strip_field_identifier(&mut input)?;
    Ok(FieldDeclaration {
        name,
        ty: parse_field_type(input)?,
    })
}

fn parse_field_type(s: &str) -> Result<FieldType> {
    let mut input = s.trim_start();
    if input.starts_with("mapping") {
        return Ok(FieldType::Mapping(Box::new(parse_mapping(input)?)));
    }
    if input.ends_with(" payable") {
        // special case for `address payable`
        input = input[..input.len() - 7].trim_end();
    }
    if let Ok(ty) = Reader::read(input) {
        Ok(FieldType::Elementary(ty))
    } else {
        // parsing elementary datatype failed, try struct
        StructFieldType::parse(input.trim_end())
    }
}

/// parse a mapping declaration
fn parse_mapping(s: &str) -> Result<MappingType> {
    let mut input = s.trim();
    if !input.starts_with("mapping") {
        bail!("Not a mapping `{}`", input)
    }
    input = input[7..].trim_start();
    let mut iter = input
        .trim_start_matches('(')
        .trim_end_matches(')')
        .splitn(2, "=>");
    let key_type = iter
        .next()
        .ok_or_else(|| format_err!("Expected mapping key type at `{}`", input))
        .map(str::trim)
        .map(Reader::read)??;

    if let ParamType::Array(_) | ParamType::FixedArray(_, _) | ParamType::Tuple(_) = &key_type {
        bail!(
            "Expected elementary mapping key type at `{}` got {:?}",
            input,
            key_type
        )
    }

    let value_type = iter
        .next()
        .ok_or_else(|| format_err!("Expected mapping value type at `{}`", input))
        .map(str::trim)
        .map(parse_field_type)??;

    Ok(MappingType {
        key_type,
        value_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_simple_struct() {
        assert_eq!(
            SolStruct::parse("struct MyStruct{uint256 x; uint256 y;}").unwrap(),
            SolStruct {
                name: "MyStruct".to_string(),
                fields: vec![
                    FieldDeclaration {
                        name: "x".to_string(),
                        ty: FieldType::Elementary(ParamType::Uint(256)),
                    },
                    FieldDeclaration {
                        name: "y".to_string(),
                        ty: FieldType::Elementary(ParamType::Uint(256)),
                    },
                ],
            }
        );
    }

    #[test]
    fn can_parse_struct() {
        assert_eq!(
            SolStruct::parse("struct MyStruct{uint256 x; uint256 y; bytes[] _b; string[10] s; mapping(address => uint256) m;}").unwrap(),
            SolStruct {
                name: "MyStruct".to_string(),
                fields: vec![
                    FieldDeclaration {
                        name: "x".to_string(),
                        ty: FieldType::Elementary(ParamType::Uint(256)),
                    },
                    FieldDeclaration {
                        name: "y".to_string(),
                        ty: FieldType::Elementary(ParamType::Uint(256)),
                    },
                    FieldDeclaration {
                        name: "_b".to_string(),
                        ty: FieldType::Elementary(ParamType::Array(Box::new(ParamType::Bytes))),
                    },
                    FieldDeclaration {
                        name: "s".to_string(),
                        ty: FieldType::Elementary(ParamType::FixedArray(Box::new(ParamType::String), 10)),
                    },
                    FieldDeclaration {
                        name: "m".to_string(),
                        ty: FieldType::Mapping(Box::new(
                            MappingType {
                                key_type: ParamType::Address,
                                value_type: FieldType::Elementary(ParamType::Uint(256))
                            }
                        )),
                    },
                ],
            }
        );
    }

    #[test]
    fn can_parse_struct_projections() {
        assert_eq!(
            SolStruct::parse("struct MyStruct{uint256 x; Some.Other.Inner _other;}").unwrap(),
            SolStruct {
                name: "MyStruct".to_string(),
                fields: vec![
                    FieldDeclaration {
                        name: "x".to_string(),
                        ty: FieldType::Elementary(ParamType::Uint(256)),
                    },
                    FieldDeclaration {
                        name: "_other".to_string(),
                        ty: FieldType::Struct(StructFieldType {
                            name: "Inner".to_string(),
                            projections: vec!["Some".to_string(), "Other".to_string()]
                        }),
                    },
                ],
            }
        );
    }

    #[test]
    fn can_parse_structs() {
        [
            "struct Demo {bytes  x; address payable d;}",
            "struct Demo2 {bytes[10]  x; mapping(bool=> bool) d; int256 value;}",
            "struct Struct { Other.MyStruct s;  bool voted;  address delegate; uint vote; }",
        ]
        .iter()
        .for_each(|s| {
            SolStruct::parse(s).unwrap();
        });
    }

    #[test]
    fn can_parse_mapping_type() {
        assert_eq!(
            parse_mapping("mapping(string=> string)").unwrap(),
            MappingType {
                key_type: ParamType::String,
                value_type: FieldType::Elementary(ParamType::String)
            }
        );
    }

    #[test]
    fn can_parse_nested_mappings() {
        assert_eq!(
            parse_mapping("mapping(string=> mapping(string=> string))").unwrap(),
            MappingType {
                key_type: ParamType::String,
                value_type: FieldType::Mapping(Box::new(MappingType {
                    key_type: ParamType::String,
                    value_type: FieldType::Elementary(ParamType::String),
                })),
            }
        );
    }

    #[test]
    fn can_detect_illegal_mappings_key_type() {
        [
            "mapping(string[]=> mapping(string=> string))",
            "mapping(bytes[10] => bool)",
            "mapping(uint256[10] => bool)",
            "mapping(Item=> bool)",
            "mapping(Item[]=> mapping(address  => bool))",
        ]
        .iter()
        .for_each(|s| {
            assert!(parse_mapping(s).is_err());
        });
    }

    #[test]
    fn can_parse_mappings() {
        [
            "mapping(string=> mapping(string=> string))",
            "mapping(string=> mapping(string=> mapping(string=> mapping(string=> string))))",
            "mapping(bool=> bool)",
            "mapping(bytes32 => bool)",
            "mapping(bytes=> bool)",
            "mapping(uint256=> mapping(address  => bool))",
        ]
        .iter()
        .for_each(|s| {
            parse_mapping(s).unwrap();
        });
    }

    #[test]
    fn can_strip_field_ident() {
        let mut s = "uint256 _myvar,
                    ";
        let name = strip_field_identifier(&mut s).unwrap();
        assert_eq!("_myvar", name);
        assert_eq!("uint256", s);
    }
}
