use btrfs_sys::{
    btrfs_compression_type_BTRFS_COMPRESS_LZO, btrfs_compression_type_BTRFS_COMPRESS_NONE,
    btrfs_compression_type_BTRFS_COMPRESS_ZLIB, btrfs_compression_type_BTRFS_COMPRESS_ZSTD,
    btrfs_file_extent_item,
};

use crate::{le, Compression};

#[derive(Clone, Copy, Debug)]
pub struct FileExtentReg {
    generation: le::U64,
    ram_bytes: le::U64,
    compression: Compression,
    disk_bytenr: le::U64,
    disk_num_bytes: le::U64,
    offset: le::U64,
    num_bytes: le::U64,
}

#[derive(Clone, Debug)]
pub struct FileExtentInline {
    generation: le::U64,
    ram_bytes: le::U64,
    compression: Compression,
    data: Vec<u8>,
}

impl FileExtentReg {
    #[allow(non_upper_case_globals)]
    pub(crate) fn from_c_struct(item: btrfs_file_extent_item) -> Self {
        Self {
            generation: le::U64::new(item.generation),
            ram_bytes: le::U64::new(item.ram_bytes),
            compression: match item.compression as u32 {
                btrfs_compression_type_BTRFS_COMPRESS_NONE => Compression::None,
                btrfs_compression_type_BTRFS_COMPRESS_LZO => Compression::Lzo,
                btrfs_compression_type_BTRFS_COMPRESS_ZLIB => Compression::Zlib,
                btrfs_compression_type_BTRFS_COMPRESS_ZSTD => Compression::Zstd,
                _ => unreachable!(),
            },
            disk_bytenr: le::U64::new(item.disk_bytenr),
            disk_num_bytes: le::U64::new(item.disk_num_bytes),
            offset: le::U64::new(item.offset),
            num_bytes: le::U64::new(item.num_bytes),
        }
    }
}

impl FileExtentInline {
    #[allow(non_upper_case_globals)]
    pub(crate) fn from_c_struct_and_data(item: btrfs_file_extent_item, data: &[u8]) -> Self {
        Self {
            generation: le::U64::new(item.generation),
            ram_bytes: le::U64::new(item.ram_bytes),
            compression: match item.compression as u32 {
                btrfs_compression_type_BTRFS_COMPRESS_NONE => Compression::None,
                btrfs_compression_type_BTRFS_COMPRESS_LZO => Compression::Lzo,
                btrfs_compression_type_BTRFS_COMPRESS_ZLIB => Compression::Zlib,
                btrfs_compression_type_BTRFS_COMPRESS_ZSTD => Compression::Zstd,
                _ => unreachable!(),
            },
            data: data.to_vec(),
        }
    }
}
