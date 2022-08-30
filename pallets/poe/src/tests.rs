use crate::{mock::*, Config, Error, Proofs};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::BoundedVec;

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim).unwrap();

		let (owner, block) = Proofs::<Test>::get(&bounded_claim).unwrap();

		assert_eq!(owner, 1);
		assert_eq!(block, frame_system::Pallet::<Test>::block_number());
	})
}

#[test]
fn create_claim_failed_when_already_exists() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

#[test]
fn create_claim_failed_when_too_long() {
	new_test_ext().execute_with(|| {
		const CLAIM_LEN: usize = 513;
		let claim = vec![0; CLAIM_LEN];
		//assert!(CLAIM_LEN > <Test as Config>::MaxClaimLength::get());

		assert_err!(PoeModule::create_claim(Origin::signed(1), claim), Error::<Test>::ClaimTooLong);
	})
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		let _ = PoeModule::revoke_claim(Origin::signed(1), claim.clone());

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim).unwrap();

		assert_eq!(Proofs::<Test>::get(&bounded_claim), None);
	})
}

#[test]
fn revoke_claim_failed_when_not_exists() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_err!(
			PoeModule::revoke_claim(Origin::signed(1), claim),
			Error::<Test>::ClaimNotExist
		);
	})
}

#[test]
fn revoke_claim_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_err!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let origin1 = Origin::signed(1);

		let _ = PoeModule::create_claim(origin1.clone(), claim.clone());

		assert_ok!(PoeModule::transfer_claim(origin1.clone(), claim.clone(), 2));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn transfer_claim_failed_when_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_err!(
			PoeModule::transfer_claim(Origin::signed(1), claim, 2),
			Error::<Test>::ClaimNotExist
		);
	})
}

#[test]
fn transfer_claim_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_err!(
			PoeModule::transfer_claim(Origin::signed(2), claim, 3),
			Error::<Test>::NotClaimOwner
		);
	})
}
