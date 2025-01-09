use std::time;

use btrfs_sys::{
    btrfs_compression_type_BTRFS_COMPRESS_LZO, btrfs_compression_type_BTRFS_COMPRESS_NONE,
    btrfs_compression_type_BTRFS_COMPRESS_ZLIB, btrfs_compression_type_BTRFS_COMPRESS_ZSTD,
    btrfs_disk_key, btrfs_file_extent_item, btrfs_inode_item, btrfs_root_item,
    BTRFS_ROOT_SUBVOL_RDONLY, BTRFS_UUID_SIZE,
};

use crate::{le, Compression};

#[derive(Clone, Copy, Debug)]
pub struct Uuid([u8; BTRFS_UUID_SIZE as usize]);

#[derive(Clone, Copy, Debug)]
pub struct Inode {
    generation: le::U64,
    transid: le::U64,
    size: le::U64,
    nbytes: le::U64,
    block_group: le::U64,
    nlink: le::U32,
    uid: le::U32,
    gid: le::U32,
    mode: le::U32,
    rdev: le::U64,
    sequence: le::U64,
    atime: time::Duration,
    ctime: time::Duration,
    mtime: time::Duration,
    otime: time::Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct DiskKey {
    objectid: le::U64,
    r#type: u8,
    offset: le::U64,
}

#[derive(Clone, Copy, Debug)]
pub enum RootFlag {
    ReadOnly,
    Dead,
}

#[derive(Clone, Copy, Debug)]
pub struct Root {
    inode: Inode,
    generation: le::U64,
    root_dirid: le::U64,
    bytenr: le::U64,
    byte_limit: le::U64,
    bytes_used: le::U64,
    last_snapshot: le::U64,
    flags: RootFlag,
    refs: bool,
    btrfs_disk_key: DiskKey,
    level: u8,
    generation_v2: le::U64,
    uuid: Uuid,
    parent_uuid: Uuid,
    received_uuid: Uuid,
    ctransid: le::U64,
    stransid: le::U64,
    rtransid: le::U64,
    ctime: time::Duration,
    otime: time::Duration,
    stime: time::Duration,
    rtime: time::Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct RootRef {
    dirid: le::U64,
    sequence: le::U64,
    name_len: le::U16,
}

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
            flags: match root.flags as u32 {
                BTRFS_ROOT_SUBVOL_RDONLY => RootFlag::ReadOnly,
                _ => unreachable!("internal only flag?"),
            },
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
