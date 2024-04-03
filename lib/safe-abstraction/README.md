# Safe Abstraction Crate

The `safe_abstraction` crate is designed
to facilitate safer abstraction over `unsafe` code,
aiming to enhance the safety and maintainability of Rust code,
especially when dealing with `unsafe` code blocks.

## Requirements

To achieve its goals, this crate is built around the following key requirements:

1. **Reduce the Absolute Amount of Unsafe Code**:
By minimizing the usage of `unsafe` code,
we reduce the potential for safety violations and
simplify the codebase for easier auditing and maintenance.

2. **Define Code-level Safety Rules for Unsafe Code**:
We establish clear, enforceable safety rules
that can be checked at the code level,
guiding the correct and safe use of `unsafe` code.

3. **Define Developer Assurance for Unsafe Code**:
In addition to automated checks,
developers must assure certain safety guarantees
when writing `unsafe` code.
These rules demand a deep understanding of Rust's safety guarantees
and the implications of using `unsafe` code.

4. **Provide Interface for Safely Writing Unsafe Code Upon Assurance**:
With the safety rules from points 2 and 3 assured,
the crate offers an interface that enables developers to write `unsafe` code safely.
This interface acts as a bridge, maintaining Rust's safety standards
while utilizing the power of `unsafe` code.

5. **Ensure Compatibility with MIR Analysis Tools**:
The design and functionality of this crate
do not hinder the effectiveness of MIR analysis tools.
These tools play a crucial role in optimizing Rust code
and identifying potential safety violations,
and our crate is built to complement their operation,
offering developers insights into their code's safety and efficiency.


By adhering to these requirements,
the crate aims to provide Rust developers with the tools
they need to use unsafe code responsibly
while maintaining the high safety standards that Rust is known for.

## Features

- **Encapsulation of Unsafe Code**:
Provides a means to safely abstract `unsafe` operations,
enhancing safety for lower-level operations such as memory access.

- **Runtime Safety Checks**:
Offers methods to perform essential safety checks at runtime,
such as verifying non-null pointers and checking alignment,
ensuring safety during program execution.

- **Compile-Time Type Safety Checks**:
Leverages Rust's type system to enforce safety guarantees at compile time,
ensuring the use of only properly sized pointers through the `Sized` trait bound.

- **Developer-Driven Safety Assurance**:
Introduces traits that enable developers to explicitly mark
and assure parts of `unsafe` code that require manual safety guarantees,
clarifying the need for careful review.

## Key Concept

The core philosophy of this crate revolves around
three data structures and one API
that ensure safety when working with `unsafe` code.
These data structures and API orchestrate a comprehensive safety assurance process,
allowing for safer abstractions and interactions with potentially unsafe operations.
Here's an overview of how these components interact:

- **`SafetyChecked` Trait**:
Defines rules that can be checked at the code level.
These checks aim to verify the basic safety requirements directly through code analysis.

- **`SafetyAssured` Trait**:
Establishes rules that must be assured by the developer.
These are higher-level safety guarantees
that cannot be fully automated and require developer intervention to ensure compliance.

- **`SafetyAssumed` Struct**:
Provides safe abstraction, enabling the writing of code solely
with safe constructs once the safety rules have been satisfied.
This ensures that interactions with the underlying unsafe operations are safely abstracted,
maintaining the safety guarantees of Rust.

- **`assume_safe` API**:
Acts as a gateway, verifying that all defined rules are met.
If the verification process is successful,
it returns an instance of `SafetyAssumed`.
The use of "Assume" signifies that complementary verification
via formal methods or other MIR analysis tools
might be necessary for comprehensive safety assurance.

```
+-----------------+   +-----------------+
|  SafetyChecked  |   |  SafetyAssured  |
+-----------------+   +-----------------+
            |                 |
            |                 |
            V                 v
+---------------------------------------+
|           assume_safe API             |
+---------------------------------------+
                   |
                   |
                   v
          +-----------------+
          |  SafetyAssumed  |
          +-----------------+
```

By implementing the `SafetyChecked` and `SafetyAssured` traits,
developers can confidently utilize the `SafetyAssumed` structure
to safely access instances that adhere to these rules,
allowing for the safe encapsulation of `unsafe` code
while maintaining compatibility with MIR analysis tools.

## Safety Rules for Raw Pointers

The following safety rules are critical
for ensuring the safe use of raw pointers within this crate:

1. **Compile-time Size Determination**:
Guarantees that the size of the structure pointed
to by the raw pointer is determined at compile time,
enabling safe and predictable memory operations.

2. **Non-null**:
Ensures that the raw pointer is not null,
preventing potential null pointer dereferences.

3. **Proper Alignment**:
Requires that the address within the raw pointer is properly aligned,
aligning with hardware's memory access requirements.

4. **Appropriate Permissions**:
Checks that the raw pointer has suitable permissions
for the intended operation (such as read, write, execute).

5. **Initialization Assurance**:
Assures that the instance pointed
to by the raw pointer is properly initialized,
preventing undefined behavior from uninitialized memory access.

6. **Ownership Rules Compliance**:
Assures adherence to Rust's ownership rules,
safeguarding against data races and unsafe memory access.

By adhering to these safety rules
through the implementation of the `RawPtr`, `SafetyChecked`, and `SafetyAssured` traits,
developers can leverage the power of raw pointers in Rust
while ensuring safety and adhering to the language's strict safety guarantees.

Once these traits have been implemented, access to the target instance can be made through
the `SafetyAssumed` structure, allowing interaction with the instance using only safe code.
It emphasizes that by adhering to the safety rules and
utilizing the `SafetyAssumed` structure, developers can maintain safety guarantees
while interacting with potentially unsafe sources or operations.

## Examples
### Without Safe Abstraction
```rust
// This approach works, but consider the potential side effects
// if the implementation of the mock function is not verifiable,
// or if the developer uses `unsafe` solely for functionality
// without due consideration for Memory Safety.
//
// Imagine the consequences of neglecting Memory Safety in pursuit of mere operation.
let addr = mock_get_addr_of_instance_from_external();

unsafe {
    let raw_ptr = &mut *(addr as *mut MyStruct);
    raw_ptr.set(0xABC);
    raw_ptr.print();
}
```

### With Safe Abstraction
```rust
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
```

## Implementation Guide

When implementing the `SafetyChecked` and `SafetyAssured` traits,
we strongly recommend utilizing MIR analysis tools or formal verification tools
to ensure the absence of safety issues within these methods.

**Once the safety of these methods is guaranteed
by such rigorous analysis,
it's advised to make these methods always return `true`.**

This approach not only solidifies the safety guarantees of your code
but also aids in performance optimization.
When the compiler recognizes that these methods consistently return `true`,
it can perform optimizations by eliminating unnecessary checks
or branches, thereby enhancing the runtime efficiency of your application.

Therefore, after thorough verification,
set the body of these trait methods to unconditionally return `true`.
This not only assures the safety of your code
but also enables the compiler to optimize for better performance,
ensuring a high level of efficiency without compromising safety.

## In Closing

Ultimately, this crate provides a structured approach
to safely navigate and abstract `unsafe` code,
allowing developers to uphold Rust's stringent safety protocols
while leveraging the full power of the language.
By integrating safety checks, developer assurances, and seamless abstractions,
it harmonizes safety with practicality,
marking a significant stride towards safer Rust programming.
