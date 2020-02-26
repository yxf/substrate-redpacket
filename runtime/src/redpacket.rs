
//! # RedPacket Module
//!
//!	A simple module for airdropping.
//! 
//!
//! ## Overview
//!
//! RedPacket is a easy way for airdropping, called * 红包 * in chinese.
//! Someone can create a RedPacket that reserve some balances. 
//! Others can claim balances from RedPacket until the RedPacket expired or finished. 
//! Finally, creator of the RedPacket can distribute some amount to all participated accounts.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create` - Create a new RedPacket.
//! * `claim` - Create a claiming record.
//! * `distribute` - After a RedPacket was expired or finished, 
//!    the RedPacket's creator can distribute to all claimed accounts.
//!

use frame_support::{
	StorageValue, StorageMap, 
	decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult, Parameter,
	ensure,
	traits::{Currency, ReservableCurrency, ExistenceRequirement }
};
use codec::{Encode, Decode};
use system::ensure_signed;

use sp_runtime::traits::{SimpleArithmetic, Zero, One, Saturating};
use sp_std::{prelude::*};


pub type BalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Currency: ReservableCurrency<Self::AccountId>;

	/// A u32 type 
	type PacketId: Parameter + SimpleArithmetic + Default + Copy;
}


#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Packet<PacketId, Balance, BlockNumber, AccountId> {
	id: PacketId,
	total: Balance,
	unclaimed: Balance,
	count: u32,
	expires_at: BlockNumber,
	owner: AccountId,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as RedPacket {

		/// All packets.
		pub Packets get(fn packets): map T::PacketId => Packet<T::PacketId, BalanceOf<T>, T::BlockNumber, T::AccountId>;

		/// Get claims of redpacket by id
		pub Claims get(fn claims_of): map T::PacketId => Vec<T::AccountId>;

		/// The next package id.
		pub NextPacketId get(next_packet_id): T::PacketId;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Create a new RedPacket
		/// This will reserve balances(`quota` * `count`) of creator to prevent insufficient balance when distributing.
		/// 
		/// - `quota`: Amount per person will be get.
		/// - `count`: Number of participants.
		/// - `expires`: Expires after `expires` block number passed.
		pub fn create(origin, quota: BalanceOf<T>, count: u32, expires: T::BlockNumber) -> DispatchResult {

			ensure!(count > 0, Error::<T>::GreaterThanZero);
			ensure!(quota > Zero::zero(), Error::<T>::GreaterThanZero);

			let sender = ensure_signed(origin)?;

			let total = quota.saturating_mul(<BalanceOf<T>>::from(count));

			let sender_balance = T::Currency::free_balance(&sender);

			// Make sure sender has sufficient balance 
			ensure!(sender_balance >= total, Error::<T>::InsufficientBalance);

			// Reserve balance for RedPacket
			T::Currency::reserve(&sender, total)?;

			let current_block_number = <system::Module<T>>::block_number();

			let expires_at = current_block_number + expires;
			
			let id = Self::next_packet_id();

			let new_packet = Packet {
				id: id,
				total: total,
				unclaimed: total,
				count: count,
				expires_at: expires_at,
				owner: sender.clone(), 
			};

			<Packets<T>>::insert(id, new_packet);

			<NextPacketId<T>>::mutate(|id| *id += One::one());

			Self::deposit_event(RawEvent::Created(id, sender, total, count));

			Ok(())
		}

		/// Claim some amount from a RedPacket selected by id
		fn claim(origin, packet_id: T::PacketId) -> DispatchResult {
			let user = ensure_signed(origin)?;

			let mut packet = Self::packets(packet_id);

			let current_block_number = <system::Module<T>>::block_number();

			ensure!(current_block_number <= packet.expires_at , Error::<T>::Expired);

			// Check RedPacket available
			ensure!(packet.unclaimed > Zero::zero(), Error::<T>::AlreadyDistributed);

			let claims =  Self::claims_of(packet_id);

			ensure!(!claims.contains(&user), Error::<T>::AlreadyClaimed);

			let claiming_amount = packet.total / <BalanceOf<T>>::from(packet.count);

			packet.unclaimed -= claiming_amount;

			<Packets<T>>::insert(packet_id, packet);

			<Claims<T>>::mutate(packet_id, |claims| claims.push(user.clone()));

			Self::deposit_event(RawEvent::Claimed(packet_id, user, claiming_amount));

			Ok(())
		}

		/// Distribute the RedPacket to claimers.
		/// Iterate `Self::claims`, transfer balances of creator to each participant.
		fn distribute(origin, id: T::PacketId) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut packet = Self::packets(id);

			// Check owner
			ensure!(packet.owner == owner, Error::<T>::NotOwner);
			let current_block_number = <system::Module<T>>::block_number();

			let expired = current_block_number > packet.expires_at;
			let finished = packet.unclaimed == Zero::zero();

			// Redpacket can be distributed when expired or finished.
			if expired || finished {

				// Unreserve balance of Redpacket for transfering
				T::Currency::unreserve(&owner, packet.total);

				let mut total_distributed: BalanceOf<T> = Zero::zero();

				let claims =  Self::claims_of(id);
				let quota = packet.total / <BalanceOf<T>>::from(packet.count);

				for user in claims.into_iter(){
					if user != owner {
						<T::Currency>::transfer(&owner, &user, quota, ExistenceRequirement::KeepAlive)?;
						total_distributed += quota;
					}
				}

				packet.unclaimed = Zero::zero();

				<Packets<T>>::insert(id, packet);

				Self::deposit_event(RawEvent::Distributed(id, owner, total_distributed));

				Ok(())

			} else {
				Err(Error::<T>::CanNotBeDistributed)?
			}
		}
	}
}

decl_event!(
	pub enum Event<T> 
		where 
			AccountId = <T as system::Trait>::AccountId,
			PacketId = <T as Trait>::PacketId,
			Balance = BalanceOf<T>
	{
		/// A new RedPacket was created.
		Created(PacketId, AccountId, Balance, u32),

		/// A new claim was created.
		Claimed(PacketId, AccountId, Balance),

		/// Distribute the RedPacket to claimers.
		Distributed(PacketId, AccountId, Balance),

		/// The RedPacket was Retrieved.
		Retrieved(PacketId, AccountId, Balance),
	}
);

decl_error! {
	/// Error for the treasury module.
	pub enum Error for Module<T: Trait> {
		/// Sender's balance is too low.
		InsufficientBalance,
		/// Parameter must be greater than zero
		GreaterThanZero,
		/// RedPacket was Expired
		Expired,
		/// Aleadly claimed by a Account
		AlreadyClaimed,
		/// Not owner
		NotOwner,
		/// Can not be distributed
		CanNotBeDistributed,
		/// Aleadly distributed
		AlreadyDistributed,

	}
}

