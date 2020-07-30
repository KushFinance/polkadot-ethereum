#![allow(dead_code)]
#![allow(unused_variables)]

use std::io::prelude::*;

use rlp::{Rlp};

use ethereum_types::{H160, H256};

use ethabi::{self, Bytes, Token};

pub type RawMessage = Vec<u8>;
pub type Signature = Vec<u8>;

pub fn verify(message: RawMessage, signature: Signature) -> bool {
	true
}

pub fn decodeLog(contract: ethabi::Contract, data: Vec<u8>) -> bool {

	let rlp = Rlp::new(data.as_ref());

	let mut iter = rlp.iter();

	let address: H160 = iter.next().unwrap().as_val().unwrap();
	let topics: Vec<H256> = iter.next().unwrap().as_list().unwrap();
	let data = iter.next().unwrap().data().unwrap().to_vec();

	let event = contract.event("AppEvent").unwrap();
	let raw_log = ethabi::RawLog::from((topics, data));
	let log = event.parse_log(raw_log).unwrap();	

	for param in &log.params {
		println!("{:?}", param);
	}

	let token = &log.params[2].value;

	if let Token::Bytes(foo) = token {
		let rlp2 = Rlp::new(foo.as_ref());
		let mut iter2 = rlp2.iter();
		let aa = iter2.next().unwrap();
		println!("{:?}", aa);

	}

	true
}
#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io::BufReader;
	use super::*;

    #[test]
    fn test_decode() {
		// read RLP
		let mut reader = BufReader::new(File::open("/tmp/log.rlp").unwrap());
		let mut data: Vec<u8> = Vec::new();
		reader.read_to_end(&mut data).unwrap();

		// read ABI
		let mut creader = BufReader::new(File::open("/tmp/Bank.json").unwrap());
		let contract = ethabi::Contract::load(creader).unwrap();

        assert_eq!(decodeLog(contract, data), true);
    }
}