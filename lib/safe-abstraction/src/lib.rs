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
//!   allowing for lower-level operations like memory access to be performed more safely.
//!
//! - **Runtime Safety Checks**: Provides methods to perform crucial safety checks at runtime,
//!   such as verifying if a pointer is null and checking whether a pointer is properly aligned.
//!   These checks happen when the methods are called during the execution of a program.
//!
//! - **Compile-Time Type Safety Checks**: Enforces certain safety guarantees at compile time.
//!   For example, the use of Rust's type system can ensure that only pointers
//!   to types with known sizes are used, leveraging the `Sized` trait bound.
//!
//! - **Developer-Driven Safety Verification**: Introduces traits that allow developers
//!   to explicitly mark parts of `unsafe` code that still require manual safety guarantees,
//!   making it clear which parts of the code need careful review.

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
    //!   pointed to by the raw pointer is determined at compile time,
    //!   enabling safe memory operations.
    //!
    //! - `SafetyChecked`: Implements checks to ensure that the raw pointer
    //!   is not null and properly aligned.
    //!
    //! - `SafetyAssured`: Provides guarantees that the instance
    //!   pointed to by the raw pointer is properly initialized,
    //!   adheres to Rust's ownership rules.

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

    /// Enumerates the types of errors that can occur in the `assume_safe` function.
    #[derive(Debug)]
    pub enum Error {
        /// Indicates a failure in safety checks (SafetyChecked trait).
        SafetyCheckFailed,

        /// Indicates a failure in assurance checks (SafetyAssured trait).
        AssuranceCheckFailed,
    }

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match *self {
                Error::SafetyCheckFailed => write!(f, "Safety check failed"),
                Error::AssuranceCheckFailed => write!(f, "Assurance check failed"),
            }
        }
    }

    impl core::error::Error for Error {}

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
    /// Returns `Ok(SafetyAssumed)` if all safety checks are satisfied, or `Error`
    pub fn assume_safe<T: SafetyChecked + SafetyAssured>(
        addr: usize,
    ) -> Result<SafetyAssumed<T>, Error> {
        let ptr = addr as *const T;
        // Safety: This cast from a raw pointer to a reference is considered safe
        //         because it is used solely for the purpose of verifying alignment and range,
        //         without actually dereferencing the pointer.
        let ref_ = unsafe { &*(ptr) };

        if !ref_.is_not_null() || !ref_.is_aligned() {
            return Err(Error::SafetyCheckFailed);
        }

        if !ref_.is_initialized() || !ref_.verify_ownership() {
            return Err(Error::AssuranceCheckFailed);
        }

        Ok(SafetyAssumed {
            addr,
            assume_init: false,
            _phantom: core::marker::PhantomData,
        })
    }

    pub fn assume_safe_uninit_with<T: SafetyChecked + SafetyAssured>(
        addr: usize,
        value: T,
    ) -> Result<SafetyAssumed<T>, Error> {
        let ptr = addr as *const T;
        // Safety: This cast from a raw pointer to a reference is considered safe
        //         because it is used solely for the purpose of verifying alignment and range,
        //         without actually dereferencing the pointer.
        let ref_ = unsafe { &*(ptr) };

        if !ref_.is_not_null() || !ref_.is_aligned() {
            return Err(Error::SafetyCheckFailed);
        }

        if !ref_.verify_ownership() {
            return Err(Error::AssuranceCheckFailed);
        }

        Ok(SafetyAssumed::new_maybe_uninit_with(addr, value))
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
    /// * `_phantom` - A `PhantomData` used to associate generic type `T` with this struct
    ///                without storing any data of type `T`. This helps manage type invariance
    ///                and ensure that the Rust compiler accounts for `T` in its type checking.
    pub struct SafetyAssumed<T: SafetyChecked + SafetyAssured> {
        addr: usize,
        assume_init: bool,
        _phantom: core::marker::PhantomData<T>,
    }

    impl<T> SafetyAssumed<T>
    where
        T: SafetyChecked + SafetyAssured,
    {
        pub fn new_maybe_uninit_with(addr: usize, value: T) -> Self {
            unsafe {
                let src: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::new(value);
                let src = &src as *const core::mem::MaybeUninit<T>;
                let dst = addr as *mut core::mem::MaybeUninit<T>;
                core::ptr::copy_nonoverlapping(src, dst, 1);
            }

            Self {
                addr,
                assume_init: true,
                _phantom: core::marker::PhantomData,
            }
        }
    }

    impl<T> AsRef<T> for SafetyAssumed<T>
    where
        T: SafetyChecked + SafetyAssured,
    {
        /// Safely returns a mutable reference to the instance of `T`.
        ///
        /// # Safety
        /// Similar to `as_ref`, this function assumes that all required safety checks
        /// are in place. Mutable access is granted under the presumption of exclusive ownership
        /// and proper synchronization when accessed in multi-threaded contexts.
        fn as_ref(&self) -> &T {
            unsafe {
                if self.assume_init {
                    let ptr = self.addr as *mut core::mem::MaybeUninit<T>;
                    (*ptr).assume_init_ref()
                } else {
                    T::as_ref(self.addr)
                }
            }
        }
    }

    impl<T> AsMut<T> for SafetyAssumed<T>
    where
        T: SafetyChecked + SafetyAssured,
    {
        /// Safely returns a mutable reference to the instance of `T`.
        ///
        /// # Safety
        /// Similar to `as_ref`, this function assumes that all required safety checks
        /// are in place. Mutable access is granted under the presumption of exclusive ownership
        /// and proper synchronization when accessed in multi-threaded contexts.
        fn as_mut(&mut self) -> &mut T {
            unsafe {
                if self.assume_init {
                    let ptr = self.addr as *mut core::mem::MaybeUninit<T>;
                    (*ptr).assume_init_mut()
                } else {
                    T::as_mut(self.addr)
                }
            }
        }
    }

    impl<T> core::ops::Deref for SafetyAssumed<T>
    where
        T: SafetyChecked + SafetyAssured,
    {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.as_ref()
        }
    }

    impl<T> core::ops::DerefMut for SafetyAssumed<T>
    where
        T: SafetyChecked + SafetyAssured,
    {
        fn deref_mut(&mut self) -> &mut T {
            self.as_mut()
        }
    }
}
