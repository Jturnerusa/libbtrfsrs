#![allow(dead_code)]

pub mod info;
pub mod item;
pub mod le;
pub mod tree_search;

pub use btrfs_sys;
pub use tree_search::TreeSearch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compression {
    None,
    Zlib,
    Lzo,
    Zstd,
}
