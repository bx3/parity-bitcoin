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
    fn get_balance(&self) -> Result<(), Error>;
    fn shard_pay(&self, recipient: AddressHash, value: u64) -> Result<(), Error>;
    fn get_spendable(&self) -> Result<(), Error>;
    fn wallet_add_tx(&self, H256, u32) -> Result<(), Error>;
    fn generate_keypair(&self) -> Result<AddressHash, Error>;
}

pub struct WalletClientCore {
    pub wallet: Arc<Mutex<LocalWallet>>,
}

impl WalletClientCore{
    pub fn new(wallet: Arc<Mutex<LocalWallet>>) -> Self {
        WalletClientCore {
            wallet: wallet,
        }
    }
}

impl WalletClientCoreApi for WalletClientCore {
    fn get_balance(&self) -> Result<(), Error> {
        let wallet = self.wallet.lock().unwrap();
        let balance = wallet.get_balance();
        Ok(())
    }

    fn shard_pay(&self, addrhash: AddressHash, value: u64) -> Result<(), Error> {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.pay(addrhash, value);
        Ok(())
    }

    fn get_spendable(&self) -> Result<(), Error> {
        let mut wallet = self.wallet.lock().unwrap();
        let balance = wallet.get_spendable();
        Ok(())
    }

    fn wallet_add_tx(&self, txid: H256, index: u32) -> Result<(), Error> {
        let mut wallet = self.wallet.lock().unwrap();

        let balance = wallet.add_tx_to_candidate(txid, index);
        Ok(())
    }

    fn generate_keypair(&self) -> Result<AddressHash, Error> {
        let mut wallet = self.wallet.lock().unwrap();
        Ok(wallet.generate_keypair().unwrap())
    }
}

impl<T> WalletClient<T>
where
    T: WalletClientCoreApi,
{
    pub fn new(core: T) -> Self {
        WalletClient { core: core }
    }
}

impl<T> Wallet for WalletClient<T>
where
    T: WalletClientCoreApi,
{
    fn get_balance(&self) -> Result<(), Error> {
        Ok(self.core.get_balance().unwrap())
    }

    fn shard_pay(&self, addrhash: AddressHash_ser, value: u64) -> Result<(), Error> {
        let mut receipant_hash: AddressHash = addrhash.clone().into();
        Ok(self.core.shard_pay(receipant_hash, value).unwrap())
    }

    fn get_spendable(&self) -> Result<(), Error> {
        Ok(self.core.get_spendable().unwrap())
    }

    fn wallet_add_tx(&self, txid: H256_ser, index: u32) -> Result<(), Error> {
        let txid = txid.reversed().into();
        self.core.wallet_add_tx(txid, index);
        Ok(())
    }

    fn generate_keypair(&self) -> Result<AddressHash_ser, Error> {
        let addr = self.core.generate_keypair().unwrap();
        Ok(addr.into())
    }
}
