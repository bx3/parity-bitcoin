use chain::IndexedBlock;
use hash::H32;
use ser::{Deserializable, Error as ReaderError, Reader};
use std::io;

#[derive(Debug, PartialEq)]
pub struct Block {
    pub magic: H32,
    pub block_size: u32,
    pub block: IndexedBlock,
}

impl Deserializable for Block {
    fn deserialize<T>(reader: &mut Reader<T>) -> Result<Self, ReaderError>
    where
        T: io::Read,
    {
        let block = Block {
            magic: try!(reader.read()),
            block_size: try!(reader.read()),
            block: try!(reader.read()),
        };

        Ok(block)
    }
}
