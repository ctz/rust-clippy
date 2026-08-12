#![warn(clippy::rustls_item_ordering)]
#![allow(clippy::new_without_default, clippy::result_unit_err)]

// A fully correct inherent `impl` block: associated function, then constructors
// ordered by argument count, then public `&mut self`, public `&self`, private
// `&mut self`, private `&self`, and finally the `const` values.
pub mod trifle {
    pub struct Trifle {
        pub custard: u8,
    }

    impl Trifle {
        pub fn sponge() {}

        pub fn new() -> Self {
            Self { custard: 0 }
        }

        pub fn with_custard(custard: u8) -> Self {
            Self { custard }
        }

        pub fn try_layered(custard: u8, sherry: u8) -> Result<Self, ()> {
            let _ = sherry;
            Ok(Self { custard })
        }

        pub fn soak(&mut self) {}

        pub fn custard(&self) -> u8 {
            self.custard
        }

        fn whisk(&mut self) {}

        fn is_set(&self) -> bool {
            true
        }

        pub const SHERRY: u8 = 1;
    }
}

// A `const` value placed before the constructor.
pub mod syllabub {
    pub struct Syllabub;

    impl Syllabub {
        pub const CREAM: u8 = 1;

        pub fn new() -> Self {
            //~^ rustls_item_ordering
            Self
        }
    }
}

// Public API before a constructor.
pub mod posset {
    pub struct Posset;

    impl Posset {
        pub fn curdle(&self) {}

        pub fn new() -> Self {
            //~^ rustls_item_ordering
            Self
        }
    }
}

// Public `&self` before public `&mut self`.
pub mod fudge {
    pub struct Fudge;

    impl Fudge {
        pub fn sweetness(&self) -> u8 {
            0
        }

        pub fn stir(&mut self) {}
        //~^ rustls_item_ordering
    }
}

// Private API before public API.
pub mod brittle {
    pub struct Brittle;

    impl Brittle {
        fn shatter(&self) {}

        pub fn snap(&self) {}
        //~^ rustls_item_ordering
    }
}

// Private `&self` before private `&mut self`.
pub mod toffee {
    pub struct Toffee;

    impl Toffee {
        fn pulled(&self) {}

        fn pull(&mut self) {}
        //~^ rustls_item_ordering
    }
}

// Constructors must be ordered by argument count, fewest first. The constructor
// returning `Result<Self, _>` is recognised as a constructor too.
pub mod parfait {
    pub struct Parfait;

    impl Parfait {
        pub fn with_cream(cream: u8, sugar: u8) -> Self {
            let _ = (cream, sugar);
            Self
        }

        pub fn new() -> Self {
            //~^ rustls_item_ordering
            Self
        }
    }
}

// An associated function that is not a constructor, because it does not produce
// the type being implemented. It must precede the constructors.
pub mod nougatine {
    pub struct Nougatine;

    impl Nougatine {
        pub fn new() -> Self {
            Self
        }

        pub fn roast_temperature() -> u16 {
            //~^ rustls_item_ordering
            160
        }
    }
}

// Associated types have no defined position and are skipped entirely, so they
// neither lint nor break the ordering of the items around them.
pub mod semolina {
    pub struct Semolina;

    impl Semolina {
        pub fn new() -> Self {
            Self
        }

        pub fn serve(&self) {}
    }

    pub trait Pudding {
        type Bowl;
    }

    impl Pudding for Semolina {
        type Bowl = ();
    }
}

// Trait `impl` blocks follow the trait's own ordering, so a misordered one is
// not checked by this rule.
pub mod treacle {
    pub struct Treacle;

    pub trait Sticky {
        const TACK: u8;
        fn spread(&self);
    }

    impl Sticky for Treacle {
        const TACK: u8 = 1;
        fn spread(&self) {}
    }
}

// A `pub fn` on a type in a private module is not part of the crate's public
// API, so it ranks as private. Both methods here are private, meaning `&self`
// after `&mut self` is correct and does not lint.
mod praline {
    pub struct Praline;

    impl Praline {
        pub fn caramelise(&mut self) {}

        pub fn nuts(&self) -> u8 {
            0
        }
    }
}

fn main() {}
