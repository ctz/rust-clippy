#![warn(clippy::rustls_item_ordering)]
#![allow(clippy::new_without_default, dead_code)]

// Correct top-down ordering: every item sits above the items it uses.
pub mod ganache {
    pub fn whip() -> u8 {
        temper() + CREAM
    }

    pub fn temper() -> u8 {
        CREAM
    }

    const CREAM: u8 = 1;
}

// A `const` above its only user. Constants belong at the bottom of the module.
pub mod sabayon {
    const MARSALA: u8 = 1;

    pub fn beat() -> u8 {
        //~^ rustls_item_ordering
        MARSALA
    }
}

// A function calling another function defined above it.
pub mod frangipane {
    pub fn grind() -> u8 {
        1
    }

    pub fn bake() -> u8 {
        //~^ rustls_item_ordering
        grind()
    }
}

// A type used by a function defined below it. The struct is the used item, so
// it is the one that could move down.
pub mod dacquoise {
    pub struct Meringue;

    pub fn layer() -> Meringue {
        //~^ rustls_item_ordering
        Meringue
    }
}

// A type group is a single node: the struct and its `impl` blocks together.
// `Sponge` is used from a method on `Roulade`, so it must sit below the whole
// `Roulade` group rather than below the individual `impl`.
pub mod roulade {
    pub struct Sponge;

    pub struct Roulade;
    //~^ rustls_item_ordering

    impl Roulade {
        pub fn roll(&self) -> Sponge {
            Sponge
        }
    }
}

// Method calls resolve through the receiver's type rather than through a path,
// so this use is only visible with type checking results.
pub mod petitfour {
    pub struct Icing;

    impl Icing {
        pub fn drizzle(&self) {}
    }

    pub fn decorate(icing: &Icing) {
        //~^ rustls_item_ordering
        icing.drizzle();
    }
}

// A constructor call, which appears in the HIR as a type relative path.
pub mod tuile {
    pub struct Wafer;

    impl Wafer {
        pub fn new() -> Self {
            Self
        }
    }

    pub fn curl() -> Wafer {
        //~^ rustls_item_ordering
        Wafer::new()
    }
}

// Mutually recursive functions cannot both sit above the other, so neither is
// reported.
pub mod millefeuille {
    pub fn crisp(layers: u8) -> u8 {
        if layers == 0 { 0 } else { flake(layers - 1) }
    }

    pub fn flake(layers: u8) -> u8 {
        if layers == 0 { 0 } else { crisp(layers - 1) }
    }
}

// Several uses of the same item produce a single diagnostic, not one per use.
pub mod amaretti {
    const ALMOND: u8 = 1;

    pub fn crumble() -> u8 {
        //~^ rustls_item_ordering
        ALMOND + ALMOND + ALMOND
    }
}

// Uses of items in other modules take no part in this module's ordering.
pub mod cassata {
    pub fn assemble() -> u8 {
        super::amaretti::crumble()
    }

    pub const RICOTTA: u8 = 1;
}

// A type referenced from a struct field, rather than from a body.
pub mod stollen {
    pub struct Marzipan;

    pub struct Stollen {
        //~^ rustls_item_ordering
        pub centre: Marzipan,
    }
}

// A trait used by a type that implements it.
pub mod panforte {
    pub trait Spiced {
        fn spice(&self) -> u8;
    }

    pub struct Panforte;
    //~^ rustls_item_ordering

    impl Spiced for Panforte {
        fn spice(&self) -> u8 {
            1
        }
    }
}

fn main() {}
