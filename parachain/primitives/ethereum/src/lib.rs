#![allow(dead_code)]
#![allow(unused_variables)]

pub type RawMessage = Vec<u8>;
pub type Signature = Vec<u8>;

pub fn verify(message: RawMessage, signature: Signature) -> bool {
	true
}
