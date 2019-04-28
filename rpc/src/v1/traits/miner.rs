use jsonrpc_core::Error;

use v1::types::{BlockTemplate, BlockTemplateRequest};

use v1::types::H160;
use v1::types::H256;

build_rpc_trait! {
    /// Parity-bitcoin miner data interface.
    pub trait Miner {
        /// Get block template for mining.
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "getblocktemplate", "params": [{"capabilities": ["coinbasetxn", "workid", "coinbase/append"]}], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        #[rpc(name = "getblocktemplate")]
        fn get_block_template(&self, BlockTemplateRequest) -> Result<BlockTemplate, Error>;
        //generate
        #[rpc(name = "generateblocks")]
        fn generate_blocks(&self, H160, u32) -> Result<H256, Error>;

        #[rpc(name = "print_blocks")]
        fn print_blocks(&self) -> Result<(), Error>;    
    }
}
