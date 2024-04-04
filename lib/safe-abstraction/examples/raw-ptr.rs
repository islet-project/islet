use safe_abstraction::raw_ptr::{assume_safe, RawPtr, SafetyAssured, SafetyChecked};

struct MyStruct {
    data: i32,
}

impl MyStruct {
    fn new() -> Self {
        Self { data: 0 }
    }

    fn set(&mut self, data: i32) {
        self.data = data;
    }

    fn print(&self) {
        println!("Data: {:X}", self.data);
    }
}

/*
 *  To apply Safe Abstraction, three traits must be implemented.
 *
 *  These traits serve to either veify or proof that the safety rules are checked.
 */

const ASSUMED_SAFE_BY_DEVELOPER: bool = true;

impl RawPtr for MyStruct {}

impl SafetyChecked for MyStruct {}

// Assume that the developer has assured it.
impl SafetyAssured for MyStruct {
    fn is_initialized(&self) -> bool {
        ASSUMED_SAFE_BY_DEVELOPER
    }

    fn verify_ownership(&self) -> bool {
        ASSUMED_SAFE_BY_DEVELOPER
    }
}

fn mock_get_addr_of_instance_from_external() -> usize {
    let object = Box::new(MyStruct::new());
    let ptr = Box::into_raw(object);
    ptr as usize
}

fn without_safe_abstraction() {
    // This approach works, but consider the potential side effects
    // if the implementation of the mock function is not verifiable,
    // or if the developer uses `unsafe` solely for functionality
    // without due consideration  for Memory Safety.
    //
    // Imagine the consequences of neglecting Memory Safety in pursuit of mere operation.
    let addr = mock_get_addr_of_instance_from_external();

    unsafe {
        let raw_ptr = &mut *(addr as *mut MyStruct);
        raw_ptr.set(0xABC);
        raw_ptr.print();
    }
}

fn with_safe_abstraction() {
    // We can apply Safe Abstraction for accessing instances
    // that have been checked and assured by three traits.
    // This approach encapsulates unsafe code
    // but still allows for analysis of the behavior at the MIR stage.
    //
    // Additioanlly, in client crates, the `#![forbid(unsafe_code)]` attribute can be used
    // to prohibit the use of unsafe code.
    let addr = mock_get_addr_of_instance_from_external();

    let safety_assumed = assume_safe::<MyStruct>(addr).expect("Memory Safety Violation!");
    safety_assumed.mut_with(|my_struct: &mut MyStruct| {
        my_struct.set(0xDEF);
        my_struct.print();
    });
}

fn main() {
    without_safe_abstraction();
    with_safe_abstraction();
}
