# Into inner drop

Safe helper for types that should implement both `Drop` and `.into_inner()` method.

## About

There's a repeating pattern where people want to have a type that does something on drop,
but also want to be able to avoid dropping and destructure the type using some method that
takes `self`.

Furhter, sometimes people want to destructure the type in `drop` implementation, which isn't
safe due to it being passed by mutable refernece.

Hand-rolling `unsafe` code was neccessary until this crate existed. This crate takes the
responsibility for ensuring that the drop impl is sound. More eyes, less bugs.

This crate is `no_std`.

## Example

Let's say you want to have a special type that prints a string on drop, but with ability to
take out the string without printing. This is how you would approach it using this crate:

```rust
// Not strictly neccessary, but helps encapsulate the inner representation.
mod inner {
    // In fact, you could even avoid this newtype!
    // But in case you have something more complicated, you might need a newtype.
    pub(super) struct PrintOnDrop(pub(super) String);

    // You need this helper type to implement Drop.
    pub(super) enum PrintOnDropImpl {}

    // Drop is implemented a bit differently
    impl into_inner_drop::DetachedDrop for PrintOnDropImpl {
        // This type will be passed to your drop function by value (move).
        type Implementor = PrintOnDrop;

        // The drop implementation. The main difference is passing inner representation by-value.
        fn drop(value: Self::Implementor) {
            // You can destructucutre your type here if you want!
            // E.g. let string = value.0;
            println!("Dropping: {}", value.0);
        }
    }
}

use into_inner_drop::IntoInnerHelper;

// Public representation

/// A String that is printed when dropped.
pub struct PrintOnDrop(IntoInnerHelper<inner::PrintOnDrop, inner::PrintOnDropImpl>);

impl PrintOnDrop {
    /// Crates a string that is printed on drop.
    fn new(string: String) -> Self {
        PrintOnDrop(IntoInnerHelper::new(inner::PrintOnDrop(string)))
    }

    /// Takes out the string, preventing printing on drop.
    fn into_string(self) -> String {
        self.0.into_inner().0
    }
}

fn main() {
    let print_on_drop = PrintOnDrop::new("Hello world!".to_owned());
    let dont_print_on_drop = PrintOnDrop::new("Hello Rustceans!".to_owned());

    let string = dont_print_on_drop.into_string();
    println!("NOT on drop: {}", string);
}

```

As you can see, the code has some boilerplate, but no `unsafe`. I'm already trying to come up
with a macro to make it much easier. See the appropriate issue on GitHub to participate.

## License

MITNFA
