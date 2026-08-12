#![warn(clippy::rustls_item_ordering)]

pub struct Cheesecake;

impl std::fmt::Debug for Cheesecake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cheesecake")
    }
}

impl Cheesecake {
    //~^ rustls_item_ordering
    pub fn bake() -> Self {
        Self
    }
}

fn main() {}
