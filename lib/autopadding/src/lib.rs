#![no_std]
pub extern crate paste;

#[macro_export]
macro_rules! pad_field {
    // entry point.
    (@root $(#[$attr_struct:meta])* $vis:vis $name:ident { $($input:tt)* } ) => {
        pad_field!(
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
            $(#[$attr])* $vis struct $name { $($vis_field $id: $ty, [<_pad $id>]: [u8;$amount]),* }
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
        pad_field!(
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
        pad_field!(
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
macro_rules! pad_struct {
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident {$($fields:tt)*}
    ) => {
        $crate::pad_field!(@root $(#[$attr])* $vis $name { $($fields)* } );
    };
}
