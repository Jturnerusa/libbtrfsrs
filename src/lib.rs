#![allow(dead_code)]

pub mod item;
pub mod le;
pub mod tree_search;

pub use tree_search::TreeSearch;

#[derive(Clone, Copy, Debug)]
pub enum Compression {
    None,
    Zlib,
    Lzo,
    Zstd,
}
