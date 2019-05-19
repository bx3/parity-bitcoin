use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use v1::types::H256 as H256_ser;
use v1::types::H160 as AddressHash_ser;

build_rpc_trait! {
    /// Parity-bitcoin network interface
    pub trait Wallet {
        /// Add/remove/connect to the node
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "addnode", "params": ["127.0.0.1:8888", "add"], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "addnode", "params": ["127.0.0.1:8888", "remove"], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "addnode", "params": ["127.0.0.1:8888", "onetry"], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        #[rpc(name = "generatekeypair")]
        fn generate_keypair(&self) -> Result<AddressHash_ser, Error>;

        #[rpc(name = "getbalance")]
        fn get_balance(&self) -> Result<u64, Error>;

        // index is 0, for outpoint
        #[rpc(name = "shardpay")]
        fn shard_pay(&self, AddressHash_ser, u64) -> Result<H256_ser, Error>;

        #[rpc(name = "updatewallet")]
        fn update_wallet(&self) -> Result<(), Error>;

        #[rpc(name = "walletaddtx")]
        fn wallet_add_tx(&self, H256_ser, u32) -> Result<(), Error>;

        #[rpc(name = "getaddresshash")]
        fn get_addresshash(&self) -> Result<AddressHash_ser, Error>;

        #[rpc(name = "printcoins")]
        fn print_coins(&self) -> Result<(), Error>;

        #[rpc(name = "covetpay")]
        fn covet_pay(&self, AddressHash_ser, u64) -> Result<H256_ser, Error>;

        #[rpc(name = "covetgeneratekeypair")]
        fn covet_generate_keypair(&self) -> Result<AddressHash_ser, Error>;

        #[rpc(name = "covetgetaddresshash")]
        fn covet_get_addresshash(&self) -> Result<AddressHash_ser, Error>;

        #[rpc(name = "covetwalletaddtx")]
        fn covet_wallet_add_tx(&self, H256_ser, u32) -> Result<(), Error>;
    }
}
