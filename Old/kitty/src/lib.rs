#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

//#[cfg(test)]
//mod mock;

//#[cfg(test)]
//mod tests;

//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_support::inherent::Vec;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	//use frame_support::pallet_prelude::*;
	//use frame_system::pallet_prelude::*;

	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kittys<T:Config> {
		id: u32,
		dna: Vec<u8>,
		price: u32,
		gender: Gender,
		owner: T::AccountId,
	}

	pub type Id = u32;

	#[derive(TypeInfo, Encode ,Decode, Debug)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender{
		fn default()-> Self{
			Gender::Male
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type KittyID<T> = StorageValue<_, Id, ValueQuery>;

	#[pallet::storage]
	pub(super) type KittyList<T: Config> = StorageMap<_, Blake2_128Concat, Id, Kittys<T>, OptionQuery>;

	#[pallet::storage]
	pub(super) type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Kittys<T>>, OptionQuery>;

	// // The pallet's runtime storage items.
	// // https://docs.substrate.io/v3/runtime/storage
	// #[pallet::storage]
	// #[pallet::getter(fn something)]
	// // Learn more about declaring storage items:
	// // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	// pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important chan 	ges are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		//SomethingStored(u32, T::AccountId),
		KittyStored(Vec<u8>, T::AccountId),
		KittySwapped(T::AccountId, u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		// Error kitty is not exist
		KittyNotExist,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		/// kitty creating logics
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>, price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let kitty_gender = Self::gen_kitty_gender(dna.clone())?;

			let mut current_kitty_id = <KittyID<T>>::get();

			current_kitty_id += 1;

			let kitty = Kittys {
				id: current_kitty_id,
				dna: dna.clone(),
				price: price,
				gender: kitty_gender,
				owner: who.clone(),
			};

			// Update storage.
			KittyList::<T>::insert(current_kitty_id, &kitty);

			KittyID::<T>::put(current_kitty_id);

			//update kitty ownership
			let kitty_owner = <KittyOwner<T>>::get(who.clone());
			let mut kitty_owner_vec = match kitty_owner{
				None => Vec::new(),
				_	 => <KittyOwner<T>>::get(who.clone()).unwrap(),
			};

			kitty_owner_vec.push(kitty);
			<KittyOwner<T>>::insert(who.clone(), kitty_owner_vec);

			// Emit an event.
			Self::deposit_event(Event::KittyStored(dna, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// kitty swapping logics
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn swap_kitty(origin: OriginFor<T>, swap_kitty_id: u32, to_account: T::AccountId) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let swapped_kitty = <KittyList<T>>::get(swap_kitty_id);

			ensure!(swapped_kitty.is_some(), Error::<T>::KittyNotExist);

			let mut kitty = swapped_kitty.unwrap();

			// Update Kitty owner
			let current_owner = kitty.owner;

			// Set new owner
			kitty.owner = to_account.clone();

			// Update Kitty list
			<KittyList<T>>::insert(swap_kitty_id, &kitty);

			// Update Kitty_Owner storage
			//// prev owner
			let prev_kitty_owner = <KittyOwner<T>>::get(&current_owner);
			let mut prev_kitty_owner_vec = match prev_kitty_owner{
				None => Vec::new(),
				_	 =>	prev_kitty_owner.unwrap(),
			};

			//// remove kitty in list owner
			if let Some(index) = prev_kitty_owner_vec.iter().position(|value| value.id == swap_kitty_id) {
				prev_kitty_owner_vec.swap_remove(index);
			}
			<KittyOwner<T>>::insert(current_owner, prev_kitty_owner_vec);

			//// New Owner
			let new_kitty_owner = <KittyOwner<T>>::get(&to_account);
			let mut new_kitty_owner_vec = match new_kitty_owner{
				None => Vec::new(),
				_	 =>	new_kitty_owner.unwrap(),
			};
			new_kitty_owner_vec.push(kitty);
			<KittyOwner<T>>::insert(to_account.clone(), new_kitty_owner_vec);

			// Emit an event.
			Self::deposit_event(Event::KittySwapped(who, swap_kitty_id, to_account));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

// helper function
impl<T> Pallet<T> {
	fn gen_kitty_gender(dna: Vec<u8>) -> Result<Gender,Error<T>>{
		let mut res = Gender::Male;
		if dna.len() % 2 !=0 {
			res = Gender::Female;
		}
		Ok(res)
	}
}
