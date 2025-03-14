use std::time::Duration;

use tarantool::fiber;
use tarantool_test::bind_test_suite;

bind_test_suite!();

#[tarantool_test::test]
fn test_nothing() {
    fiber::sleep(Duration::from_millis(200));
}
