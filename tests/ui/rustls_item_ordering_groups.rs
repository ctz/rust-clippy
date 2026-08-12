#![warn(clippy::rustls_item_ordering)]

// This file exercises *grouping*: which `impl` blocks get associated with a
// type's group at all, i.e. the boundaries of `classify` / `self_ty_def_id`.
// Ordering *within* a group is covered elsewhere; here every expected lint (and
// every expected silence) is really a statement about which group an item
// landed in.
//
// NOTE: the traits used here are declared at the top of the file, above the
// types that implement them, so several types additionally trip the top-down
// ordering rule. Those diagnostics are incidental to what this file is testing.

trait Churro {}

// A blanket impl's self type is a type parameter, so `self_ty_def_id` returns
// `None` and the impl is never part of any type's group. Not linted, whatever
// position it takes.
impl<T> Churro for T {}

trait Eclair {}

// Impls for foreign types. `Vec` and `String` resolve to `DefKind::Struct` but
// `as_local()` fails, and `u32` resolves to `Res::PrimTy`, which
// `self_ty_def_id` rejects outright. All three are ungrouped: no lints.
impl Eclair for Vec<u8> {}

impl Eclair for u32 {}

impl Eclair for String {}

trait Ladoo {}

trait Semifreddo {}

// Type def, inherent impl, common trait impl: correctly ordered so far, so no
// rank lint. `Mochi` does trip top-down twice, because its group implements
// both `Ladoo` and `Semifreddo`, which are declared above it. The `Semifreddo`
// use comes from the impl written on the `Halva` alias further down, which is
// grouped with `Mochi`.
struct Mochi;
//~^ rustls_item_ordering
//~^^ rustls_item_ordering

impl Mochi {
    fn churro() -> Self {
        Mochi
    }
}

impl Clone for Mochi {
    fn clone(&self) -> Self {
        Mochi
    }
}

// A reference is a wrapper, so `&Mochi` is grouped with `Mochi` and ranks after
// the impls written on `Mochi` itself. Landing after `Mochi`'s common trait
// impl, it is linted, and the message names it as being on a wrapping type.
impl Ladoo for &Mochi {}
//~^ rustls_item_ordering

// `Box<Mochi>` is a wrapper too, so this behaves exactly like the `&Mochi` impl
// above rather than escaping the lint. `Box` itself is foreign, so grouping has
// to look through it to the local type inside.
impl Ladoo for Box<Mochi> {}
//~^ rustls_item_ordering

// The self type is resolved rather than read off the written path, so an impl
// through an alias is an impl on the aliased type: both of these are grouped
// with `Mochi` and follow its common trait impl, so both are linted. Note the
// inherent impl is ranked as being on the type itself, not on a wrapper, since
// the alias resolves straight to `Mochi`.
//
// The alias item itself is not reported for top-down ordering even though it
// uses `Mochi` above it, because the impls below name `Halva` in turn: the two
// use each other, and a mutual pair has no satisfiable order.
type Halva = Mochi;

impl Halva {
    //~^ rustls_item_ordering
    fn gelato() -> Self {
        Mochi
    }
}

impl Semifreddo for Halva {}
//~^ rustls_item_ordering

// A generic local type. The self type path resolves to `Gelato` whatever the
// type arguments are, so every impl below is grouped under `Gelato`.
struct Gelato<T>(T);
//~^ rustls_item_ordering
//~^^ rustls_item_ordering

impl<T> Gelato<T> {
    fn ladoo(self) -> T {
        self.0
    }
}

// A constrained inherent impl has the same rank as the unconstrained one above,
// so it is not linted.
impl<T: Clone> Gelato<T> {
    fn kulfi(&self) -> T {
        self.0.clone()
    }
}

impl<T: core::fmt::Debug> Ladoo for Gelato<T> {}

impl<T: Clone> Clone for Gelato<T> {
    fn clone(&self) -> Self {
        Gelato(self.0.clone())
    }
}

// `Gelato<Mochi>` groups with `Gelato`, not with `Mochi`: the note in the
// expected output points at `Gelato`'s `Clone` impl, not at `Mochi`'s. Linted
// as an inherent impl following a common trait impl.
impl Gelato<Mochi> {
    //~^ rustls_item_ordering
    fn semifreddo() {}
}

// Two types in one module with interleaved items. Each type gets its own entry
// in the `seen` map, so interleaving is fine as long as each type's own items
// are individually in order. No rank lints; both trip top-down via `Ladoo`.
struct Affogato;
//~^ rustls_item_ordering

struct Bienenstich;
//~^ rustls_item_ordering

impl Affogato {
    fn mochi() {}
}

impl Bienenstich {
    fn halva() {}
}

impl Ladoo for Affogato {}

impl Ladoo for Bienenstich {}

impl Clone for Affogato {
    fn clone(&self) -> Self {
        Affogato
    }
}

impl Clone for Bienenstich {
    fn clone(&self) -> Self {
        Bienenstich
    }
}

mod kulfi {
    use super::Ladoo;

    pub struct Kulfi;

    // A type and its impls inside a non-root module are ordered against each
    // other, exactly as at the crate root. This inherent impl follows a
    // specific trait impl, so it lints.
    impl Ladoo for Kulfi {}

    impl Kulfi {
        //~^ rustls_item_ordering
        fn gelato() -> Self {
            Kulfi
        }
    }

    // Impls written on `kulfi::Kulfi` but living in the child module
    // `kulfi::eclair`. The type is declared in a different module from the
    // impls, so there is nothing in this module to order them against and they
    // are correctly NOT linted, despite the inherent impl following a trait
    // impl.
    mod eclair {
        impl Clone for super::Kulfi {
            fn clone(&self) -> Self {
                super::Kulfi
            }
        }

        impl super::Kulfi {
            fn churro() {}
        }
    }
}

// Impls in the crate root for a type defined in an inner module. `Kulfi`'s
// parent module is `kulfi`, the module being checked is the crate root, so
// these are ungrouped. Correctly NOT linted, despite the inherent impl
// following a specific trait impl.
impl Semifreddo for kulfi::Kulfi {}

impl kulfi::Kulfi {
    fn ladoo() {}
}

// The mirror image of the `kulfi::eclair` case at the crate root: impls living
// in `mod affogato` but written on `Bienenstich`, which is defined in the
// parent module. A type declared elsewhere gives nothing to be ordered
// against, so these are correctly NOT linted.
mod affogato {
    impl super::Semifreddo for super::Bienenstich {}

    impl super::Bienenstich {
        fn affogato() {}
    }
}

fn main() {}
