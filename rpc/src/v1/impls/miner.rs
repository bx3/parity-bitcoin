use jsonrpc_core::Error;
use miner;
use sync;
use v1::traits::Miner;
use v1::types::{BlockTemplate, BlockTemplateRequest};

use miner::Sh_CoinbaseTransactionBuilder;
use primitives::bigint::{Uint, U256};
use v1::types::H160;
use v1::types::H256;

use chain::Block;
use chain::BlockHeader;

use chain::IndexedBlock;
use std::{thread, time};
use global_script::Script;

//use chain::IndexedBlockHeader;

//use primitives::hash::H256 as p_H256;

pub struct MinerClient<T: MinerClientCoreApi> {
    core: T,
}

pub trait MinerClientCoreApi: Send + Sync + 'static {
    fn get_block_template(&self) -> miner::BlockTemplate;
    fn insert_block(&self, indexed_block: IndexedBlock);
    fn execute_broadcast_block(&self, indexed_block: IndexedBlock);
    fn print_blocks(&self);
}

pub struct MinerClientCore {
    pub local_sync_node: sync::LocalNodeRef,
}

impl MinerClientCore {
    pub fn new(local_sync_node: sync::LocalNodeRef) -> Self {
        MinerClientCore {
            local_sync_node: local_sync_node,
        }
    }
}

impl MinerClientCoreApi for MinerClientCore {
    fn get_block_template(&self) -> miner::BlockTemplate {
        self.local_sync_node.get_block_template()
    }

    fn insert_block(&self, indexed_block: IndexedBlock) {
        self.local_sync_node.on_block(0, indexed_block);
    }

    fn execute_broadcast_block(&self, indexed_block: IndexedBlock) {
        self.local_sync_node.unsolicited_block(0, indexed_block);
    }

    fn print_blocks(&self) {
        self.local_sync_node.print_blocks();
    }
}

impl<T> MinerClient<T>
where
    T: MinerClientCoreApi,
{
    pub fn new(core: T) -> Self {
        MinerClient { core: core }
    }
}

impl<T> Miner for MinerClient<T>
where
    T: MinerClientCoreApi,
{
    fn print_blocks(&self) -> Result<(), Error> {
        let wallet = self.core.print_blocks();
        Ok(())
    }

    fn get_block_template(&self, _request: BlockTemplateRequest) -> Result<BlockTemplate, Error> {
        Ok(self.core.get_block_template().into())
    }

    fn generate_blocks(&self, addrhash: H160, num_blocks: u32) -> Result<H256, Error> {
        let mut hash: primitives::hash::H160 = addrhash.clone().into();

        let mut coinbase_txid = H256::default();


        for _i in 0..num_blocks {
            let peer_index = 0;

            let coinbase_builder = Sh_CoinbaseTransactionBuilder::new(&hash, 10);
            let block_template = self.core.get_block_template(); //.into()

            //let mut retarget: primitives::compact::Compact = 0xfffffffffffffffffffffffff.into();
            //block_template.bits = retarget;

            let solution =
                miner::find_solution(&block_template, coinbase_builder, U256::max_value());

            let solution = solution.unwrap();


            let block_header = BlockHeader {
                version: block_template.version,
                previous_header_hash: block_template.previous_header_hash.clone(),
                merkle_root_hash: solution.coinbase_transaction.hash().clone(), //use coinbase transaction for
                time: solution.time.clone(),
                bits: block_template.bits.clone(),
                nonce: solution.nonce.clone(),
            };

            coinbase_txid = solution.coinbase_transaction.hash().clone().into();

            let block = Block::new(block_header.clone(), vec![solution.coinbase_transaction]);
            let indexed_block = IndexedBlock::from(block);

            // insert to local
            self.core.insert_block(indexed_block.clone());
            //broadcast
            self.core.execute_broadcast_block(indexed_block);

            let ten_millis = time::Duration::from_millis(100);
            thread::sleep(ten_millis);
        }

        Ok(coinbase_txid)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chain;
    use jsonrpc_core::IoHandler;
    use miner;
    use primitives::hash::H256;
    use v1::traits::Miner;

    #[derive(Default)]
    struct SuccessMinerClientCore;

    impl MinerClientCoreApi for SuccessMinerClientCore {
        fn get_block_template(&self) -> miner::BlockTemplate {
            let tx: chain::Transaction = "00000000013ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a0000000000000000000101000000000000000000000000".into();
            miner::BlockTemplate {
                version: 777,
                previous_header_hash: H256::from(1),
                time: 33,
                bits: 44.into(),
                height: 55,
                transactions: vec![tx.into()],
                coinbase_value: 66,
                size_limit: 77,
                sigop_limit: 88,
            }
        }

        fn insert_block(&self, indexed_block: IndexedBlock) {
            unimplemented!();
        }

        fn execute_broadcast_block(&self, indexed_block: IndexedBlock) {
            unimplemented!();
        }
    }

    #[test]
    fn getblocktemplate_accepted() {
        let client = MinerClient::new(SuccessMinerClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getblocktemplate",
				"params": [{}],
				"id": 1
			}"#),
            )
            .unwrap();

        // direct hash is 0100000000000000000000000000000000000000000000000000000000000000
        // but client expects reverse hash
        assert_eq!(&sample, r#"{"jsonrpc":"2.0","result":{"bits":44,"coinbaseaux":null,"coinbasetxn":null,"coinbasevalue":66,"curtime":33,"height":55,"mintime":null,"mutable":null,"noncerange":null,"previousblockhash":"0000000000000000000000000000000000000000000000000000000000000001","rules":null,"sigoplimit":88,"sizelimit":77,"target":"0000000000000000000000000000000000000000000000000000000000000000","transactions":[{"data":"00000000013ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a0000000000000000000101000000000000000000000000","depends":null,"fee":null,"hash":null,"required":false,"sigops":null,"txid":null,"weight":null}],"vbavailable":null,"vbrequired":null,"version":777,"weightlimit":null},"id":1}"#);
    }
}
