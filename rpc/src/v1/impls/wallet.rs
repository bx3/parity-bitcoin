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

pub struct WalletClient<T: WalletClientCoreApi> {
    core: T,
}

pub trait WalletClientCoreApi: Send + Sync + 'static {
    fn get_balance(&self) -> Result<(), Error>;
    fn shard_pay(&self, recipient: AddressHash, value: u64) -> Result<(), Error>;
    fn get_spendable(&self) -> Result<(), Error>;
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
}
