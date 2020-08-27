use std::fmt::{self, Write, LowerHex, UpperHex, Formatter};

pub struct Hex<'a>(pub &'a [u8]);

macro_rules! fmt_impl { ($trait:ident, $fmt:expr) => {
    impl $trait for Hex<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            for b in self.0 { write!(f, $fmt, b)?; }
            Ok(())
        }
    }
}}

fmt_impl!(LowerHex, "{:02x}");
fmt_impl!(UpperHex, "{:02X}");
