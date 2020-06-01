pub use erc20token_mod::ERC20Token;
mod erc20token_mod {
    use ethers_contract::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        providers::JsonRpcClient,
        signers::{Client, Signer},
        types::*,
        Contract, Lazy,
    };
    static ABI: Lazy<Abi> = Lazy::new(|| {
        serde_json :: from_str ( "[{\"constant\":true,\"inputs\":[],\"name\":\"name\",\"outputs\":[{\"name\":\"name\",\"type\":\"string\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"symbol\",\"outputs\":[{\"name\":\"symbol\",\"type\":\"string\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"decimals\",\"outputs\":[{\"name\":\"decimals\",\"type\":\"uint8\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"spender\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"approve\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"totalSupply\",\"outputs\":[{\"name\":\"totalSupply\",\"type\":\"uint256\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"from\",\"type\":\"address\"},{\"name\":\"to\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"transferFrom\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"who\",\"type\":\"address\"}],\"name\":\"balanceOf\",\"outputs\":[{\"name\":\"balance\",\"type\":\"uint256\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"to\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"transfer\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"payable\":false,\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"owner\",\"type\":\"address\"},{\"name\":\"spender\",\"type\":\"address\"}],\"name\":\"allowance\",\"outputs\":[{\"name\":\"remaining\",\"type\":\"uint256\"}],\"payable\":false,\"type\":\"function\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"owner\",\"type\":\"address\"},{\"indexed\":true,\"name\":\"spender\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"Approval\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"from\",\"type\":\"address\"},{\"indexed\":true,\"name\":\"to\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"Transfer\",\"type\":\"event\"}]" ) . expect ( "invalid abi" )
    });
    #[derive(Clone)]
    pub struct ERC20Token<'a, S, P>(Contract<'a, S, P>);
    impl<'a, S, P> std::ops::Deref for ERC20Token<'a, S, P> {
        type Target = Contract<'a, S, P>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<'a, S: Signer, P: JsonRpcClient> std::fmt::Debug for ERC20Token<'a, S, P> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ERC20Token))
                .field(&self.address())
                .finish()
        }
    }
    impl<'a, S: Signer, P: JsonRpcClient> ERC20Token<'a, S, P> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<Address>>(address: T, client: &'a Client<'a, S, P>) -> Self {
            let contract = Contract::new(client, ABI.clone(), address.into());
            Self(contract)
        }
        #[doc = "Calls the contract's balanceOf function"]
        pub fn balance_of(&self, who: Address) -> Sender<'a, S, P, U256> {
            self.0
                .method([112, 160, 130, 49], (who,))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's transfer function"]
        pub fn transfer(&self, to: Address, value: U256) -> Sender<'a, S, P, H256> {
            self.0
                .method([169, 5, 156, 187], (to, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's approve function"]
        pub fn approve(&self, spender: Address, value: U256) -> Sender<'a, S, P, H256> {
            self.0
                .method([9, 94, 167, 179], (spender, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's name function"]
        pub fn name(&self) -> Sender<'a, S, P, String> {
            self.0
                .method([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's decimals function"]
        pub fn decimals(&self) -> Sender<'a, S, P, u8> {
            self.0
                .method([49, 60, 229, 103], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's symbol function"]
        pub fn symbol(&self) -> Sender<'a, S, P, String> {
            self.0
                .method([149, 216, 155, 65], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's totalSupply function"]
        pub fn total_supply(&self) -> Sender<'a, S, P, U256> {
            self.0
                .method([24, 22, 13, 221], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's transferFrom function"]
        pub fn transfer_from(
            &self,
            from: Address,
            to: Address,
            value: U256,
        ) -> Sender<'a, S, P, H256> {
            self.0
                .method([35, 184, 114, 221], (from, to, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's allowance function"]
        pub fn allowance(&self, owner: Address, spender: Address) -> Sender<'a, S, P, U256> {
            self.0
                .method([221, 98, 237, 62], (owner, spender))
                .expect("method not found (this should never happen)")
        }
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct Transfer {
        pub from: Address,
        pub to: Address,
        pub value: U256,
    }
    impl Transfer {
        #[doc = r" Retrieves the signature for the event this data corresponds to."]
        #[doc = r" This signature is the Keccak-256 hash of the ABI signature of"]
        #[doc = r" this event."]
        pub const fn signature() -> H256 {
            H256([
                221, 242, 82, 173, 27, 226, 200, 155, 105, 194, 176, 104, 252, 55, 141, 170, 149,
                43, 167, 241, 99, 196, 161, 22, 40, 245, 90, 77, 245, 35, 179, 239,
            ])
        }
        #[doc = r" Retrieves the ABI signature for the event this data corresponds"]
        #[doc = r" to. For this event the value should always be:"]
        #[doc = r""]
        #[doc = "`Transfer(address,address,uint256)`"]
        pub const fn abi_signature() -> &'static str {
            "Transfer(address,address,uint256)"
        }
    }
    impl Detokenize for Transfer {
        fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
            if tokens.len() != 3 {
                return Err(InvalidOutputType(format!(
                    "Expected {} tokens, got {}: {:?}",
                    3,
                    tokens.len(),
                    tokens
                )));
            }
            #[allow(unused_mut)]
            let mut tokens = tokens.into_iter();
            let from = Address::from_token(tokens.next().expect("this should never happen"))?;
            let to = Address::from_token(tokens.next().expect("this should never happen"))?;
            let value = U256::from_token(tokens.next().expect("this should never happen"))?;
            Ok(Transfer { from, to, value })
        }
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct Approval {
        pub owner: Address,
        pub spender: Address,
        pub value: U256,
    }
    impl Approval {
        #[doc = r" Retrieves the signature for the event this data corresponds to."]
        #[doc = r" This signature is the Keccak-256 hash of the ABI signature of"]
        #[doc = r" this event."]
        pub const fn signature() -> H256 {
            H256([
                140, 91, 225, 229, 235, 236, 125, 91, 209, 79, 113, 66, 125, 30, 132, 243, 221, 3,
                20, 192, 247, 178, 41, 30, 91, 32, 10, 200, 199, 195, 185, 37,
            ])
        }
        #[doc = r" Retrieves the ABI signature for the event this data corresponds"]
        #[doc = r" to. For this event the value should always be:"]
        #[doc = r""]
        #[doc = "`Approval(address,address,uint256)`"]
        pub const fn abi_signature() -> &'static str {
            "Approval(address,address,uint256)"
        }
    }
    impl Detokenize for Approval {
        fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
            if tokens.len() != 3 {
                return Err(InvalidOutputType(format!(
                    "Expected {} tokens, got {}: {:?}",
                    3,
                    tokens.len(),
                    tokens
                )));
            }
            #[allow(unused_mut)]
            let mut tokens = tokens.into_iter();
            let owner = Address::from_token(tokens.next().expect("this should never happen"))?;
            let spender = Address::from_token(tokens.next().expect("this should never happen"))?;
            let value = U256::from_token(tokens.next().expect("this should never happen"))?;
            Ok(Approval {
                owner,
                spender,
                value,
            })
        }
    }
}
fn main() {}
