#![cfg(feature = "nightly")]
#![feature(custom_test_frameworks)]
#![test_runner(mimicaw_framework::test_runner)]

#[mimicaw_framework::test]
async fn foo() {}

#[mimicaw_framework::test]
async fn bar() {
    panic!("explicit panic");
}
