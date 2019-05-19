use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use p2p;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::sync::Mutex;
use v1::helpers::errors;
use sync::WalletError;
use v1::traits::Wallet;
use sync::Wallet as LocalWallet;
use v1::types::H160 as AddressHash_ser;
use keys::AddressHash;
use primitives::hash::H256;
use v1::types::H256 as H256_ser;
use chain::OutPoint;

pub struct WalletClient<T: WalletClientCoreApi> {
    core: T,
}

pub trait WalletClientCoreApi: Send + Sync + 'static {
    // get balance for this wallet
    fn get_balance(&self) -> Result<u64, WalletError>;
    // return transaction id, outpoint index is default to be 0
    fn shard_pay(&self, recipient: AddressHash, value: u64) -> Result<H256, WalletError>;
    // check with blockchain to get all spendable coins
    fn update_wallet(&self) -> Result<(), WalletError>;
    // add txid and outpoint to wallet candidate set, so that it can use update_wallet
    fn wallet_add_tx(&self, H256, u32) -> Result<(), WalletError>;
    // generate pub pri key pair for using blockchain
    fn generate_keypair(&self) -> Result<AddressHash, WalletError>;
    // get one pub key hash address from wallet
    fn get_addresshash(&self) -> Result<AddressHash, WalletError>;
    // debug get coin
    fn print_coins(&self);
}

pub trait CovetWalletClientCoreApi {
    fn covet_generate_keypair(&self) -> Result<AddressHash, WalletError>;
    fn covet_get_addresshash(&self) -> Result<AddressHash, WalletError>;
    fn covet_pay(&self, addrhash: AddressHash, value: u64) -> Result<H256, WalletError>;
    fn covet_wallet_add_tx(&self, txid: H256, index: u32) -> Result<(), WalletError>;
}

impl CovetWalletClientCoreApi for WalletClientCore {

    fn covet_generate_keypair(&self) -> Result<AddressHash, WalletError> {
        let mut wallet = self.covetous_wallet.lock().unwrap();
        Ok(wallet.generate_keypair().unwrap())
    }

    fn covet_get_addresshash(&self) -> Result<AddressHash, WalletError> {
        let wallet = self.covetous_wallet.lock().unwrap();
        Ok(wallet.get_addresshash().unwrap())
    }

    fn covet_pay(&self, addrhash: AddressHash, value: u64) -> Result<H256, WalletError> {
        let mut wallet = self.covetous_wallet.lock().unwrap();
        wallet.covet_pay(addrhash, value)
    }

    fn covet_wallet_add_tx(&self, txid: H256, index: u32) -> Result<(), WalletError> {
        let mut wallet = self.covetous_wallet.lock().unwrap();
        let balance = wallet.wallet_add_tx(txid, index);
        Ok(())
    }
}



pub struct WalletClientCore {
    // contain two wallets
    pub wallet: Arc<Mutex<LocalWallet>>,
    pub covetous_wallet: Arc<Mutex<LocalWallet>>,
}

impl WalletClientCore{
    pub fn new(wallet: Arc<Mutex<LocalWallet>>, covetous_wallet: Arc<Mutex<LocalWallet>>) -> Self {
        WalletClientCore {
            wallet: wallet,
            covetous_wallet: covetous_wallet,
        }
    }
}

impl WalletClientCoreApi for WalletClientCore {
    fn get_balance(&self) -> Result<u64, WalletError> {
        let wallet = self.wallet.lock().unwrap();
        Ok(wallet.get_balance())
    }

    fn shard_pay(&self, addrhash: AddressHash, value: u64) -> Result<H256, WalletError> {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.pay(addrhash, value)
    }

    fn update_wallet(&self) -> Result<(), WalletError> {
        let mut wallet = self.wallet.lock().unwrap();
        let balance = wallet.update_wallet();
        Ok(())
    }

    fn wallet_add_tx(&self, txid: H256, index: u32) -> Result<(), WalletError> {
        let mut wallet = self.wallet.lock().unwrap();

        let balance = wallet.wallet_add_tx(txid, index);
        Ok(())
    }

    fn generate_keypair(&self) -> Result<AddressHash, WalletError> {
        let mut wallet = self.wallet.lock().unwrap();
        Ok(wallet.generate_keypair().unwrap())
    }

    fn get_addresshash(&self) -> Result<AddressHash, WalletError> {
        let wallet = self.wallet.lock().unwrap();
        Ok(wallet.get_addresshash().unwrap())
    }

    fn print_coins(&self) {
        let wallet = self.wallet.lock().unwrap();
        wallet.print_coins();
    }

}

impl<T> WalletClient<T>
where
    T: WalletClientCoreApi,
{
    pub fn new(core: T) -> Self {
        WalletClient { core: core }
    }

    pub fn format_error_msg(&self, e: WalletError) -> Error {
        let mut err_with_message = Error::invalid_request();
        match e {
            InsufficientMoney => err_with_message.message = "InsufficientMoney".to_string(),
            EmptyKeySpace => err_with_message.message = "EmptyKeySpace".to_string(),
            MissingKeypairForAddressHash => err_with_message.message = "MissingKeypairForAddressHash".to_string(),
            DuplicatePublicKey =>err_with_message.message = "DuplicatePublicKey".to_string(),
        }
        err_with_message
    }
}

impl<T> Wallet for WalletClient<T>
where
    T: WalletClientCoreApi + CovetWalletClientCoreApi
{
    fn get_balance(&self) -> Result<u64, Error> {
        match self.core.get_balance() {
            Ok(result) => Ok(result),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }

    fn shard_pay(&self, addrhash: AddressHash_ser, value: u64) -> Result<H256_ser, Error> {
        let mut receipant_hash: AddressHash = addrhash.clone().into();
        match self.core.shard_pay(receipant_hash, value) {
            Ok(result) => Ok(result.into()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }

    fn update_wallet(&self) -> Result<(), Error> {
        match self.core.update_wallet() {
            Ok(()) => Ok(()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }

    fn wallet_add_tx(&self, txid: H256_ser, index: u32) -> Result<(), Error> {
        let txid = txid.reversed().into();
        match self.core.wallet_add_tx(txid, index) {
            Ok(()) => Ok(()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }

    fn generate_keypair(&self) -> Result<AddressHash_ser, Error> {
        match self.core.generate_keypair() {
            Ok(addr) => Ok(addr.into()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }

    fn get_addresshash(&self) -> Result<AddressHash_ser, Error> {
        match self.core.get_addresshash() {
            Ok(addr) => Ok(addr.into()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }

    fn print_coins(&self) -> Result<(), Error> {
        self.core.print_coins();
        Ok(())        
    }

    fn covet_generate_keypair(&self) -> Result<AddressHash_ser, Error> {
        match self.core.covet_generate_keypair() {
            Ok(addr) => Ok(addr.into()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }

    fn covet_get_addresshash(&self) -> Result<AddressHash_ser, Error> {
        match self.core.covet_get_addresshash() {
            Ok(addr) => Ok(addr.into()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }


    fn covet_pay(&self, addrhash: AddressHash_ser, value: u64) -> Result<H256_ser, Error> {
        let mut receipant_hash: AddressHash = addrhash.clone().into();
        match self.core.covet_pay(receipant_hash, value) {
            Ok(result) => Ok(result.into()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }


    fn covet_wallet_add_tx(&self, txid: H256_ser, index: u32) -> Result<(), Error> {
        let txid = txid.reversed().into();
        match self.core.covet_wallet_add_tx(txid, index) {
            Ok(()) => Ok(()),
            Err(e) => Err(self.format_error_msg(e)),
        }
    }
}
