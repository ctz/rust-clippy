//@compile-flags: --cfg test

#![warn(clippy::rustls_item_ordering)]
#![allow(clippy::items_after_test_module)]

// A `#[allow]` on the offending item alone suppresses the diagnostic: the lint
// is emitted against the item's own `HirId` precisely so that this works.
pub struct Nougat;

impl Default for Nougat {
    fn default() -> Self {
        Self
    }
}

#[allow(clippy::rustls_item_ordering)]
impl Nougat {
    pub fn nougat(&self) {}
}

// Allowing the ordering on one item does not suppress an unrelated violation in
// the same module.
pub struct Pavlova;

impl Default for Pavlova {
    fn default() -> Self {
        Self
    }
}

impl Pavlova {
    //~^ rustls_item_ordering
    pub fn pavlova(&self) {}
}

// A `#[allow]` on an enclosing module suppresses the items within it. The
// `dobos` module below is the unsuppressed control: it is structurally
// identical and does lint.
#[allow(clippy::rustls_item_ordering)]
mod canele {
    pub struct Canele;

    impl Default for Canele {
        fn default() -> Self {
            Self
        }
    }

    impl Canele {
        pub fn canele(&self) {}
    }
}

mod dobos {
    pub struct Dobos;

    impl Default for Dobos {
        fn default() -> Self {
            Self
        }
    }

    impl Dobos {
        //~^ rustls_item_ordering
        pub fn dobos(&self) {}
    }
}

// An item carrying `#[cfg(test)]` is skipped, so the inherent impl below is not
// ordered against it. Without the skip this would lint, as `Default` is a
// common trait. The file is compiled with `--cfg test` so the `#[cfg(test)]`
// items are really present in the HIR.
pub struct Sorbet;

#[cfg(test)]
impl Default for Sorbet {
    fn default() -> Self {
        Self
    }
}

impl Sorbet {
    pub fn sorbet(&self) {}
}

// `#[cfg(test)]` exempts the annotated module's contents, not just the module
// item itself, so neither of the misorderings below is reported. Both shapes
// are covered: a type declared and implemented inside the test module, and
// impls written on a type declared outside it.
pub struct Bombolone;

#[cfg(test)]
mod praline {
    use super::Bombolone;

    pub struct Praline;

    impl Default for Praline {
        fn default() -> Self {
            Self
        }
    }

    impl Praline {
        pub fn praline(&self) {}
    }

    impl Default for Bombolone {
        fn default() -> Self {
            Self
        }
    }

    impl Bombolone {
        pub fn bombolone(&self) {}
    }
}

// Items expanded from a local `macro_rules!` macro are linted: only external
// macros are exempt, and the generated items have distinct spans so the
// equal-span guard does not apply either.
macro_rules! strudel {
    ($name:ident) => {
        pub struct $name;

        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }

        impl $name {
            //~^ rustls_item_ordering
            pub fn strudel(&self) {}
        }
    };
}

strudel!(Marzipan);

// `#[expect]` on a genuine violation is fulfilled by the emitted diagnostic, so
// no `unfulfilled_lint_expectations` warning is produced.
pub struct Zeppole;

impl Default for Zeppole {
    fn default() -> Self {
        Self
    }
}

#[expect(clippy::rustls_item_ordering)]
impl Zeppole {
    pub fn zeppole(&self) {}
}

fn main() {}
