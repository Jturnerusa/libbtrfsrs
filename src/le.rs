macro_rules! le {
    ($struct:tt, $t:ty) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $struct($t);

        impl $struct {
            pub fn new(i: $t) -> Self {
                if cfg!(target_endian = "little") {
                    Self(i)
                } else {
                    Self(<$t>::from_le_bytes(<$t>::to_le_bytes(i)))
                }
            }

            pub fn get(self) -> $t {
                self.0
            }
        }
    };
}

le!(U64, u64);
le!(U32, u32);
le!(U16, u16);
