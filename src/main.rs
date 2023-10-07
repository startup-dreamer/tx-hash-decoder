use std::{
    io::{self, Write},
    sync::Arc,
};
use hex::FromHex;
use ethers::{
    abi::AbiDecode,
    prelude::{abigen, k256::ecdsa::SigningKey, rand::thread_rng, Abigen, SignerMiddleware, *},
    providers::{Http, Provider},
};

pub fn address(address: &str) -> Address {
    address.parse::<Address>().unwrap()
}
abigen!(SmartContract, "src/abi.json");

pub fn bind(name: &str, abi: &str) {
    let name: String = format!("{}_rs_bndg", name);
    let bindings = Abigen::new(&name, abi).unwrap().generate().unwrap();
    let path: String = format!("src/{}.rs", name);
    match std::fs::File::create(path.clone()) {
        Ok(_) => {}
        Err(_) => {}
    }
    bindings.write_to_file(&path).unwrap();
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    network: String,
    provider: Provider<Http>,
    pub middleware: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
}

impl Config {
    pub fn new(network: &str, chain_id: &u64) -> Self {
        let provider: Provider<Http> = Provider::<Http>::try_from(network).unwrap();
        let wallet = LocalWallet::new(&mut thread_rng()).with_chain_id(*chain_id);
        let middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

        Self {
            network: network.to_string(),
            provider: provider,
            middleware: middleware,
        }
    }

    pub fn create_contract(&self, contract_address: &str, contract_name: &str) {
        bind(contract_name, "src/abi.json");
        let addr: H160 = address(contract_address);
        SmartContract::new(addr, Arc::clone(&self.middleware));
    }
}

#[tokio::main]
pub async fn transaction_data(tx_hash_str: &str, rpc: &str) -> Bytes {
    let bytes = Vec::from_hex(tx_hash_str.trim_start_matches("0x")).unwrap();
    let tx_bytes = &bytes[..32];
    let tx_hash = TxHash::from_slice(tx_bytes);
    let provider: Provider<Http> = Provider::<Http>::try_from(rpc).unwrap();
    let tx = provider.get_transaction(tx_hash).await.unwrap();

    match tx {
        Some(transaction) => transaction.input,
        None => panic!("Transaction not found"),
    }
}

pub fn input() -> String {
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn main() {
    // pub const contract_address: &str = "0x5f8456fc4f6a4dde545d50e3d51109b47c263252";
    // let contract_name = "contract";
    // let tx_hash_str = "0x706afa2546a132cb437cb7a186551558e92dda7d33d8c2605f8f6a5bc2cbdb1a";
    // let rpc = "https://polygon-rpc.com";
    // let chain_id = 137

    println!("================= Input Tx =================");
    // Prompt the user for the contract address
    print!("Enter contract address: ");
    let binding = input();
    let contract_address = binding.trim();

    // Prompt the user for the contract name
    print!("Enter contract name: ");
    let binding = input();
    let contract_name = binding.trim();

    // Prompt the user for the transaction hash
    print!("Enter transaction hash: ");
    let binding = input();
    let tx_hash_str = binding.trim();

    // // Prompt the user for the rpc url
    print!("Enter rpc url: ");
    let binding = input();
    let rpc = binding.trim();

    // Prompt the user for the transaction hash
    print!("Enter chain id: ");
    let binding = input();
    let chain_id = binding.parse().unwrap();

    println!("========== Creating Rust Bindings ==========");
    let config = Config::new(rpc, &chain_id);
    let contract_middleware = config.create_contract(contract_address, contract_name);
    let calldata = transaction_data(tx_hash_str, rpc);

    println!("================= Decoding =================");
    let decoded = SmartContractCalls::decode(&calldata).unwrap();
    println!("{:#?}", decoded);
}
