//! Safe helper for types that should implement both `Drop` and `.into_inner()` method.
//!
//! There's a repeating pattern where people want to have a type that does something on drop,
//! but also want to be able to avoid dropping and destructure the type using some method that
//! takes `self`.
//!
//! Furhter, sometimes people want to destructure the type in `drop` implementation, which isn't
//! safe due to it being passed by mutable refernece.
//!
//! Hand-rolling `unsafe` code was neccessary until this crate existed. This crate takes the
//! responsibility for ensuring that the drop impl is sound. More eyes, less bugs. The performace
//! overhead of this crate is zero.
//!
//! This crate is `no_std`.
//!
//! # Example
//!
//! Let's say you want to have a special type that prints a string on drop, but with ability to
//! take out the string without printing. This is how you would approach it using this crate:
//!
//! ```
//! // Not strictly neccessary, but helps encapsulate the inner representation.
//! mod inner {
//!     // In fact, you could even avoid this newtype!
//!     // But in case you have something more complicated, you might need a newtype.
//!     pub(super) struct PrintOnDrop(pub(super) String);
//!
//!     // You need this helper type to implement Drop.
//!     pub(super) enum PrintOnDropImpl {}
//!
//!     // Drop is implemented a bit differently
//!     impl into_inner_drop::DetachedDrop for PrintOnDropImpl {
//!         // This type will be passed to your drop function by value (move).
//!         type Implementor = PrintOnDrop;
//!
//!         // The drop implementation. The main difference is passing inner representation by-value.
//!         fn drop(value: Self::Implementor) {
//!             // You can destructucutre your type here if you want!
//!             // E.g. let string = value.0;
//!             println!("Dropping: {}", value.0);
//!         }
//!     }
//! }
//!
//! use into_inner_drop::IntoInnerHelper;
//!
//! // Public representation
//!
//! /// A String that is printed when dropped.
//! pub struct PrintOnDrop(IntoInnerHelper<inner::PrintOnDrop, inner::PrintOnDropImpl>);
//!
//! impl PrintOnDrop {
//!     /// Crates a string that is printed on drop.
//!     fn new(string: String) -> Self {
//!         PrintOnDrop(IntoInnerHelper::new(inner::PrintOnDrop(string)))
//!     }
//!
//!     /// Takes out the string, preventing printing on drop.
//!     fn into_string(self) -> String {
//!         self.0.into_inner().0
//!     }
//! }
//!
//! fn main() {
//!     let print_on_drop = PrintOnDrop::new("Hello world!".to_owned());
//!     let dont_print_on_drop = PrintOnDrop::new("Hello Rustceans!".to_owned());
//!
//!     let string = dont_print_on_drop.into_string();
//!     println!("NOT on drop: {}", string);
//! }
//!
//! ```
//!
//! As you can see, the code has some boilerplate, but no `unsafe`. I'm already trying to come up
//! with a macro to make it much easier. See the appropriate issue on GitHub to participate.

#![no_std]

use core::mem::ManuallyDrop;

/// A replacement trait for providing Drop implementation.
///
/// Since `self` is not used, it's recommended to create an empty enum and implement this trait for
/// it.
pub trait DetachedDrop {
    /// The inner type you want to implement Drop for.
    type Implementor;

    /// The drop implementation called by `IntoInnerHelper<Self::Implementor, Self>`.
    ///
    /// This function will only be called if `into_inner` was NOT called.
    fn drop(value: Self::Implementor);
}

/// The helper which allows you to implement `Drop` for your type while still allowing to take it
/// apart by moving out.
pub struct IntoInnerHelper<T, D> where D: DetachedDrop<Implementor=T> {
    inner: ManuallyDrop<T>,
    _phantom: core::marker::PhantomData<D>,
}

impl<T, D> IntoInnerHelper<T, D> where D: DetachedDrop<Implementor=T> {
    /// Creates the helper.
    pub fn new(inner: T) -> Self {
        IntoInnerHelper {
            inner: ManuallyDrop::new(inner),
            _phantom: Default::default(),
        }
    }

    /// Accesses the inner value.
    pub fn inner(&self) -> &T {
        &*self.inner
    }

    /// Accesses the inner value mutably.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut *self.inner
    }

    /// Moves out the inner value.
    pub fn into_inner(self) -> T {
        unsafe {
            let inner = core::ptr::read(&*self.inner);
            core::mem::forget(self);
            inner
        }
    }
}

impl<T, D> Drop for IntoInnerHelper<T, D> where D: DetachedDrop<Implementor=T> {
    fn drop(&mut self) {
        unsafe {
            D::drop(core::ptr::read(&*self.inner));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn drop_once() {
        use super::{IntoInnerHelper, DetachedDrop};

        enum Dummy {}

        impl DetachedDrop for Dummy {
            type Implementor = dropcheck::DropToken;

            fn drop(_: Self::Implementor) {}
        }

        let check = dropcheck::DropCheck::new();
        let (drop_token, drop_state) = check.pair();
        let helper = <IntoInnerHelper<_, Dummy>>::new(drop_token);
        assert!(drop_state.is_not_dropped());
        core::mem::drop(helper);
        assert!(drop_state.is_dropped());
    }

    #[test]
    fn into_inner() {
        use super::{IntoInnerHelper, DetachedDrop};

        enum Dummy {}

        impl DetachedDrop for Dummy {
            type Implementor = dropcheck::DropToken;

            fn drop(_: Self::Implementor) {}
        }

        let check = dropcheck::DropCheck::new();
        let (drop_token, drop_state) = check.pair();
        let helper = <IntoInnerHelper<_, Dummy>>::new(drop_token);
        assert!(drop_state.is_not_dropped());
        let inner = helper.into_inner();
        assert!(drop_state.is_not_dropped());
        core::mem::drop(inner);
        assert!(drop_state.is_dropped());
    }
}
