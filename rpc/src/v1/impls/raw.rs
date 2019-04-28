use chain::Transaction as GlobalTransaction;
use global_script::Script;
use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use keys::Address;
use network::Network;
use primitives::bytes::Bytes as GlobalBytes;
use primitives::hash::H256 as GlobalH256;
use ser::{deserialize, serialize, Reader, Serializable, SERIALIZE_TRANSACTION_WITNESS};
use storage;
use sync;
use v1::helpers::errors::{
    execution, invalid_params, transaction_not_found, transaction_of_side_branch,
};
use v1::traits::Raw;
use v1::types::H256;
use v1::types::{
    GetRawTransactionResponse, RawTransaction, SignedTransactionInput, SignedTransactionOutput,
    Transaction, TransactionInput, TransactionInputScript, TransactionOutput,
    TransactionOutputScript, TransactionOutputs,
};

use keys::generator::*;
use keys::Network as Key_Network;
use primitives::bytes::Bytes;

pub struct RawClient<T: RawClientCoreApi> {
    core: T,
}

pub trait RawClientCoreApi: Send + Sync + 'static {
    fn accept_transaction(&self, transaction: GlobalTransaction) -> Result<GlobalH256, String>;
    fn create_raw_transaction(
        &self,
        inputs: Vec<TransactionInput>,
        outputs: TransactionOutputs,
        lock_time: Trailing<u32>,
    ) -> Result<GlobalTransaction, String>;
    fn get_raw_transaction(
        &self,
        hash: GlobalH256,
        verbose: bool,
    ) -> Result<GetRawTransactionResponse, Error>;
}

pub struct RawClientCore {
    network: Network,
    local_sync_node: sync::LocalNodeRef,
    storage: storage::SharedStore,
}

impl RawClientCore {
    pub fn new(
        network: Network,
        local_sync_node: sync::LocalNodeRef,
        storage: storage::SharedStore,
    ) -> Self {
        RawClientCore {
            network,
            local_sync_node,
            storage,
        }
    }

    pub fn do_create_raw_transaction(
        inputs: Vec<TransactionInput>,
        outputs: TransactionOutputs,
        lock_time: Trailing<u32>,
    ) -> Result<GlobalTransaction, String> {
        use chain;
        use global_script::Builder as ScriptBuilder;
        use keys;

        // to make lock_time work at least one input must have sequnce < SEQUENCE_FINAL
        let lock_time = lock_time.unwrap_or_default();
        let default_sequence = if lock_time != 0 {
            chain::constants::SEQUENCE_FINAL - 1
        } else {
            chain::constants::SEQUENCE_FINAL
        };

        // prepare inputs
        let inputs: Vec<_> = inputs
            .into_iter()
            .map(|input| chain::TransactionInput {
                previous_output: chain::OutPoint {
                    hash: Into::<GlobalH256>::into(input.txid).reversed(),
                    index: input.vout,
                },
                script_sig: GlobalBytes::new(), // default script
                sequence: input.sequence.unwrap_or(default_sequence),
                script_witness: vec![],
            })
            .collect();

        // prepare outputs
        let outputs: Vec<_> = outputs
            .outputs
            .into_iter()
            .map(|output| match output {
                TransactionOutput::Address(with_address) => {
                    let amount_in_satoshis =
                        (with_address.amount * (chain::constants::SATOSHIS_IN_COIN as f64)) as u64;
                    let script = match with_address.address.kind {
                        keys::Type::P2PKH => ScriptBuilder::build_p2pkh(&with_address.address.hash),
                        keys::Type::P2SH => ScriptBuilder::build_p2sh(&with_address.address.hash),
                    };

                    chain::TransactionOutput {
                        value: amount_in_satoshis,
                        script_pubkey: script.to_bytes(),
                    }
                }
                TransactionOutput::ScriptData(with_script_data) => {
                    let script = ScriptBuilder::default()
                        .return_bytes(&*with_script_data.script_data)
                        .into_script();

                    chain::TransactionOutput {
                        value: 0,
                        script_pubkey: script.to_bytes(),
                    }
                }
            })
            .collect();

        // now construct && serialize transaction
        let transaction = GlobalTransaction {
            version: 1,
            inputs: inputs,
            outputs: outputs,
            lock_time: lock_time,
        };

        Ok(transaction)
    }
}

