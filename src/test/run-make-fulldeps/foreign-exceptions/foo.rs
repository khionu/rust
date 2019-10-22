// Tests that C++ exceptions can unwind through Rust code, run destructors and
// are ignored by catch_unwind. Also tests that Rust panics can unwind through
// C++ code.

#![feature(unwind_attributes)]

use std::panic::{catch_unwind, AssertUnwindSafe};

struct DropCheck<'a>(&'a mut bool);
impl<'a> Drop for DropCheck<'a> {
    fn drop(&mut self) {
        println!("DropCheck::drop");
        *self.0 = true;
    }
}

extern "C" {
    fn throw_cxx_exception();

    #[unwind(allowed)]
    fn cxx_catch_callback(cb: extern "C" fn(), ok: *mut bool);
}

#[no_mangle]
extern "C" fn rust_catch_callback(cb: extern "C" fn(), rust_ok: &mut bool) {
    let _caught_unwind = catch_unwind(AssertUnwindSafe(|| {
        let _drop = DropCheck(rust_ok);
        cb();
        unreachable!("should have unwound instead of returned");
    }));
    unreachable!("catch_unwind should not have caught foreign exception");
}

fn throw_rust_panic() {
    extern "C" fn callback() {
        panic!("throwing rust panic");
    }

    let mut dropped = false;
    let mut cxx_ok = false;
    let caught_unwind = catch_unwind(AssertUnwindSafe(|| {
        let _drop = DropCheck(&mut dropped);
        unsafe {
            cxx_catch_callback(callback, &mut cxx_ok);
        }
        unreachable!("should have unwound instead of returned");
    }))
    .is_err();
    assert!(dropped);
    assert!(caught_unwind);
    assert!(cxx_ok);
    println!("caught rust panic");
}

fn main() {
    unsafe { throw_cxx_exception() };
    throw_rust_panic();
}
