// compile-flags: --edition=2021
// run-rustfix

pub struct Struct<T> {
    pub p: T,
}

impl<T> Struct<T> {
    pub fn method(&self) {}

    pub fn some_mutable_method(&mut self) {}
}

fn main() {
    let other_item = std::cell::RefCell::new(Struct { p: 42_u32 });

    other_item.method();
    //~^ ERROR no method named `method` found for struct `RefCell` in the current scope [E0599]
    //~| HELP use `.borrow()` to borrow the Struct<u32>, panicking if any outstanding mutable borrows exist.

    other_item.some_mutable_method();
    //~^ ERROR no method named `some_mutable_method` found for struct `RefCell` in the current scope [E0599]
    //~| HELP use `.borrow_mut()` to mutably borrow the Struct<u32>, panicking if any outstanding borrows exist.

    let another_item = std::sync::Mutex::new(Struct { p: 42_u32 });

    another_item.method();
    //~^ ERROR no method named `method` found for struct `Mutex` in the current scope [E0599]
    //~| HELP use `.lock()` to borrow the Struct<u32>, blocking the current thread until it can be acquired
}