impl RawClientCoreApi for RawClientCore {
    fn accept_transaction(&self, transaction: GlobalTransaction) -> Result<GlobalH256, String> {
        self.local_sync_node.accept_transaction(transaction)
    }

    fn create_raw_transaction(
        &self,
        inputs: Vec<TransactionInput>,
        outputs: TransactionOutputs,
        lock_time: Trailing<u32>,
    ) -> Result<GlobalTransaction, String> {
        RawClientCore::do_create_raw_transaction(inputs, outputs, lock_time)
    }

    fn get_raw_transaction(
        &self,
        hash: GlobalH256,
        verbose: bool,
    ) -> Result<GetRawTransactionResponse, Error> {
        let transaction = match self.storage.transaction(&hash) {
            Some(transaction) => transaction,
            None => return Err(transaction_not_found(hash)),
        };

        let transaction_bytes = serialize(&transaction);
        let raw_transaction = RawTransaction::new(transaction_bytes.take());

        if verbose {
            let meta = match self.storage.transaction_meta(&hash) {
                Some(meta) => meta,
                None => return Err(transaction_of_side_branch(hash)),
            };

            let block_header = match self.storage.block_header(meta.height().into()) {
                Some(block_header) => block_header,
                None => return Err(transaction_not_found(hash)),
            };

            let best_block = self.storage.best_block();
            if best_block.number < meta.height() {
                return Err(transaction_not_found(hash));
            }

            let txid: H256 = transaction.witness_hash().into();
            let hash: H256 = transaction.hash().into();
            let blockhash: H256 = block_header.hash().into();

            let inputs = transaction
                .clone()
                .inputs
                .into_iter()
                .map(|input| {
                    let txid: H256 = input.previous_output.hash.into();
                    let script_sig_bytes = input.script_sig;
                    let script_sig: Script = script_sig_bytes.clone().into();
                    let script_sig_asm = format!("{}", script_sig);
                    SignedTransactionInput {
                        txid: txid.reversed(),
                        vout: input.previous_output.index,
                        script_sig: TransactionInputScript {
                            asm: script_sig_asm,
                            hex: script_sig_bytes.clone().into(),
                        },
                        sequence: input.sequence,
                        txinwitness: input
                            .script_witness
                            .into_iter()
                            .map(|s| s.clone().into())
                            .collect(),
                    }
                })
                .collect();

            let outputs = transaction
                .clone()
                .outputs
                .into_iter()
                .enumerate()
                .map(|(index, output)| {
                    let script_pubkey_bytes = output.script_pubkey;
                    let script_pubkey: Script = script_pubkey_bytes.clone().into();
                    let script_pubkey_asm = format!("{}", script_pubkey);
                    let script_addresses = script_pubkey.extract_destinations().unwrap_or(vec![]);
                    SignedTransactionOutput {
                        value: 0.00000001f64 * output.value as f64,
                        n: index as u32,
                        script: TransactionOutputScript {
                            asm: script_pubkey_asm,
                            hex: script_pubkey_bytes.clone().into(),
                            req_sigs: script_pubkey.num_signatures_required() as u32,
                            script_type: script_pubkey.script_type().into(),
                            addresses: script_addresses
                                .into_iter()
                                .map(|address| Address {
                                    hash: address.hash,
                                    kind: address.kind,
                                    network: match self.network {
                                        Network::Mainnet => keys::Network::Mainnet,
                                        _ => keys::Network::Testnet,
                                    },
                                })
                                .collect(),
                        },
                    }
                })
                .collect();

            Ok(GetRawTransactionResponse::Verbose(Transaction {
                hex: raw_transaction,
                txid: txid.reversed(),
                hash: hash.reversed(),
                size: transaction.serialized_size(),
                vsize: transaction.serialized_size_with_flags(SERIALIZE_TRANSACTION_WITNESS),
                version: transaction.version,
                locktime: transaction.lock_time as i32,
                vin: inputs,
                vout: outputs,
                blockhash: blockhash.reversed(),
                confirmations: best_block.number - meta.height() + 1,
                time: block_header.time,
                blocktime: block_header.time,
            }))
        } else {
            Ok(GetRawTransactionResponse::Raw(raw_transaction))
        }
    }
}

impl<T> RawClient<T>
where
    T: RawClientCoreApi,
{
    pub fn new(core: T) -> Self {
        RawClient { core: core }
    }
}

