#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::Codec;
	use frame_support::{pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded};
	use sp_std::fmt::Debug;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type KittyIndex: Parameter
			+ Bounded
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo;

		#[pallet::constant]
		type MaxOwnedKitties: Get<u32>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, <T as Config>::KittyIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, <T as Config>::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as Config>::KittyIndex, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn owned_kitties)]
	pub type OwnedKitties<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<<T as Config>::KittyIndex, T::MaxOwnedKitties>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, <T as Config>::KittyIndex, Kitty),
		KittyBreed(
			T::AccountId,
			<T as Config>::KittyIndex,
			<T as Config>::KittyIndex,
			<T as Config>::KittyIndex,
			Kitty,
		),
		KittyTransferred(T::AccountId, <T as Config>::KittyIndex, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyIndex,
		KittyNotExists,
		SameKittyId,
		NotKittyOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyIndex)?;

			let dna = Self::random_value(&who);
			let kitty = Kitty(dna);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKittyId::<T>::set(kitty_id + <T as Config>::KittyIndex::from(1u32));
			OwnedKitties::<T>::mutate(&who, |kitties| kitties.try_push(kitty_id).unwrap());

			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: <T as Config>::KittyIndex,
			kitty_id_2: <T as Config>::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			let kitty1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::KittyNotExists)?;
			let kitty2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::KittyNotExists)?;

			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyIndex)?;

			let selector = Self::random_value(&who);

			let mut data = [0u8; 16];
			for i in 0..kitty1.0.len() {
				data[i] = (kitty1.0[i] & selector[i]) | (kitty2.0[i] & selector[i]);
			}

			let next_kitty = Kitty(data);

			Kitties::<T>::insert(kitty_id, &next_kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKittyId::<T>::set(kitty_id + <T as Config>::KittyIndex::from(1u32));
			// FixMe: handle error
			OwnedKitties::<T>::mutate(&who, |kitties| {
				let _ = kitties.try_push(kitty_id);
			});

			Self::deposit_event(Event::KittyBreed(
				who, kitty_id_1, kitty_id_2, kitty_id, next_kitty,
			));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin1: OriginFor<T>,
			receiver: T::AccountId,
			kitty_id: <T as Config>::KittyIndex,
		) -> DispatchResult {
			let sender = ensure_signed(origin1)?;

			ensure!(Self::get_kitty(kitty_id).is_ok(), Error::<T>::KittyNotExists);
			ensure!(
				KittyOwner::<T>::get(kitty_id) == Some(sender.clone()),
				Error::<T>::NotKittyOwner
			);

			KittyOwner::<T>::insert(kitty_id, &receiver);
			OwnedKitties::<T>::mutate(&sender, |kitties| {
				let idx = {
					let mut found = None;
					for (i, x) in kitties.iter().enumerate() {
						if x == &kitty_id {
							found = Some(i);
							break;
						}
					}
					found
				};
				if let Some(idx) = idx {
					kitties.remove(idx);
				}
			});
			OwnedKitties::<T>::mutate(&receiver, |kitties| kitties.try_push(kitty_id).unwrap());

			Self::deposit_event(Event::KittyTransferred(sender, kitty_id, receiver));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// get random 256.
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}

		fn get_next_id() -> Result<<T as Config>::KittyIndex, ()> {
			let max = <T as Config>::KittyIndex::max_value();
			match Self::next_kitty_id() {
				id if id == max => Err(()),
				id => Ok(id),
			}
		}

		fn get_kitty(id: <T as Config>::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}
	}
}
