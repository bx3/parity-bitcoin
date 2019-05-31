//RR_edit
extern crate rand;
use bytes::Bytes;
use constants::{LOCKTIME_THRESHOLD, SEQUENCE_FINAL};
use crypto::dhash256;
use hash::H256;
use heapsize::HeapSizeOf;
use hex::FromHex;
use rand::Rng;
use ser::{deserialize, serialize, serialize_with_flags, SERIALIZE_TRANSACTION_WITNESS};
use ser::{Deserializable, Error, Reader, Serializable, Stream};
use std::io;
use compact::Compact;
use std::fmt;

/*
/// Must be zero.
const WITNESS_MARKER: u8 = 0;
/// Must be nonzero.
const WITNESS_FLAG: u8 = 1;
*/

#[derive(Debug, PartialEq, Default, Clone, Serializable, Deserializable)]
pub struct ShardHeader {
    pub version: i32,
    pub ShardID: i32,
    pub shard_block_hashes: Vec<H256>,
}

impl ShardHeader{
    pub fn hash(&self) -> H256 {
        dhash256(&serialize(self))
    }
}