impl<T> Raw for RawClient<T>
where
    T: RawClientCoreApi,
{
    fn send_raw_transaction(&self, raw_transaction: RawTransaction) -> Result<H256, Error> {
        let raw_transaction_data: Vec<u8> = raw_transaction.into();
        let transaction =
            try!(deserialize(Reader::new(&raw_transaction_data))
                .map_err(|e| invalid_params("tx", e)));
        self.core
            .accept_transaction(transaction)
            .map(|h| h.reversed().into())
            .map_err(|e| execution(e))
    }

    fn create_raw_transaction(
        &self,
        inputs: Vec<TransactionInput>,
        outputs: TransactionOutputs,
        lock_time: Trailing<u32>,
    ) -> Result<RawTransaction, Error> {
        // reverse hashes of inputs
        let inputs: Vec<_> = inputs
            .into_iter()
            .map(|mut input| {
                input.txid = input.txid.reversed();
                input
            })
            .collect();

        let transaction = try!(self
            .core
            .create_raw_transaction(inputs, outputs, lock_time)
            .map_err(|e| execution(e)));
        let transaction = serialize(&transaction);
        Ok(transaction.into())
    }

    fn decode_raw_transaction(&self, _transaction: RawTransaction) -> Result<Transaction, Error> {
        rpc_unimplemented!()
    }

    fn get_raw_transaction(
        &self,
        hash: H256,
        verbose: Trailing<bool>,
    ) -> Result<GetRawTransactionResponse, Error> {
        let global_hash: GlobalH256 = hash.clone().into();
        self.core
            .get_raw_transaction(global_hash.reversed(), verbose.unwrap_or_default())
    }

    

    fn sign_raw_transaction(
        &self,
        inputs: Vec<TransactionInput>,
        outputs: TransactionOutputs,
        lock_time: Trailing<u32>,
    ) -> Result<RawTransaction, Error> {
        println!("sign_raw_transaction");

        let inputs: Vec<_> = inputs
            .into_iter()
            .map(|mut input| {
                input.txid = input.txid.reversed();
                input
            })
            .collect();

        let mut transaction = try!(self
            .core
            .create_raw_transaction(inputs, outputs, lock_time)
            .map_err(|e| execution(e)));

        let kp_generator = Random::new(Key_Network::Testnet);
        let kp = kp_generator.generate().unwrap();
        //let tx = transaction.clone();

        println!("unsigned transaction {:?}", transaction);

        //let mut tx_mut = transaction.clone();

        for input in &mut transaction.inputs {
            let txid = input.previous_output.hash.clone().into();
            let signature = kp.private().sign(&txid).unwrap();

            let mut sig_byte = signature.take();
            println!("signature {:?}", sig_byte);
            input.script_sig = Bytes::from(sig_byte);
        }

        println!("signed transaction {:?}", transaction);

        let raw_transaction = serialize(&transaction);
        let raw_transaction: RawTransaction = raw_transaction.into();
        self.send_raw_transaction(raw_transaction.clone());
        Ok(raw_transaction)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chain::Transaction;
    use jsonrpc_core::IoHandler;
    use jsonrpc_macros::Trailing;
    use primitives::hash::H256 as GlobalH256;
    use v1::traits::Raw;
    use v1::types::{Bytes, TransactionInput, TransactionOutputs};

    #[derive(Default)]
    struct SuccessRawClientCore;

    #[derive(Default)]
    struct ErrorRawClientCore;

    impl RawClientCoreApi for SuccessRawClientCore {
        fn accept_transaction(&self, transaction: Transaction) -> Result<GlobalH256, String> {
            Ok(transaction.hash())
        }

        fn create_raw_transaction(
            &self,
            _inputs: Vec<TransactionInput>,
            _outputs: TransactionOutputs,
            _lock_time: Trailing<u32>,
        ) -> Result<Transaction, String> {
            Ok("0100000001ad9d38823d95f31dc6c0cb0724c11a3cf5a466ca4147254a10cd94aade6eb5b3230000006b483045022100b7683165c3ecd57b0c44bf6a0fb258dc08c328458321c8fadc2b9348d4e66bd502204fd164c58d1a949a4d39bb380f8f05c9f6b3e9417f06bf72e5c068428ca3578601210391c35ac5ee7cf82c5015229dcff89507f83f9b8c952b8fecfa469066c1cb44ccffffffff0170f30500000000001976a914801da3cb2ed9e44540f4b982bde07cd3fbae264288ac00000000".into())
        }

        fn get_raw_transaction(
            &self,
            _hash: GlobalH256,
            _verbose: bool,
        ) -> Result<GetRawTransactionResponse, Error> {
            Ok(GetRawTransactionResponse::Raw(Bytes::from("0100000001273d7b971b6788f911038f917dfa9ba85980b018a80b2e8caa4fca85475afdaf010000008b48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83ffffffff02003b5808000000001976a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac404b4c00000000001976a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac00000000")))
        }
    }

    impl RawClientCoreApi for ErrorRawClientCore {
        fn accept_transaction(&self, _transaction: Transaction) -> Result<GlobalH256, String> {
            Err("error".to_owned())
        }

        fn create_raw_transaction(
            &self,
            _inputs: Vec<TransactionInput>,
            _outputs: TransactionOutputs,
            _lock_time: Trailing<u32>,
        ) -> Result<Transaction, String> {
            Err("error".to_owned())
        }

        fn get_raw_transaction(
            &self,
            hash: GlobalH256,
            _verbose: bool,
        ) -> Result<GetRawTransactionResponse, Error> {
            Err(transaction_not_found(hash))
        }
    }

    #[test]
    fn sendrawtransaction_accepted() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "sendrawtransaction",
				"params": ["00000000013ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a0000000000000000000101000000000000000000000000"],
				"id": 1
			}"#)
		).unwrap();

        // direct hash is 0791efccd035c5fe501023ff888106eba5eff533965de4a6e06400f623bcac34
        // but client expects reverse hash
        assert_eq!(r#"{"jsonrpc":"2.0","result":"34acbc23f60064e0a6e45d9633f5efa5eb068188ff231050fec535d0ccef9107","id":1}"#, &sample);
    }

    #[test]
    fn sendrawtransaction_rejected() {
        let client = RawClient::new(ErrorRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "sendrawtransaction",
				"params": ["00000000013ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a0000000000000000000101000000000000000000000000"],
				"id": 1
			}"#)
		).unwrap();

        assert_eq!(r#"{"jsonrpc":"2.0","error":{"code":-32015,"message":"Execution error.","data":"\"error\""},"id":1}"#, &sample);
    }

    #[test]
    fn createrawtransaction_success() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "createrawtransaction",
				"params": [[{"txid":"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b","vout":0}],{"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":0.01}],
				"id": 1
			}"#)
		).unwrap();

        assert_eq!(r#"{"jsonrpc":"2.0","result":"0100000001ad9d38823d95f31dc6c0cb0724c11a3cf5a466ca4147254a10cd94aade6eb5b3230000006b483045022100b7683165c3ecd57b0c44bf6a0fb258dc08c328458321c8fadc2b9348d4e66bd502204fd164c58d1a949a4d39bb380f8f05c9f6b3e9417f06bf72e5c068428ca3578601210391c35ac5ee7cf82c5015229dcff89507f83f9b8c952b8fecfa469066c1cb44ccffffffff0170f30500000000001976a914801da3cb2ed9e44540f4b982bde07cd3fbae264288ac00000000","id":1}"#, &sample);
    }

    #[test]
    fn createrawtransaction_error() {
        let client = RawClient::new(ErrorRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "createrawtransaction",
				"params": [[{"txid":"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b","vout":0}],{"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":0.01}],
				"id": 1
			}"#)
		).unwrap();

        assert_eq!(r#"{"jsonrpc":"2.0","error":{"code":-32015,"message":"Execution error.","data":"\"error\""},"id":1}"#, &sample);
    }

    #[test]
    fn getrawtransaction_success() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getrawtransaction",
				"params": ["635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb"],
				"id": 1
			}"#),
            )
            .unwrap();

        assert_eq!(r#"{"jsonrpc":"2.0","result":"0100000001273d7b971b6788f911038f917dfa9ba85980b018a80b2e8caa4fca85475afdaf010000008b48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83ffffffff02003b5808000000001976a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac404b4c00000000001976a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac00000000","id":1}"#, &sample);
    }

    #[test]
    fn getrawtransaction_error() {
        let client = RawClient::new(ErrorRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getrawtransaction",
				"params": ["4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"],
				"id": 1
			}"#),
            )
            .unwrap();

        assert_eq!(r#"{"jsonrpc":"2.0","error":{"code":-32096,"message":"Transaction with given hash is not found","data":"3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"},"id":1}"#, &sample);
    }
}
