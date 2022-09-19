use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::BoundedVec;

#[test]
fn it_works() {
	new_test_ext().execute_with(|| {})
}
