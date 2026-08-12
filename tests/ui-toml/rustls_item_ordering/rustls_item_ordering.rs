#![warn(clippy::rustls_item_ordering)]

// Configured as a common trait by this directory's `clippy.toml`, so its
// implementations rank last.
pub trait Bombolone {
    fn bombolone(&self);
}

// `core::fmt::Debug` is not in the configured list, so it is only a specific
// trait here and is allowed to precede the common `Bombolone` impl. Under the
// default configuration `Debug` is a common trait and this ordering lints.
pub struct Canele;

impl core::fmt::Debug for Canele {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Canele")
    }
}

impl Bombolone for Canele {
    fn bombolone(&self) {}
}

// The opposite ordering lints here, because the locally defined `Bombolone` is
// the common trait and `Debug` the specific one. Under the default
// configuration this ordering is accepted.
pub struct Dobos;

impl Bombolone for Dobos {
    fn bombolone(&self) {}
}

impl core::fmt::Debug for Dobos {
    //~^ rustls_item_ordering
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Dobos")
    }
}

// A trait kept from the default list still ranks as common, so an inherent impl
// may not follow it.
pub struct Praline;

impl Default for Praline {
    fn default() -> Self {
        Self
    }
}

impl Praline {
    //~^ rustls_item_ordering
    pub fn praline(&self) {}
}

fn main() {}
