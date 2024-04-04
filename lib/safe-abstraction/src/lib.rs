#![warn(rust_2018_idioms)]
#![deny(warnings)]
#![no_std]

//! # Safe Abstraction Crate
//!
//! The `safe_abstraction` crate is a library designed
//! to facilitate safer abstraction over `unsafe` code.
//!
//! Its primary goal is to enhance the safety of `unsafe` code
//! by providing data structures and functions that minimize
//! the need for direct `unsafe` code usage,
//! and by offering traits for automating
//! and explicitly marking parts of `unsafe` code
//! that require developer intervention.
//!
//! ## Features
//!
//! - **Encapsulation of Unsafe Code**: Offers a way to safely abstract `unsafe` operations,
//! allowing for lower-level operations like memory access to be performed more safely.
//!
//! - **Runtime Safety Checks**: Provides methods to perform crucial safety checks at runtime,
//! such as verifying if a pointer is null and checking whether a pointer is properly aligned.
//! These checks happen when the methods are called during the execution of a program.
//!
//! - **Compile-Time Type Safety Checks**: Enforces certain safety guarantees at compile time.
//! For example, the use of Rust's type system can ensure that only pointers
//! to types with known sizes are used, leveraging the `Sized` trait bound.
//!
//! - **Developer-Driven Safety Verification**: Introduces traits that allow developers
//! to explicitly mark parts of `unsafe` code that still require manual safety guarantees,
//! making it clear which parts of the code need careful review.

pub mod raw_ptr {
    //! # Raw Pointer Safety Abstraction Module
    //!
    //! This module provides a set of traits designed
    //! to facilitate safe abstraction over raw pointers.
    //!
    //! Raw pointers (`*const T` and `*mut T`) offer great power
    //! but come with great responsibility: they are unsafe by nature
    //! and require careful handling to ensure safety.
    //!
    //! The `raw_ptr` module introduces traits
    //! that enforce checks on raw pointers
    //! to ensure that their usage adheres
    //! to Rust’s safety guarantees.
    //!
    //! ## Traits Overview
    //! - `RawPtr`: Ensures that the size of the structure
    //! pointed to by the raw pointer is determined at compile time,
    //! enabling safe memory operations.
    //!
    //! - `SafetyChecked`: Implements checks to ensure that the raw pointer
    //! is not null and properly aligned.
    //!
    //! - `SafetyAssured`: Provides guarantees that the instance
    //! pointed to by the raw pointer is properly initialized,
    //! adheres to Rust's ownership rules.

    pub trait RawPtr: Sized {
        /// # Safety
        ///
        /// When calling this method, you have to ensure that all of the following is true:
        ///
        /// * The pointer must point to an initialized instance of `T`.
        ///
        /// * You must enforce Rust's aliasing rules
        unsafe fn as_ref<'a, T: RawPtr>(addr: usize) -> &'a T {
            &*(addr as *const T)
        }

        /// # Safety
        ///
        /// When calling this method, you have to ensure that all of the following is true:
        ///
        /// * The pointer must point to an initialized instance of `T`.
        ///
        /// * You must enforce Rust's aliasing rules
        unsafe fn as_mut<'a, T: RawPtr>(addr: usize) -> &'a mut T {
            &mut *(addr as *mut T)
        }

        fn addr(&self) -> usize {
            let ptr: *const Self = self;
            ptr as usize
        }
    }

    /// `SafetyChecked` Trait
    ///
    /// This trait signifies that certain safety checks
    /// can be automatically performed by the code itself.
    ///
    /// Implementing this trait indicates that the associated functionality
    /// has been designed to undergo automatic safety verification processes,
    /// minimizing the need for manual intervention.
    ///
    /// It is particularly useful for encapsulating operations
    /// that can be safely abstracted away from direct `unsafe` code usage.
    ///
    /// Types implementing `SafetyChecked` should ensure
    /// that all potential safety risks are either inherently
    /// mitigated by the implementation or are automatically checkable at compile or run time.
    pub trait SafetyChecked: RawPtr {
        fn is_not_null(&self) -> bool {
            let ptr: *const Self = self;
            !ptr.is_null()
        }

        fn is_aligned(&self) -> bool {
            self.addr() % core::mem::align_of::<usize>() == 0
        }
    }

    /// `SafetyAssured` Trait
    ///
    /// The `SafetyAssured` trait is intended
    /// to be used as a marker for code sections
    /// where safety cannot be automatically checked
    /// or guaranteed by the compiler or runtime environment.
    /// Instead, the safety of operations marked with this trait relies on manual checks
    /// and guarantees provided by the developer.
    ///
    /// Implementing `SafetyAssured` serves
    /// as a declaration that the developer has manually reviewed
    /// the associated operations and is confident in their safety,
    /// despite the inability to enforce these guarantees automatically.
    /// It is a commitment to adhering to Rust's safety principles
    /// while working within the necessary confines of `unsafe` code.
    pub trait SafetyAssured {
        /// Checks if the instance is properly initialized.
        ///
        /// This method should verify that all necessary initializations
        /// for the instance have been completed.
        /// For example, it should check if memory allocations have been made
        /// and if they are filled with appropriate values,
        /// or if all fields of a struct have been initialized to their expected default values.
        /// Proper initialization is crucial to prevent issues such as use of uninitialized memory.
        fn is_initialized(&self) -> bool;

