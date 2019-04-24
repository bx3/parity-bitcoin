use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Result};
use std::sync::{Arc, Mutex};
use keys::{KeyPair, Public, Private, AddressHash};
use miner::memory_pool::MemoryPool;
use Arc<RwLock<MemoryPool>>


//pub struct WalletDb {
//	key_db_path: String,
//	coin_db_path: String,
//	keypairs: HashMap<AddressHash, KeyPair>,
//	mempool: Arc<RwLock<MemoryPool>>,

//}

impl WalletDb {
	fn new(file_name: &str)
	{

	}

	//fn new(file_name: &str) {
	//	self.db_path = file_name.to_string();
	//	self.pub_pri_map = HashMap::new();
	//	self.pri_pub_map = HashMap::new();
	//}

	//fn open_db(&self) {
	//	if Path::new(&self.db_path).exists() {
	//		let file = File::open(self.db_path).unwrap();

	//		for line in BufReader::new(file).lines() {
	//			let mut token: Vec<&str> = line.unwrap().split_whitespace().collect();

	//			let pub_key = token[0].to_string();
	//			let pri_key = token[1].to_string();
	//			self.pub_pri_map.insert(pub_key.clone(), pri_key.clone());
	//			self.pri_pub_map.insert(pri_key, pub_key);
	//		}
	//	} else {
	//		OpenOptions::new()
    //        .create(true)
    //        .write(true)
    //        .append(true)
    //        .open(self.db_path)
    //        .unwrap();
	//	}
	//}
}
