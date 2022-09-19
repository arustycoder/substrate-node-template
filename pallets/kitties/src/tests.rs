use crate::{mock::*, Config, Error, NextKittyId};
use frame_support::{assert_err, assert_ok};
use frame_system::Origin;
use sp_runtime::traits::{BadOrigin, Get};

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let origin = Origin::signed(1);
		let kitty_id = <Test as Config>::KittyIndex::from(0u8);

		assert_ok!(KittiesModule::create(origin));
		assert!(KittiesModule::kitties(kitty_id).is_some());
		assert_eq!(KittiesModule::next_kitty_id(), <Test as Config>::KittyIndex::from(1u8));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(1));
	})
}

#[test]
fn create_unsigned_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let unsigned = Origin::none();

		assert_err!(KittiesModule::create(unsigned), BadOrigin);
	})
}

#[test]
fn create_invalid_kitty_idx_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let origin = Origin::signed(1);

		NextKittyId::<Test>::set(<Test as Config>::KittyIndex::max_value());

		assert_err!(KittiesModule::create(origin), Error::<Test>::InvalidKittyIndex);
	})
}

#[test]
fn create_too_many_kitties_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let origin = Origin::signed(1);
		let limit: u32 = <Test as Config>::MaxOwnedKitties::get();

		for _ in 0..limit {
			assert_ok!(KittiesModule::create(origin.clone()));
		}

		assert_err!(KittiesModule::create(origin), Error::<Test>::TooManyKitties);
	})
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let sender = Origin::signed(1);
		let receiver = 2;
		let kitty_id = <Test as Config>::KittyIndex::from(0);

		assert_ok!(KittiesModule::create(sender.clone()));

		assert_ok!(KittiesModule::transfer(sender.clone(), receiver, kitty_id));

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(2));
	})
}

#[test]
fn transfer_kitty_not_exists_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let owner = Origin::signed(1);
		let receiver = 3;
		let kitty_id = <Test as Config>::KittyIndex::from(0);
		let kitty_id1 = <Test as Config>::KittyIndex::from(1);

		assert_ok!(KittiesModule::create(owner.clone()));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(1));
		assert_err!(
			KittiesModule::transfer(owner, receiver, kitty_id1),
			Error::<Test>::KittyNotExists
		);
	})
}

#[test]
fn transfer_not_owner_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let owner = Origin::signed(1);
		let other = Origin::signed(2);
		let kitty_id = <Test as Config>::KittyIndex::from(0);

		assert_ok!(KittiesModule::create(owner));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(1));
		assert_err!(KittiesModule::transfer(other, 3, kitty_id), Error::<Test>::NotKittyOwner);
	})
}

#[test]
fn transfer_too_many_kitties_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let owner = Origin::signed(1);
		let receiver_id = 2;
		let receiver = Origin::signed(receiver_id);
		let kitty_id = <Test as Config>::KittyIndex::from(0);
		let limit: u32 = <Test as Config>::MaxOwnedKitties::get();

		assert_ok!(KittiesModule::create(owner.clone()));

		for id in 1..=limit as u8 {
			assert_ok!(KittiesModule::create(receiver.clone()));
			let kid = <Test as Config>::KittyIndex::from(id);
			assert_eq!(KittiesModule::kitty_owner(kid), Some(receiver_id));
		}

		assert_err!(
			KittiesModule::transfer(owner, receiver_id, kitty_id),
			Error::<Test>::TooManyKitties
		);
	})
}

#[test]
fn breed_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let owner = Origin::signed(1);

		assert_ok!(KittiesModule::create(owner.clone()));
		assert_ok!(KittiesModule::create(owner.clone()));

		assert_eq!(KittiesModule::kitty_owner(0), Some(1));
		assert_eq!(KittiesModule::kitty_owner(1), Some(1));

		let kitty_id_0 = <Test as Config>::KittyIndex::from(0);
		let kitty_id_1 = <Test as Config>::KittyIndex::from(1);

		assert_ok!(KittiesModule::breed(owner, kitty_id_0, kitty_id_1));
	})
}

#[test]
fn breed_same_kitty_id_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let owner = Origin::signed(1);

		let kitty_id = <Test as Config>::KittyIndex::from(0);

		assert_err!(
			KittiesModule::breed(owner, kitty_id.clone(), kitty_id),
			Error::<Test>::SameKittyId
		);
	})
}

#[test]
fn breed_first_kitty_not_exists_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let owner = Origin::signed(1);

		let kitty_id = <Test as Config>::KittyIndex::from(1);
		let kitty_id_1 = <Test as Config>::KittyIndex::from(0);

		assert_ok!(KittiesModule::create(owner.clone()));
		assert_eq!(KittiesModule::kitty_owner(kitty_id_1), Some(1));
		assert!(KittiesModule::kitty_owner(kitty_id).is_none());

		assert_err!(
			KittiesModule::breed(owner, kitty_id, kitty_id_1),
			Error::<Test>::KittyNotExists
		);
	})
}

#[test]
fn breed_second_kitty_not_exists_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let owner = Origin::signed(1);

		let kitty_id = <Test as Config>::KittyIndex::from(0);
		let kitty_id_1 = <Test as Config>::KittyIndex::from(1);

		assert_ok!(KittiesModule::create(owner.clone()));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(1));
		assert!(KittiesModule::kitty_owner(kitty_id_1).is_none());

		assert_err!(
			KittiesModule::breed(owner, kitty_id, kitty_id_1),
			Error::<Test>::KittyNotExists
		);
	})
}

#[test]
fn breed_invalid_kitty_index_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let limit: u32 = <Test as Config>::MaxOwnedKitties::get();
		let total_limit = <Test as Config>::KittyIndex::max_value();
		let mut j = 0;

		loop {
			let owner = Origin::signed(j);
			if (j + 1) * limit > total_limit {
				break;
			}
			for _ in 0..limit {
				assert_ok!(KittiesModule::create(owner.clone()));
			}
			j += 1;
		}

		if j * limit < total_limit {
			for _ in 0..total_limit - j * limit {
				let owner = Origin::signed(j + 1);
				assert_ok!(KittiesModule::create(owner.clone()));
			}
		}

		let owner = Origin::signed(0);
		let kitty_id_0 = <Test as Config>::KittyIndex::from(0);
		let kitty_id_1 = <Test as Config>::KittyIndex::from(1);

		assert_err!(
			KittiesModule::breed(owner, kitty_id_0, kitty_id_1),
			Error::<Test>::InvalidKittyIndex
		);
	})
}

#[test]
fn breed_too_many_kitties_err() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
	})
}
