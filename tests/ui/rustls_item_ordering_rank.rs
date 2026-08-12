#![warn(clippy::rustls_item_ordering)]

//! Exhaustive coverage of the rank ordering enforced by
//! `clippy::rustls_item_ordering`:
//!
//! 1. the type definition (`struct`, `enum` or `union`)
//! 2. inherent `impl` blocks
//! 3. specific trait `impl` blocks
//! 4. common trait `impl` blocks (`Clone`, `Debug`, `Drop`, ... by default)
//!
//! Within each of 2 to 4, an `impl` written on the type itself precedes one
//! written on a wrapper of it, such as `&Type` or `Box<Type>`. The cases at the
//! bottom of this file cover that tiebreak.
//!
//! Ordering is not enforced between two items of the same rank, and items are
//! grouped per type, so two correctly ordered types may be interleaved.
//!
//! `Not`, `Neg` and friends stand in for "specific" traits here, as they are
//! absent from the default `rustls-common-traits` list.

use std::hash::{Hash, Hasher};
use std::{fmt, ops};

// Every rank, in order, with several `impl` blocks of each rank. Ordering
// within a rank is arbitrary, so none of this may lint.

pub struct Baklava;

impl Baklava {
    pub fn baklava() {}
}

impl Baklava {
    pub fn madeleine() {}
}

impl Baklava {
    pub fn macaron() {}
}

impl ops::Not for Baklava {
    type Output = Self;

    fn not(self) -> Self {
        self
    }
}

impl ops::Neg for Baklava {
    type Output = Self;

    fn neg(self) -> Self {
        self
    }
}

impl Clone for Baklava {
    fn clone(&self) -> Self {
        Self
    }
}

impl fmt::Debug for Baklava {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Baklava")
    }
}

impl Hash for Baklava {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

// An inherent `impl` block after a common trait `impl` block.

pub struct Macaron;

impl Clone for Macaron {
    fn clone(&self) -> Self {
        Self
    }
}

impl Macaron {
    //~^ rustls_item_ordering
    pub fn macaron() {}
}

// A specific trait `impl` block after a common trait `impl` block.

pub struct Profiterole;

impl fmt::Display for Profiterole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Profiterole")
    }
}

impl ops::Not for Profiterole {
    //~^ rustls_item_ordering
    type Output = Self;

    fn not(self) -> Self {
        self
    }
}

// A `union` definition after an inherent `impl` block.

impl Stroopwafel {
    pub fn stroopwafel() {}
}

pub union Stroopwafel {
    //~^ rustls_item_ordering
    macaron: u32,
    madeleine: f32,
}

// A type definition after a specific trait `impl` block.

impl ops::Not for Financier {
    type Output = Self;

    fn not(self) -> Self {
        self
    }
}

pub struct Financier;
//~^ rustls_item_ordering

// A type definition after a common trait `impl` block.

impl fmt::Debug for Madeleine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Madeleine")
    }
}

pub struct Madeleine;
//~^ rustls_item_ordering

// Several `impl` blocks of each rank, badly interleaved. Both misplaced blocks
// are reported against the `Debug` block, as a misplaced item does not become
// the item that later ones are compared against.

pub enum Clafoutis {
    Baklava,
    Madeleine,
}

impl Clafoutis {
    pub fn clafoutis() {}
}

impl ops::Not for Clafoutis {
    type Output = Self;

    fn not(self) -> Self {
        self
    }
}

impl fmt::Debug for Clafoutis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Clafoutis")
    }
}

impl ops::Neg for Clafoutis {
    //~^ rustls_item_ordering
    type Output = Self;

    fn neg(self) -> Self {
        self
    }
}

impl Clafoutis {
    //~^ rustls_item_ordering
    pub fn cannoli() {}
}

impl Drop for Clafoutis {
    fn drop(&mut self) {}
}

// Three interleaved types: an `enum` and a `union` that are correctly ordered,
// and a `struct` whose inherent `impl` block trails its specific trait `impl`
// block. Only the `struct` may lint.

pub enum Cannoli {
    Macaron,
    Profiterole,
}

pub union Panettone {
    baklava: u32,
    tiramisu: f32,
}

pub struct Tiramisu;

impl Cannoli {
    pub fn cannoli() {}
}

impl Panettone {
    pub fn panettone() {}
}

impl ops::Not for Cannoli {
    type Output = Self;

    fn not(self) -> Self {
        self
    }
}

impl ops::Not for Tiramisu {
    type Output = Self;

    fn not(self) -> Self {
        self
    }
}

impl fmt::Debug for Panettone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Panettone")
    }
}

impl Tiramisu {
    //~^ rustls_item_ordering
    pub fn tiramisu() {}
}

impl fmt::Debug for Cannoli {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Cannoli")
    }
}

// Ranks are enforced inside a `mod` just as they are at the crate root.

mod baklava {
    pub struct Stroopwafel;

    impl Clone for Stroopwafel {
        fn clone(&self) -> Self {
            Self
        }
    }

    impl Stroopwafel {
        //~^ rustls_item_ordering
        // An inherent `impl` block after a common trait `impl` block.
        pub fn stroopwafel() {}
    }
}

// Within each rank, an impl written on the type itself precedes one written on
// a wrapper of it. A fully correct sequence, with the local trait declared
// below the group so that top-down ordering is satisfied too.
//
// Note two of the ranks cannot be written in Rust at all: an inherent impl on a
// wrapper (`impl Box<Battenberg>`) is not permitted for a foreign type, and a
// common trait impl on a wrapper (`impl Clone for Box<Battenberg>`) is barred by
// the orphan rules. The latter is only reachable when a local trait is named by
// `rustls-common-traits`.
pub mod battenberg {
    pub struct Battenberg;

    impl Battenberg {
        pub fn marzipan() {}
    }

    impl Lamington for Battenberg {}

    // Both wrappers share a rank, so their relative order is not enforced.
    impl Lamington for &Battenberg {}

    impl Lamington for Box<Battenberg> {}

    impl Clone for Battenberg {
        fn clone(&self) -> Self {
            Self
        }
    }

    pub trait Lamington {}
}

// A specific trait impl on the type must precede one on a wrapper of it.
pub mod knafeh {
    pub struct Knafeh;

    impl Basbousa for Box<Knafeh> {}

    impl Basbousa for Knafeh {}
    //~^ rustls_item_ordering

    pub trait Basbousa {}
}

// An inherent impl must precede a specific trait impl on a wrapper.
pub mod bostock {
    pub struct Bostock;

    impl Tarte for &Bostock {}

    impl Bostock {
        //~^ rustls_item_ordering
        pub fn glaze() {}
    }

    pub trait Tarte {}
}

// A specific trait impl on a wrapper still outranks a common trait impl on the
// type itself, since wrapping is only a tiebreak within each rank.
pub mod sfogliatella {
    pub struct Sfogliatella;

    impl Clone for Sfogliatella {
        fn clone(&self) -> Self {
            Self
        }
    }

    impl Cronut for Box<Sfogliatella> {}
    //~^ rustls_item_ordering

    pub trait Cronut {}
}

fn main() {}
