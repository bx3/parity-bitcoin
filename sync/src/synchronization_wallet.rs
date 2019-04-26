use std::collections::{VecDeque, HashMap};
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use parking_lot::{Mutex, Condvar};
use message::{types, common};
use primitives::hash::H256;
use synchronization_executor::{Task, TaskExecutor};
use types::{PeerIndex, RequestId, BlockHeight, StorageRef, ExecutorRef, MemoryPoolRef, PeersRef, LocalNodeRef};
use utils::KnownHashType;
use chain::{Transaction, TransactionInput, TransactionOutput, IndexedTransaction};

// for wallet
use keys::{KeyPair, Public, Private, AddressHash};
use keys::Network as Key_Network;
use keys::generator::Random;
use keys::generator::Generator;
use script::Builder as ScriptBuilder;
use chain::OutPoint;
use script::TransactionInputSigner;
use script::UnsignedTransactionInput;
use script::SignatureVersion;
use primitives::bytes::Bytes;
use std::collections::HashSet;


#[derive(Debug)]
pub enum WalletError {
    InsufficientMoney,
    MissingKey,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct CoinAccessor {
    pub id: String,
    pub outpoint: OutPoint
}

impl CoinAccessor {
    pub fn new(id: String, outpoint: OutPoint) -> Self {
        CoinAccessor {
            id,
            outpoint,
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct Coin {
    id: String,
    outpoint: OutPoint,
    recipient_addr: AddressHash, //recipient
    value: u64
}

impl Coin {
    pub fn new(id: String, outp: OutPoint, recipient_addr: AddressHash, value: u64) -> Self {
        Coin {
            id,
            outpoint: outp,
            recipient_addr,
            value
        }
    }

    pub fn get_id(&self) -> String {self.id.clone()}
    pub fn get_outpoint(&self) -> OutPoint {self.outpoint.clone()}
}


pub struct Wallet {
    local_node: LocalNodeRef,
	coins: HashSet<Coin>,
	keypairs: HashMap<AddressHash, KeyPair>,
}

impl Wallet {
	pub fn new(local_sync_node: LocalNodeRef) -> Self {
        Wallet {
            local_node: local_sync_node,
            coins: HashSet::new(),
            keypairs: HashMap::new(),
        }
	}

	pub fn generate_key_pair(&mut self) {
		let kp_generator = Random::new(Key_Network::Testnet);
		let kp = kp_generator.generate().unwrap();

		let pub_key_hash = kp.public().address_hash();

		if self.keypairs.contains_key(&pub_key_hash) {
			println!("pubkey exists, no key generated, retry");
		} else {
			self.keypairs.insert(pub_key_hash, kp);
		}
	}

    pub fn get_pubkey(&self) -> Result<&Public, WalletError> {
        if let Some(keypair) = self.keypairs.values().next() {
            return Ok(keypair.public());
        }
        Err(WalletError::MissingKey)
    }

    pub fn get_pubkey_hash(&self) -> Result<&AddressHash, WalletError> {
        if let Some(pubkey_hash) = self.keypairs.keys().next() {
            return Ok(pubkey_hash);
        }
        Err(WalletError::MissingKey)
    }


    pub fn get_balance(&self) -> u64 {
        let balance =   self.coins
                        .iter()
                        .map(|coin| coin.value)
                        .sum::<u64>();

        println!("get_balance {}", balance);
        balance
    }

    fn delete_coin(&mut self, coin: &Coin) {
        self.coins.remove(coin);
    }

    //Transaction
    fn create_transaction(&mut self, recipient: AddressHash, value: u64) -> Result<Transaction, WalletError> {
        let mut coins_to_use: Vec<Coin> = vec![];
        let mut value_sum = 0u64;

        // iterate thru our wallet
        for coin in self.coins.iter() {
            value_sum += coin.value;
            coins_to_use.push(coin.clone()); // coins that will be used for this transaction
            if value_sum >= value {
                // if we already have enough money, break
                break;
            }
        }
        if value_sum < value {
            // we don't have enough money in wallet
            return Err(WalletError::InsufficientMoney);
        }


        let script = ScriptBuilder::build_p2pkh(&recipient);

        // if we have enough money in our wallet, create tx

        //tx output currently, single only
        let mut transaction_output = TransactionOutput {
            value: value,
            script_pubkey: script.to_bytes(),
        };
        let mut transaction_outputs = vec![transaction_output];

        if value_sum > value {
            // transfer the remaining value back to self
            let recipient = self.get_pubkey_hash()?;
            transaction_outputs.push(
                TransactionOutput {
                    value: value_sum - value,
                    script_pubkey: script.to_bytes(),
                }
            );
        };

        // create unsigned transaction inputs
        let mut unsigned_inputs: Vec<UnsignedTransactionInput> = vec![];
        for coin in &coins_to_use {
            unsigned_inputs.push(UnsignedTransactionInput {
                    previous_output: coin.outpoint.clone(),
                    sequence: 0x00,
                }
            );
        }

        let unsigned_transactions = TransactionInputSigner {
                                        version: 1,
                                        inputs: unsigned_inputs,
                                        outputs: transaction_outputs.clone(),
                                        lock_time: 0, //no wait to include tx into block
                                };

        let mut signed_inputs: Vec<TransactionInput> = vec![];

        for (i, coin) in coins_to_use.iter().enumerate() {

            let keypair = self.keypairs.get(&coin.recipient_addr).unwrap();
            let to_me_pubkey_script = ScriptBuilder::build_p2pkh(&coin.recipient_addr);
            signed_inputs.push(
                unsigned_transactions.signed_input(keypair, i, coin.value,
                            &to_me_pubkey_script, SignatureVersion::Base, 0x40)
            );
        }

        // remove used coin from wallet
        for c in &coins_to_use {
            self.delete_coin(c);
        }

        let transaction = Transaction {
            version: 1,
            inputs: signed_inputs,
            outputs: transaction_outputs,
            lock_time: 0,
        };
        Ok((transaction))
    }

    pub fn pay(&mut self, recipient: AddressHash, value: u64) {
        println!("i am paying {} to {:?}", value, recipient);
        let tx = match self.create_transaction(recipient, value) {
            Ok(tx) => tx,
            Err(err) => match err {
                InsufficientMoney => {println!("you have insufficient money"); return;},
                MissingKey => {println!("create a pair of private, public key"); return;},
            }
        };

        let indexed_transaction = IndexedTransaction::from(tx);
        let peer_index = 1000;

        // send to local node mempool
        self.local_node.on_transaction(peer_index.clone(), indexed_transaction.clone());
        // send to network
        self.local_node.unsolicited_transaction(peer_index.clone(), indexed_transaction.clone());
    }

    pub fn get_spendable(&mut self) {
        println!("where is my money -ask-> local nodes");
        self.local_node.parse_blocks_get_spendable();
        //let coins: HashSet<CoinAccessor>
        //self.local_node.get_spendable();
    }
}
