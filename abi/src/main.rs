use elrond_wasm_debug::*;
use e_non_fungible_tokens::*;

fn main() {
    let contract = ENonFungibleTokensImpl::new(TxContext::dummy());
    print!("{}", abi_json::contract_abi(&contract));
}