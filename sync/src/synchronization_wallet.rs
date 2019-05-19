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
use std::convert::From;


#[derive(Debug)]
pub enum WalletError {
    InsufficientMoney,
    EmptyKeySpace,
    MissingKeypairForAddressHash,
    DuplicatePublicKey,
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
    pub fn get_new_outpoint(&self) -> OutPoint {self.outpoint.clone()}
}

impl From<Coin> for CoinAccessor {
    fn from(coin: Coin) -> Self{
        CoinAccessor {
            id: coin.id.clone(),
            outpoint: coin.outpoint.clone()
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
    coins_candidate: HashSet<CoinAccessor>,
    num_coin: u64,
}

impl Wallet {
	pub fn new(local_sync_node: LocalNodeRef) -> Self {
        Wallet {
            local_node: local_sync_node,
            coins: HashSet::new(),
            keypairs: HashMap::new(),
            coins_candidate: HashSet::new(),
            num_coin: 0,
        }
	}

	pub fn generate_keypair(&mut self) -> Result<AddressHash, WalletError> {
		let kp_generator = Random::new(Key_Network::Testnet);
		let kp = kp_generator.generate().unwrap();

		let pub_key_hash = kp.public().address_hash();

		if self.keypairs.contains_key(&pub_key_hash) {
			println!("pubkey exists, no key generated, retry");
            return Err(WalletError::DuplicatePublicKey);
		} else {
			self.keypairs.insert(pub_key_hash.clone(), kp);
		}
        Ok(pub_key_hash)
	}

    pub fn get_pubkey(&self) -> Result<&Public, WalletError> {
        if let Some(keypair) = self.keypairs.values().next() {
            return Ok(keypair.public());
        }
        Err(WalletError::EmptyKeySpace)
    }


    pub fn get_addresshash(&self) -> Result<AddressHash, WalletError> {
        if let Some(pubkey_hash) = self.keypairs.keys().next() {
            return Ok(pubkey_hash.clone());
        }
        Err(WalletError::EmptyKeySpace)
    }


    pub fn get_balance(&self) -> u64 {
        let balance =   self.coins
                        .iter()
                        .map(|coin| {
                            coin.value
                        })
                        .sum::<u64>();

        balance
    }

    fn delete_coin(&mut self, coin: &Coin) {
        self.coins.remove(coin);
        self.coins_candidate.remove(&CoinAccessor {
            id: coin.id.clone(),
            outpoint: coin.outpoint.clone(),
        });
    }

    fn add_coin_candidate(&mut self, outpoint: OutPoint, desc: String) {
        self.num_coin += 1;
        let msg = format!("{}: {}", self.num_coin, desc);
        self.coins_candidate.insert(
            CoinAccessor {
                id: msg,
                outpoint,
            }
        );
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
            println!("WalletError::InsufficientMoney {} < {}", value_sum, value);
            return Err(WalletError::InsufficientMoney);
        }

        let script = ScriptBuilder::build_p2pkh(&recipient);

        // if we have enough money in our wallet, create tx

        //tx output currently, single only
        let mut transaction_outputs = vec![
            TransactionOutput {
                value: value,
                script_pubkey: script.to_bytes(),
        }];

        if value_sum > value {
            // transfer the remaining value back to self
            let self_recipient = self.get_addresshash()?;
            let pay_self_script = ScriptBuilder::build_p2pkh(&self_recipient);
            transaction_outputs.push(
                TransactionOutput {
                    value: value_sum - value,
                    script_pubkey: pay_self_script.to_bytes(),
                }
            );
        };

        // create unsigned transaction inputs
        let mut unsigned_inputs: Vec<UnsignedTransactionInput> = vec![];
        for coin in &coins_to_use {
            println!("use coin {:?}\n", coin.id);
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
            let keypair = match self.keypairs.get(&coin.recipient_addr) {
                None => {
                    //println!("MissingKeypairForAddressHash");
                    //println!("coin.recipient_addr {:#?}", coin.recipient_addr);
                    //for kp in self.keypairs.values() {
                    //    println!("keypair {:#?}", kp.public().address_hash());
                    //}
                    return Err(WalletError::MissingKeypairForAddressHash);
                },
                Some(kp) => kp,
            };
            //println!("use keypair {:#?}", keypair);
            //println!("addresshash {:#?}\n", keypair.public().address_hash());
            let to_me_pubkey_script = ScriptBuilder::build_p2pkh(&coin.recipient_addr);
            signed_inputs.push(
                unsigned_transactions.signed_input(keypair, i, coin.value,
                            &to_me_pubkey_script, SignatureVersion::Base, 0x40)
            );
        }

        // remove used coin from wallet
        for c in &coins_to_use {
            println!("delete coin id {:?}, value {}", c.get_id(), c.value);
            self.delete_coin(c);
        }

        let transaction = Transaction {
            version: 1,
            inputs: signed_inputs,
            outputs: transaction_outputs,
            lock_time: 0,
        };

        let return_outpoint = OutPoint { hash: transaction.hash(), index: 1};
        self.add_coin_candidate(return_outpoint, "pay to self".to_string());

        Ok(transaction)
    }

    pub fn pay(&mut self, recipient: AddressHash, value: u64) -> Result<H256, WalletError> {
        let tx = self.create_transaction(recipient, value)?;

        let indexed_transaction = IndexedTransaction::from(tx);
        let peer_index = 1000;

        // send to local node mempool
        self.local_node.on_transaction(peer_index.clone(), indexed_transaction.clone());
        // send to network
        self.local_node.unsolicited_transaction(peer_index.clone(), indexed_transaction.clone());

        Ok(indexed_transaction.hash) //.reversed()
    }

    pub fn update_wallet(&mut self) {
        self.coins = self.local_node.get_spendable(&mut self.coins_candidate);
    }

    pub fn wallet_add_tx(&mut self,  hash: H256, index: u32) {
        //println!("add tx {:?} out {} to wallet candidate pool", hash, index);
        let id = self.num_coin.clone().to_string();
        let outpoint = chain::OutPoint {
                            hash: hash.reversed(),
                            index
                        };
        let desc = "from network".to_string();
        self.add_coin_candidate(outpoint ,desc);
    }

    pub fn print_coins(&self) {
        println!("\n**********spendable coin");
        for coin in self.coins.iter() {
            println!("coin {}; value {}", coin.id, coin.value);
        }

        println!("**********coin Accessor");
        for coin in self.coins_candidate.iter() {
            println!("coin acc {}", coin.id);
        }
        println!("********************");
    }

    pub fn covet_pay(&self, recipient: AddressHash, value: u64) -> Result<H256, WalletError> {
        let tx = self.create_covet_transaction(recipient, value)?;

        let indexed_transaction = IndexedTransaction::from(tx);
        let peer_index = 1000;

        // send to local node mempool
        self.local_node.on_transaction(peer_index.clone(), indexed_transaction.clone());
        // send to network
        self.local_node.unsolicited_transaction(peer_index.clone(), indexed_transaction.clone());

        Ok(indexed_transaction.hash)
    }

    pub fn create_covet_transaction(&self, recipient: AddressHash, value: u64) -> Result<Transaction, WalletError> {
        let script = ScriptBuilder::build_p2pkh(&recipient);
        //tx output currently, single only
        let mut transaction_outputs = vec![
            TransactionOutput {
                value: value,
                script_pubkey: script.to_bytes(),
        }];

        //void Coin
        let void_coin = OutPoint::null();

        let mut unsigned_inputs: Vec<TransactionInput> = vec![];

        println!("use void coin");
        unsigned_inputs.push(TransactionInput {
                previous_output: void_coin,
                script_sig: Bytes::new(),
                sequence: 0x00,
                script_witness: vec![],
            }
        );

        let transaction = Transaction {
            version: 1,
            inputs: unsigned_inputs,
            outputs: transaction_outputs,
            lock_time: 0,
        };
        Ok(transaction)
    }

}
