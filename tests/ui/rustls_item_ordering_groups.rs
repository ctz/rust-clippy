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
// rank lint. `Mochi` does trip top-down, because it implements `Ladoo` above.
struct Mochi;
//~^ rustls_item_ordering

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

// `self_ty_def_id` peels references, so `&Mochi` is grouped with `Mochi`. This
// specific trait impl therefore lands after `Mochi`'s common trait impl above
// and IS linted.
impl Ladoo for &Mochi {}
//~^ rustls_item_ordering

// `self_ty_def_id` does NOT peel `Box`: the path resolves to `alloc::boxed::Box`,
// which is a `DefKind::Struct` but whose `as_local()` fails, so the impl is
// ungrouped. NOT linted, even though it sits in exactly the same position as the
// `&Mochi` impl above. A surprising asymmetry with the reference case, and an
// easy way to evade the lint. The same applies to any impl written on a foreign
// wrapper of a local type.
impl Ladoo for Box<Mochi> {}

// A type alias used as the self type resolves to `DefKind::TyAlias`, which
// `self_ty_def_id` rejects, so neither impl below is grouped with `Mochi`. NOT
// linted, even though `Halva` *is* `Mochi` and both impls follow `Mochi`'s
// common trait impl -- the inherent impl in particular would be a clear
// violation if it were written as `impl Mochi`. Aliasing silently opts a type
// out of the lint.
type Halva = Mochi;
//~^ rustls_item_ordering

impl Halva {
    fn gelato() -> Self {
        Mochi
    }
}

impl Semifreddo for Halva {}

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
