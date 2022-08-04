/// Macro that expands to a struct with common AST node fields.
macro_rules! ast_node {
    (
        $(#[$struct_meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $ty:ty
            ),* $(,)*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $name {
            pub id: usize,
            #[serde(with = "serde_helpers::display_from_str")]
            pub src: SourceLocation,
            $(
                $(#[$field_meta])*
                pub $field: $ty
            ),*
        }
    };
}

/// A macro that expands to a struct with common expression node fields.
macro_rules! expr_node {
    (
        $(#[$struct_meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $ty:ty
            ),* $(,)*
        }
    ) => {
        ast_node!(
            $(#[$struct_meta])*
            struct $name {
                argument_types: Vec<TypeDescriptions>,
                is_constant: bool,
                is_l_value: bool,
                is_pure: bool,
                l_value_requested: bool,
                type_descriptions: TypeDescriptions,
                $(
                    $(#[$field_meta])*
                    $field: $ty
                ),*
            }
        );
    }
}

pub(crate) use ast_node;
pub(crate) use expr_node;
