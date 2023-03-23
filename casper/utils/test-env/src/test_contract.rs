use std::{path::PathBuf, fmt::Debug, rc::Rc};

use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, CLTyped, ContractHash, RuntimeArgs,
};

use crate::{utils::DeploySource, TestEnv};

pub struct TestContract {
    env: TestEnv,
    name: String,
    contract_owner: AccountHash,
}

impl TestContract {
    pub fn new(
        env: &TestEnv,
        wasm: &str,
        name: &str,
        sender: AccountHash,
        mut args: RuntimeArgs,
    ) -> TestContract {
        let session_code = PathBuf::from(wasm);
        args.insert("contract_name", name).unwrap();
        env.run(sender, DeploySource::Code(session_code), args);

        TestContract {
            env: env.clone(),
            name: String::from(name),
            contract_owner: sender,
        }
    }

    pub fn query_dictionary<T: CLTyped + FromBytes>(
        &self,
        dict_name: &str,
        key: String,
    ) -> Option<T> {
        self.env
            .query_dictionary(self.contract_hash(), dict_name, key)
    }

    pub fn query_named_key<T: CLTyped + FromBytes + Debug>(&self, key: String) -> T {
        let contract_name = format!("{}_contract_hash", self.name);
        self.env
            .query_account_named_key(self.contract_owner, &[contract_name, key])
    }

    pub fn contract_hash(&self) -> [u8; 32] {
        let key = format!("{}_contract_hash_wrapped", self.name);
        self.env
            .query_account_named_key(self.contract_owner, &[key])
    }

    pub fn call_contract(&self, sender: AccountHash, entry_point: &str, session_args: RuntimeArgs) {
        println!("\n\n\n\n\n\n\n BEGIN");
        println!("vvv-test-contract::call_contract(name) {:?}", format!("{}_contract_hash_wrapped", self.name));
        println!("vvv-test-contract::call_contract(contract_hash) {:?}", self.contract_hash());
        println!("vvv-test-contract::call_contract {:?}", ContractHash::new(self.contract_hash()));
        println!("END \n\n\n\n\n\n\n");

        let session_code = DeploySource::ByHash {
            hash: ContractHash::new(self.contract_hash()),
            method: entry_point.to_string(),
        };
        self.env.run(sender, session_code, session_args);
    }
}