        /// Checks whether ownership rules are upheld for this instance,
        /// with a particular focus on instances that originate from raw pointers.
        ///
        /// This method evaluates the adherence to Rust's ownership model,
        /// ensuring safe resource management while preventing issues
        /// like use-after-free, data races, and unauthorized mutable aliasing.
        /// A return value of `true` indicates strict compliance with these rules.
        ///
        /// However, it is crucial to return `false` under the following conditions:
        /// - The instance is derived from a raw pointer, and operates in a multi-core or multi-threaded
        ///   context without appropriate synchronization mechanisms (e.g., mutexes). This situation
        ///   significantly increases the risk of unsafe access patterns, such as data races or
        ///   simultaneous mutable aliasing, that violate Rust’s guarantees on memory safety.
        /// - Any other detected violation of ownership rules, such as incorrect lifecycle management
        ///   leading to potential dangling references or use-after-free scenarios.
        ///
        /// By mandating the use of synchronization tools in concurrent environments for instances
        /// originating from raw pointers, this function underscores the necessity of diligent safety
        /// practices to uphold Rust's safety guarantees, alerting developers to areas of concern
        /// that require attention.
        fn verify_ownership(&self) -> bool;
    }

    /// Attempts to create a `SafetyAssumed` instance from a address.
    ///
    /// This function checks both `SafetyChecked` and `SafetyAssured` traits' conditions
    /// to ensure that the target at the given address adheres to safety guarantees.
    /// If all checks pass, it returns a `SafetyAssumed` instance encapsulating the address,
    /// signifying that interactions with the target can be safely performed.
    /// Otherwise, it returns `None`, indicating that safety guarantees cannot be met.
    ///
    /// # Arguments
    ///
    /// * `addr` - The raw address of the target instance to be safely accessed.
    ///
    /// # Returns
    ///
    /// Returns `Some(SafetyAssumed)` if all safety checks are satisfied,
    /// or `None` if any of the checks fail, indicating that it is not safe to proceed.
    pub fn assume_safe<T: SafetyChecked + SafetyAssured>(addr: usize) -> Option<SafetyAssumed> {
        let ptr = addr as *const T;
        // Safety: This cast from a raw pointer to a reference is considered safe
        //         because it is used solely for the purpose of verifying alignment and range,
        //         without actually dereferencing the pointer.
        let ref_ = unsafe { &*(ptr) };
        let checked = ref_.is_not_null() && ref_.is_aligned();
        let assured = ref_.is_initialized() && ref_.verify_ownership();

        match checked && assured {
            true => Some(SafetyAssumed { addr }),
            false => None,
        }
    }

    /// Represents a target instance that has passed all necessary safety checks.
    ///
    /// An instance of `SafetyAssumed` signifies that it is safe to interact with the target
    /// through raw pointers, as all required safety conditions
    /// (checked by `SafetyChecked` and assured by `SafetyAssured`) have been met.
    /// This structure acts as a safe wrapper,
    /// allowing for controlled access to the underlying data
    /// while upholding Rust's safety guarantees.
    ///
    /// # Fields
    ///
    /// * `addr` - The raw address of the safely assumed target instance.
    pub struct SafetyAssumed {
        addr: usize,
    }

    impl SafetyAssumed {
        /// Provides safe access to a target structure
        /// by ensuring that `SafetyChecked` and `SafetyAssured` traits are implemented.
        ///
        /// # Safety
        /// This function facilitates safe interaction
        /// with structures accessed through raw pointers by leveraging
        /// the Rust's safety guarantees built upon
        /// the assumption that developers ensure the safety of `unsafe` code.
        ///
        /// # TODO: Checked the claim below by MIRI
        /// However, `unsafe` code passed through a closure,
        /// it becomes a subject for analysis at the MIR (Mid-level Intermediate Representation) stage.
        /// This allows for further security enhancements
        /// through the use of `unsafe` code analysis tools.
        ///
        /// # Caution
        /// It's important to remember that while this function aims
        /// to provide a safer interface for interacting with `unsafe` code,
        /// the inherent risks associated with `unsafe` code cannot be entirely eliminated.
        /// Developers are encouraged to use `unsafe` analysis tools
        /// to strengthen security and ensure that all
        /// safety guarantees are thoroughly verified.
        pub fn with<T, F, R>(&self, f: F) -> R
        where
            T: SafetyChecked + SafetyAssured,
            F: Fn(&T) -> R,
        {
            unsafe {
                let obj = T::as_ref(self.addr);
                f(obj)
            }
        }

        /// Provides safe mutation to a target structure
        /// by ensuring that `SafetyChecked` and `SafetyAssured` traits are implemented.
        ///
        /// # Safety
        /// This function facilitates safe interaction
        /// with structures accessed through raw pointers by leveraging
        /// the Rust's safety guarantees built upon
        /// the assumption that developers ensure the safety of `unsafe` code.
        ///
        /// # TODO: Checked the claim below by MIRI
        /// However, `unsafe` code passed through a closure,
        /// it becomes a subject for analysis at the MIR (Mid-level Intermediate Representation) stage.
        /// This allows for further security enhancements
        /// through the use of `unsafe` code analysis tools.
        ///
        /// # Caution
        /// It's important to remember that while this function aims
        /// to provide a safer interface for interacting with `unsafe` code,
        /// the inherent risks associated with `unsafe` code cannot be entirely eliminated.
        /// Developers are encouraged to use `unsafe` analysis tools
        /// to strengthen security and ensure that all
        /// safety guarantees are thoroughly verified.
        pub fn mut_with<T, F, R>(&self, mut f: F) -> R
        where
            T: SafetyChecked + SafetyAssured,
            F: FnMut(&mut T) -> R,
        {
            unsafe {
                let obj = T::as_mut(self.addr);
                f(obj)
            }
        }
    }
}
