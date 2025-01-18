use crate::Uuid;
use core::convert::{From, TryFrom};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf, time};

use bitflags::{bitflags, Flags};
use btrfs_sys::{
    btrfs_block_group_item, btrfs_compression_type_BTRFS_COMPRESS_LZO,
    btrfs_compression_type_BTRFS_COMPRESS_NONE, btrfs_compression_type_BTRFS_COMPRESS_ZLIB,
    btrfs_compression_type_BTRFS_COMPRESS_ZSTD, btrfs_dir_item, btrfs_disk_key,
    btrfs_file_extent_item, btrfs_free_space_header, btrfs_inode_item, btrfs_inode_ref,
    btrfs_root_item, btrfs_root_ref, BTRFS_BLOCK_GROUP_DATA, BTRFS_BLOCK_GROUP_DUP,
    BTRFS_BLOCK_GROUP_METADATA, BTRFS_BLOCK_GROUP_RAID0, BTRFS_BLOCK_GROUP_RAID1,
    BTRFS_BLOCK_GROUP_RAID10, BTRFS_BLOCK_GROUP_RAID5, BTRFS_BLOCK_GROUP_RAID6,
    BTRFS_BLOCK_GROUP_SYSTEM, BTRFS_FT_BLKDEV, BTRFS_FT_CHRDEV, BTRFS_FT_DIR, BTRFS_FT_FIFO,
    BTRFS_FT_REG_FILE, BTRFS_FT_SYMLINK, BTRFS_FT_XATTR, BTRFS_ROOT_SUBVOL_RDONLY,
};

