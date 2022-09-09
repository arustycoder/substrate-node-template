#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	//TODO: how about a Option type?
	type KittyIndex = u32;

	#[pallet::type_value]
	pub fn GetDefaultValue() -> KittyIndex {
		0_u32
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyIndex, ValueQuery, GetDefaultValue>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, KittyIndex, Kitty),
		KittyBreed(T::AccountId, KittyIndex, KittyIndex, KittyIndex, Kitty),
		KittyTransferred(T::AccountId, KittyIndex, T::AccountId),
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
			NextKittyId::<T>::set(kitty_id + 1);

			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyIndex,
			kitty_id_2: KittyIndex,
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
			NextKittyId::<T>::set(kitty_id + 1);

			Self::deposit_event(Event::KittyBreed(
				who, kitty_id_1, kitty_id_2, kitty_id, next_kitty,
			));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin1: OriginFor<T>,
			receiver: T::AccountId,
			kitty_id: KittyIndex,
		) -> DispatchResult {
			let sender = ensure_signed(origin1)?;

			ensure!(Self::get_kitty(kitty_id).is_ok(), Error::<T>::KittyNotExists);
			ensure!(
				KittyOwner::<T>::get(kitty_id) == Some(sender.clone()),
				Error::<T>::NotKittyOwner
			);

			KittyOwner::<T>::insert(kitty_id, &receiver);

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

		fn get_next_id() -> Result<KittyIndex, ()> {
			match Self::next_kitty_id() {
				KittyIndex::MAX => Err(()),
				id => Ok(id),
			}
		}

		fn get_kitty(id: KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}
	}
}
