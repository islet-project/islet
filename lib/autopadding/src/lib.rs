#![no_std]
pub extern crate paste;

// Unfortunately, [$t: ty; $size: expr] does not matches with array type...
// instead, matches all integer primitves here.
#[macro_export]
macro_rules! type_check_and_init {
    (u8) => {
        0
    };
    (i8) => {
        0
    };
    (u16) => {
        0
    };
    (i16) => {
        0
    };
    (u32) => {
        0
    };
    (i32) => {
        0
    };
    (u64) => {
        0
    };
    (i64) => {
        0
    };
    (u128) => {
        0
    };
    (i128) => {
        0
    };
    (usize) => {
        0
    };
    (isize) => {
        0
    };
    ($t:ty) => {
        [0; <$t>::LEN]
    };
}

#[macro_export]
macro_rules! pad_field_and_impl_default {
    // entry point.
    (@root $(#[$attr_struct:meta])* $vis:vis $name:ident { $($input:tt)* } ) => {
        pad_field_and_impl_default!(
            @munch (
                $($input)*
            ) -> {
                $vis struct $(#[$attr_struct])* $name
            }
        );
    };

    // TODO: Remove the zero-sized paddings added where no padding is required
    (@guard ($current_offset:expr) -> {$(#[$attr:meta])* $vis:vis struct $name:ident $(($amount:expr, $vis_field:vis $id:ident: $ty:ty))*}) =>    {

        $crate::paste::paste! {
            #[repr(C)]
            #[derive(Clone, Copy)]
            $(#[$attr])* $vis struct $name { $($vis_field $id: $ty, [<_pad $id>]: [u8;$amount]),* }
        }

        $crate::paste::paste!{
            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $($id: type_check_and_init!($ty), [<_pad $id>]: [0;$amount]),*
                    }
                }
            }
        }
    };

    // Print the struct once all fields have been munched.
    (@munch
        (
            $(#[$attr:meta])*
            $offset_start:literal $vis:vis $field:ident: $ty:ty,
            $(#[$attr_next:meta])*
            $offset_end:literal => @END,
        )
        -> {$($output:tt)*}
    ) => {
        pad_field_and_impl_default!(
            @guard (
                0
            ) -> {
                $($output)*
                ($offset_end - $offset_start - core::mem::size_of::<$ty>(), $vis $field: $ty)
            }
        );
    };

    // Munch padding.
    (@munch
        (
            $(#[$attr:meta])*
            $offset_start:literal $vis:vis $field:ident: $ty:ty,
            $(#[$attr_next:meta])*
            $offset_end:literal $vis_next:vis $field_next:ident: $ty_next:ty,
            $($after:tt)*
        )
        -> {$($output:tt)*}
    ) => {
        pad_field_and_impl_default!(
            @munch (
                $(#[$attr_next])*
                $offset_end $vis_next $field_next: $ty_next,
                $($after)*
            ) -> {
                $($output)*
                ($offset_end - $offset_start - core::mem::size_of::<$ty>(), $vis $field: $ty)
            }
        );
    };
}

#[macro_export]
macro_rules! pad_struct_and_impl_default {
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident {$($fields:tt)*}
    ) => {
        $crate::pad_field_and_impl_default!(@root $(#[$attr])* $vis $name { $($fields)* } );
    };
}

pub trait ArrayLength {
    const LEN: usize;
}

impl<T, const LENGTH: usize> ArrayLength for [T; LENGTH] {
    const LEN: usize = LENGTH;
}