use crate::{le, Compression};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Inode {
    pub generation: le::U64,
    pub transid: le::U64,
    pub size: le::U64,
    pub nbytes: le::U64,
    pub block_group: le::U64,
    pub nlink: le::U32,
    pub uid: le::U32,
    pub gid: le::U32,
    pub mode: le::U32,
    pub rdev: le::U64,
    pub sequence: le::U64,
    pub atime: time::Duration,
    pub ctime: time::Duration,
    pub mtime: time::Duration,
    pub otime: time::Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InodeRef {
    pub index: le::U64,
    pub name: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiskKey {
    pub objectid: le::U64,
    pub r#type: u8,
    pub offset: le::U64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Root {
    pub inode: Inode,
    pub generation: le::U64,
    pub root_dirid: le::U64,
    pub bytenr: le::U64,
    pub byte_limit: le::U64,
    pub bytes_used: le::U64,
    pub last_snapshot: le::U64,
    pub read_only: bool,
    pub refs: bool,
    pub btrfs_disk_key: DiskKey,
    pub level: u8,
    pub generation_v2: le::U64,
    pub uuid: Uuid,
    pub parent_uuid: Uuid,
    pub received_uuid: Uuid,
    pub ctransid: le::U64,
    pub stransid: le::U64,
    pub rtransid: le::U64,
    pub ctime: time::Duration,
    pub otime: time::Duration,
    pub stime: time::Duration,
    pub rtime: time::Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RootRef {
    pub dirid: le::U64,
    pub sequence: le::U64,
    pub name: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileType {
    Reg,
    Dir,
    ChrDev,
    BlkDev,
    Fifo,
    Sock,
    Sym,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirItem {
    Xattr {
        location: DiskKey,
        transid: le::U64,
        name: Vec<u8>,
        value: Vec<u8>,
    },
    File {
        location: DiskKey,
        transid: le::U64,
        name: PathBuf,
        r#type: FileType,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirIndex {
    Xattr {
        location: DiskKey,
        transid: le::U64,
        name: Vec<u8>,
        value: Vec<u8>,
    },
    File {
        location: DiskKey,
        transid: le::U64,
        name: PathBuf,
        r#type: FileType,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileExtentReg {
    pub generation: le::U64,
    pub ram_bytes: le::U64,
    pub compression: Compression,
    pub disk_bytenr: le::U64,
    pub disk_num_bytes: le::U64,
    pub offset: le::U64,
    pub num_bytes: le::U64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileExtentInline {
    pub generation: le::U64,
    pub ram_bytes: le::U64,
    pub compression: Compression,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FreeSpaceHeader {
    pub location: DiskKey,
    pub generation: le::U64,
    pub num_entries: le::U64,
    pub num_bitmaps: le::U64,
}

bitflags! {

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BlockGroupFlag: u64 {
        const DATA = BTRFS_BLOCK_GROUP_DATA as u64;
        const SYSTEM = BTRFS_BLOCK_GROUP_SYSTEM as u64;
        const METADATA = BTRFS_BLOCK_GROUP_METADATA as u64;
        const DUP = BTRFS_BLOCK_GROUP_DUP as u64;
        const RAID0 = BTRFS_BLOCK_GROUP_RAID0 as u64;
        const RAID1 = BTRFS_BLOCK_GROUP_RAID1 as u64;
        const RAID5 = BTRFS_BLOCK_GROUP_RAID5 as u64;
        const RAID6 = BTRFS_BLOCK_GROUP_RAID6 as u64;
        const RAID10 = BTRFS_BLOCK_GROUP_RAID10 as u64;
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockGroup {
    pub used: le::U64,
    pub chunk_objectid: le::U64,
    pub flags: BlockGroupFlag,
}

impl BlockGroup {
    pub(crate) fn from_c_struct(block_group: btrfs_block_group_item) -> Result<Self, ()> {
        Ok(Self {
            used: le::U64::new(block_group.used),
            chunk_objectid: le::U64::new(block_group.chunk_objectid),
            flags: BlockGroupFlag::from_bits(block_group.flags).ok_or(())?,
        })
    }
}

impl Inode {
    pub(crate) fn from_c_struct(inode: btrfs_inode_item) -> Self {
        Self {
            generation: le::U64::new(inode.generation),
            transid: le::U64::new(inode.transid),
            size: le::U64::new(inode.size),
            nbytes: le::U64::new(inode.nbytes),
            block_group: le::U64::new(inode.block_group),
            nlink: le::U32::new(inode.nlink),
            uid: le::U32::new(inode.uid),
            gid: le::U32::new(inode.gid),
            mode: le::U32::new(inode.mode),
            rdev: le::U64::new(inode.rdev),
            sequence: le::U64::new(inode.sequence),
            atime: time::Duration::from_secs(inode.atime.sec)
                + time::Duration::from_nanos(inode.atime.nsec as u64),
            ctime: time::Duration::from_secs(inode.ctime.sec)
                + time::Duration::from_nanos(inode.ctime.nsec as u64),
            mtime: time::Duration::from_secs(inode.mtime.sec)
                + time::Duration::from_nanos(inode.mtime.nsec as u64),
            otime: time::Duration::from_secs(inode.otime.sec)
                + time::Duration::from_nanos(inode.otime.nsec as u64),
        }
    }
}

impl InodeRef {
    pub(crate) fn from_c_struct(inode_ref: btrfs_inode_ref, data: &[u8]) -> Self {
        Self {
            index: le::U64::new(inode_ref.index),
            name: PathBuf::from(<OsStr as OsStrExt>::from_bytes(data)),
        }
    }
}

impl DiskKey {
    pub(crate) fn from_c_struct(key: btrfs_disk_key) -> Self {
        Self {
            objectid: le::U64::new(key.objectid),
            r#type: key.type_,
            offset: le::U64::new(key.offset),
        }
    }
}

impl Root {
    pub(crate) fn from_c_struct(root: btrfs_root_item) -> Self {
        Self {
            inode: Inode::from_c_struct(root.inode),
            generation: le::U64::new(root.generation),
            root_dirid: le::U64::new(root.root_dirid),
            bytenr: le::U64::new(root.bytenr),
            byte_limit: le::U64::new(root.byte_limit),
            bytes_used: le::U64::new(root.bytes_used),
            last_snapshot: le::U64::new(root.last_snapshot),
            read_only: matches!(root.flags as u32, BTRFS_ROOT_SUBVOL_RDONLY),
            refs: match root.refs {
                0 => false,
                1 => true,
                _ => unreachable!(),
            },
            btrfs_disk_key: DiskKey::from_c_struct(root.drop_progress),
            level: root.level,
            generation_v2: le::U64::new(root.generation_v2),
            uuid: Uuid(root.uuid),
            parent_uuid: Uuid(root.uuid),
            received_uuid: Uuid(root.received_uuid),
            ctransid: le::U64::new(root.ctransid),
            rtransid: le::U64::new(root.rtransid),
            stransid: le::U64::new(root.stransid),
            ctime: time::Duration::from_secs(root.ctime.sec)
                + time::Duration::from_nanos(root.ctime.nsec as u64),
            otime: time::Duration::from_secs(root.otime.sec)
                + time::Duration::from_nanos(root.otime.nsec as u64),
            rtime: time::Duration::from_secs(root.rtime.sec)
                + time::Duration::from_nanos(root.rtime.nsec as u64),
            stime: time::Duration::from_secs(root.stime.sec)
                + time::Duration::from_nanos(root.stime.nsec as u64),
        }
    }
}

impl RootRef {
    pub(crate) fn from_c_struct(root_ref: btrfs_root_ref, data: &[u8]) -> Self {
        Self {
            dirid: le::U64::new(root_ref.dirid),
            sequence: le::U64::new(root_ref.sequence),
            name: PathBuf::from(<OsStr as OsStrExt>::from_bytes(data)),
        }
    }
}

impl DirItem {
    pub(crate) fn from_c_struct(dir: btrfs_dir_item, data: &[u8]) -> Self {
        match dir.type_ as u32 {
            BTRFS_FT_XATTR => Self::Xattr {
                location: DiskKey::from_c_struct(dir.location),
                transid: le::U64::new(dir.transid),
                name: data[..dir.name_len as usize].to_vec(),
                value: data[dir.data_len as usize..].to_vec(),
            },
            _ => Self::File {
                location: DiskKey::from_c_struct(dir.location),
                transid: le::U64::new(dir.transid),
                r#type: match dir.type_ as u32 {
                    BTRFS_FT_REG_FILE => FileType::Reg,
                    BTRFS_FT_DIR => FileType::Dir,
                    BTRFS_FT_CHRDEV => FileType::ChrDev,
                    BTRFS_FT_BLKDEV => FileType::BlkDev,
                    BTRFS_FT_FIFO => FileType::Fifo,
                    BTRFS_FT_SYMLINK => FileType::Sym,
                    _ => unreachable!(),
                },
                name: PathBuf::from(<OsStr as OsStrExt>::from_bytes(data)),
            },
        }
    }
}

impl DirIndex {
    pub(crate) fn from_c_struct(dir: btrfs_dir_item, data: &[u8]) -> Self {
        match dir.type_ as u32 {
            BTRFS_FT_XATTR => Self::Xattr {
                location: DiskKey::from_c_struct(dir.location),
                transid: le::U64::new(dir.transid),
                name: data[..dir.name_len as usize].to_vec(),
                value: data[dir.data_len as usize..].to_vec(),
            },
            _ => Self::File {
                location: DiskKey::from_c_struct(dir.location),
                transid: le::U64::new(dir.transid),
                r#type: match dir.type_ as u32 {
                    BTRFS_FT_REG_FILE => FileType::Reg,
                    BTRFS_FT_DIR => FileType::Dir,
                    BTRFS_FT_CHRDEV => FileType::ChrDev,
                    BTRFS_FT_BLKDEV => FileType::BlkDev,
                    BTRFS_FT_FIFO => FileType::Fifo,
                    BTRFS_FT_SYMLINK => FileType::Sym,
                    _ => unreachable!(),
                },
                name: PathBuf::from(<OsStr as OsStrExt>::from_bytes(data)),
            },
        }
    }
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

impl FreeSpaceHeader {
    pub(crate) fn from_c_struct(free_space_header: btrfs_free_space_header) -> Self {
        Self {
            location: DiskKey::from_c_struct(free_space_header.location),
            generation: le::U64::new(free_space_header.generation),
            num_entries: le::U64::new(free_space_header.num_entries),
            num_bitmaps: le::U64::new(free_space_header.num_bitmaps),
        }
    }
}
