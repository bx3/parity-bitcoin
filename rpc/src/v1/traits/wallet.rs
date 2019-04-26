use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use v1::types::H160;

build_rpc_trait! {
    /// Parity-bitcoin network interface
    pub trait Wallet {
        /// Add/remove/connect to the node
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "addnode", "params": ["127.0.0.1:8888", "add"], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "addnode", "params": ["127.0.0.1:8888", "remove"], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "addnode", "params": ["127.0.0.1:8888", "onetry"], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        #[rpc(name = "getbalance")]
        fn get_balance(&self) -> Result<(), Error>;

        #[rpc(name = "shardpay")]
        fn shard_pay(&self, H160, u64) -> Result<(), Error>;

        #[rpc(name = "getspendable")]
        fn get_spendable(&self) -> Result<(), Error>;

    }
}
