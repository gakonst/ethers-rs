/// Asserts the result of an expansion matches source output.
///
/// # Panics
///
/// If the expanded source does not match the quoted source.
macro_rules! assert_quote {
    ($ex:expr, { $($t:tt)* } $(,)?) => {
        assert_eq!(
            $ex.to_string(),
            quote::quote! { $($t)* }.to_string(),
        )
    };
}
