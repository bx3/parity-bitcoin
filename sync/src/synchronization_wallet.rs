use std::collections::{VecDeque, HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use parking_lot::{Mutex, Condvar};
use chain::IndexedTransaction;
use message::{types, common};
use primitives::hash::H256;
use synchronization_executor::{Task, TaskExecutor};
use types::{PeerIndex, RequestId, BlockHeight, StorageRef, ExecutorRef, MemoryPoolRef, PeersRef, LocalNodeRef};
use utils::KnownHashType;

// for wallet
use keys::{KeyPair, Public, Private, AddressHash};
use keys::Network as Key_Network;
use keys::generator::Random;
use keys::generator::Generator;

#[derive(Debug)]
pub enum WalletError {
    InsufficientMoney,
    MissingKey,
}

type CoinId = i32;
struct CoinData {
	value: i32,
	recipient: AddressHash,
}

pub struct Wallet {
	keypairs: HashMap<AddressHash, KeyPair>,
	local_node: LocalNodeRef,
	coins: HashMap<CoinId, CoinData>,
}

impl Wallet {
	pub fn new(local_sync_node: LocalNodeRef) -> Self {
        Wallet {
            local_node: local_sync_node,
            coins: HashMap::new(),
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

}
