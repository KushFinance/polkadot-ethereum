#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for a PolkaETH token
///
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
	storage::StorageMap
};
use sp_core::{U256, RuntimeDebug};
use sp_std::prelude::*;
use artemis_core::{AppID, Application, Message};
use codec::{Encode, Decode};

use artemis_ethereum::{self as ethereum, SignedMessage};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData {
	free: U256,
}

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PolkaETHModule {
		pub TotalIssuance: U256;
		pub Account: map hasher(blake2_128_concat) T::AccountId => AccountData;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Minted(AccountId, U256),
		Burned(AccountId, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Free balance got overflowed after minting.
		FreeMintingOverflow,
		/// Total issuance got overflowed after minting.
		TotalMintingOverflow,
		/// Free balance got underflowed after burning.
		FreeBurningUnderflow,
		/// Total issuance got underflowed after burning.
		TotalBurningUnderflow,
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Burn PolkaETH to release ETH locked up in a bridge contract
		#[weight = 10_000]
		fn burn(origin, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_burn(&who, amount)?;
			Self::deposit_event(RawEvent::Burned(who, amount));
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	fn bytes_to_account_id(data: &[u8]) -> Option<T::AccountId> {
		T::AccountId::decode(&mut &data[..]).ok()
	}

	/// Mint PolkaETH for users who have locked up ETH in a bridge contract.
	fn do_mint(to: &T::AccountId, amount: U256) -> DispatchResult  {
		if amount.is_zero() {
			return Ok(())
		}
		Self::try_mutate_account(to, |account, is_new| -> Result<(), DispatchError> {
			let current_total_issuance = <TotalIssuance>::get();
			let new_total_issuance = current_total_issuance.checked_add(amount)
			.ok_or(Error::<T>::TotalMintingOverflow)?;
			account.free = account.free.checked_add(amount)
				.ok_or(Error::<T>::FreeMintingOverflow)?;
			<TotalIssuance>::set(new_total_issuance);
			Ok(())
		})
	}

	/// Burn PolkaETH to release ETH locked up in a bridge contract
	fn do_burn(to: &T::AccountId, amount: U256) -> DispatchResult  {
		if amount.is_zero() {
			return Ok(())
		}
		Self::try_mutate_account(to, |account, is_new| -> Result<(), DispatchError> {
			let current_total_issuance = <TotalIssuance>::get();
			let new_total_issuance = current_total_issuance.checked_sub(amount)
			.ok_or(Error::<T>::TotalBurningUnderflow)?;
			account.free = account.free.checked_sub(amount)
				.ok_or(Error::<T>::FreeBurningUnderflow)?;
			<TotalIssuance>::set(new_total_issuance);
			Ok(())
		})
	}

	fn try_mutate_account<R, E>(
		who: &T::AccountId,
		f: impl FnOnce(&mut AccountData, bool) -> Result<R, E>
	) -> Result<R, E> {
		<Account<T>>::try_mutate_exists(who, |maybe_account| {
			let is_new = maybe_account.is_none();
			let mut account = maybe_account.take().unwrap_or_default();
			f(&mut account, is_new)
		})
	}

	fn handle_event(event: ethereum::Event) -> DispatchResult {
		match event {
			ethereum::Event::SendETH { recipient, amount, ..} => {
				let account = match Self::bytes_to_account_id(&recipient) {
					Some(account) => account,
					None => {
						return Err(DispatchError::Other("Invalid sender account"))
					}
				};
				Self::do_mint(&account, amount)?;
				Self::deposit_event(RawEvent::Minted(account, amount));
				Ok(())
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
