use ethers_core::{
    abi::{Detokenize, InvalidOutputType, Token},
    types::Address,
};

// Note: We also provide the `abigen` macro for generating these bindings automatically
#[derive(Clone, Debug)]
pub struct ValueChanged {
    pub old_author: Address,
    pub new_author: Address,
    pub old_value: String,
    pub new_value: String,
}

impl Detokenize for ValueChanged {
    fn from_tokens(tokens: Vec<Token>) -> Result<ValueChanged, InvalidOutputType> {
        let old_author: Address = tokens[1].clone().to_address().unwrap();
        let new_author: Address = tokens[1].clone().to_address().unwrap();
        let old_value = tokens[2].clone().to_string().unwrap();
        let new_value = tokens[3].clone().to_string().unwrap();

        Ok(Self {
            old_author,
            new_author,
            old_value,
            new_value,
        })
    }
}
