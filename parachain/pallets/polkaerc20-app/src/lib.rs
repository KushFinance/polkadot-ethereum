#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for PolkaERC20 token assets
///
use sp_std::prelude::*;
use sp_core::{H160, U256};
use frame_system::{self as system};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
	storage::StorageDoubleMap
};

use codec::{Decode};

use artemis_core::{AppID, Application, Message};
use artemis_ethereum::{self as ethereum, SignedMessage};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PolkaERC20Map {
		pub TotalIssuance: map hasher(blake2_128_concat) H160 => U256;
		pub FreeBalance: double_map hasher(blake2_128_concat) H160, hasher(blake2_128_concat) T::AccountId => U256;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Minted(AccountId, H160, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Free balance got overflowed after minting.
		FreeMintingOverflow,
		/// Total issuance got overflowed after minting.
		TotalMintingOverflow,
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

	}
}

impl<T: Trait> Module<T> {

	fn bytes_to_account_id(data: &[u8]) -> Option<T::AccountId> {
		T::AccountId::decode(&mut &data[..]).ok()
	}

	fn do_mint(token_addr: H160, to: &T::AccountId, amount: U256) -> DispatchResult {
		let current_total_issuance = <TotalIssuance>::get(&token_addr);
		let original_free_balance = <FreeBalance<T>>::get(&token_addr, to);
		let new_total_issuance = current_total_issuance.checked_add(amount)
		.ok_or(Error::<T>::TotalMintingOverflow)?;
		let value = original_free_balance.checked_add(amount)
			.ok_or(Error::<T>::FreeMintingOverflow)?;
		<FreeBalance<T>>::insert(&token_addr, to, value);
		<TotalIssuance>::insert(&token_addr, new_total_issuance);
		Self::deposit_event(RawEvent::Minted(to.clone(), token_addr, amount));
		Ok(())
	}

	fn handle_event(event: ethereum::Event) -> DispatchResult {
		match event {
			ethereum::Event::SendERC20 { recipient, token, amount, ..} => {
				let account = match Self::bytes_to_account_id(&recipient) {
					Some(account) => account,
					None => {
						return Err(DispatchError::Other("Invalid sender account"))
					}
				};
				Self::do_mint(token, &account, amount)
			}
			_ => {
				// Ignore all other ethereum events. In the next milestone the
				// application will only receive messages it is registered to handle
				Ok(())
			}
		}
	}

}

impl<T: Trait> Application for Module<T> {

	fn handle(_app_id: AppID, message: Message) -> DispatchResult {
		let sm = match SignedMessage::decode(&mut message.as_slice()) {
			Ok(sm) => sm,
			Err(_) => return Err(DispatchError::Other("Failed to decode event"))
		};

		let event = match ethereum::Event::decode_from_rlp(sm.data) {
			Ok(event) => event,
			Err(_) => return Err(DispatchError::Other("Failed to decode event"))
		};

		Self::handle_event(event)
	}
}
