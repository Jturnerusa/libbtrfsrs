use crate::item::{
    DirIndex, DirItem, FileExtentInline, FileExtentReg, FreeSpaceHeader, Inode, InodeRef, Root,
    RootRef,
};

use btrfs_sys::{
    btrfs_dir_item, btrfs_file_extent_item, btrfs_free_space_header, btrfs_inode_item,
    btrfs_inode_ref, btrfs_ioctl_search_args_v2, btrfs_ioctl_search_header, btrfs_ioctl_search_key,
    btrfs_root_item, btrfs_root_ref, BTRFS_BLOCK_GROUP_TREE_OBJECTID, BTRFS_CHUNK_TREE_OBJECTID,
    BTRFS_CSUM_TREE_OBJECTID, BTRFS_DEV_TREE_OBJECTID, BTRFS_EXTENT_TREE_OBJECTID,
    BTRFS_FILE_EXTENT_INLINE, BTRFS_FILE_EXTENT_PREALLOC, BTRFS_FILE_EXTENT_REG,
    BTRFS_FREE_SPACE_TREE_OBJECTID, BTRFS_FS_TREE_OBJECTID, BTRFS_IOCTL_MAGIC,
    BTRFS_QUOTA_TREE_OBJECTID, BTRFS_ROOT_TREE_DIR_OBJECTID, BTRFS_ROOT_TREE_OBJECTID,
    BTRFS_UUID_TREE_OBJECTID,
};

use core::{convert::TryFrom, mem, slice, unreachable};
use std::{fs::File, ops::Range, os::fd::AsRawFd};

const IOCTL_BUFF_SIZE: usize = 2usize.pow(16);

nix::ioctl_readwrite!(
    btrfs_tree_search,
    BTRFS_IOCTL_MAGIC,
    17,
    btrfs_ioctl_search_args_v2
);

#[repr(C)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct TreeSearchArgs {
    key: btrfs_ioctl_search_key,
    size: u64,
    buffer: [u8; IOCTL_BUFF_SIZE],
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Item {
    Root(Root),
    RootRef(RootRef),
    RootBackRef(RootRef),
    FileExtentReg(FileExtentReg),
    FileExtentInline(FileExtentInline),
    DirItem(DirItem),
    DirIndex(DirIndex),
    Inode(Inode),
    InodeRef(InodeRef),
    FreeSpaceHeader(FreeSpaceHeader),
}

#[derive(Clone, Copy, Debug)]
pub enum Tree {
    Auto,
    Root,
    Extent,
    Chunk,
    Dev,
    Fs,
    Dir,
    Csum,
    Quota,
    Uuid,
    FreeSpace,
    BlockGroup,
    Subvol(u64),
}

#[derive(Clone, Copy, Debug)]
pub enum KeyType {
    FreeSpaceHeader = 0,
    InodeItem = 1,
    InodeRef = 12,
    InodeExtref = 13,
    XattrItem = 24,
    VerityDescItem = 36,
    VerityMerkleItem = 37,
    OrphanItem = 48,
    DirLogItem = 60,
    DirLogIndex = 72,
    DirItem = 84,
    DirIndex = 96,
    ExtentData = 108,
    CsumItem = 120,
    ExtentCsum = 128,
    RootItem = 132,
    RootBackref = 144,
    RootRef = 156,
    ExtentItem = 168,
    MetadataItem = 169,
    TreeBlockRef = 176,
    ExtentDataRef = 178,
    ExtentRefV0 = 180,
    SharedBlockRef = 182,
    SharedDataRef = 184,
    BlockGroupItem = 192,
    FreeSpaceInfo = 198,
    FreeSpaceExtent = 199,
    FreeSpaceBitmap = 200,
    DevExtent = 204,
    DevItem = 216,
    ChunkItem = 228,
    QgroupStatus = 240,
    QgroupInfo = 242,
    QgroupLimit = 244,
    QgroupRelation = 246,
    TemporaryItem = 248,
    PersistentItem = 249,
    DevReplace = 250,
    UuidSubvol = 251,
    UuidReceivedSubvol = 252,
    StringItem = 253,
}

#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub objectid: u64,
    pub r#type: KeyType,
    pub offset: u64,
}

#[derive(Debug)]
pub struct TreeSearch<'a> {
    args: TreeSearchArgs,
    file: &'a File,
    bp: usize,
}

impl TryFrom<u32> for KeyType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::FreeSpaceHeader,
            1 => Self::InodeItem,
            12 => Self::InodeRef,
            13 => Self::InodeExtref,
            24 => Self::XattrItem,
            36 => Self::VerityDescItem,
            37 => Self::VerityMerkleItem,
            48 => Self::OrphanItem,
            60 => Self::DirLogItem,
            72 => Self::DirLogIndex,
            84 => Self::DirItem,
            96 => Self::DirIndex,
            108 => Self::ExtentData,
            120 => Self::CsumItem,
            128 => Self::ExtentCsum,
            132 => Self::RootItem,
            144 => Self::RootBackref,
            156 => Self::RootRef,
            168 => Self::ExtentItem,
            169 => Self::MetadataItem,
            176 => Self::TreeBlockRef,
            178 => Self::ExtentDataRef,
            180 => Self::ExtentRefV0,
            182 => Self::SharedBlockRef,
            184 => Self::SharedDataRef,
            192 => Self::BlockGroupItem,
            198 => Self::FreeSpaceInfo,
            199 => Self::FreeSpaceExtent,
            200 => Self::FreeSpaceBitmap,
            204 => Self::DevExtent,
            216 => Self::DevItem,
            228 => Self::ChunkItem,
            240 => Self::QgroupStatus,
            242 => Self::QgroupInfo,
            244 => Self::QgroupLimit,
            246 => Self::QgroupRelation,
            248 => Self::TemporaryItem,
            249 => Self::PersistentItem,
            250 => Self::DevReplace,
            251 => Self::UuidSubvol,
            252 => Self::UuidReceivedSubvol,
            253 => Self::StringItem,
            _ => return Err(()),
        })
    }
}

impl Tree {
    pub fn into_u64(self) -> u64 {
        match self {
            Self::Auto => 0,
            Self::Root => BTRFS_ROOT_TREE_OBJECTID as u64,
            Self::Extent => BTRFS_EXTENT_TREE_OBJECTID as u64,
            Self::Fs => BTRFS_FS_TREE_OBJECTID as u64,
            Self::Chunk => BTRFS_CHUNK_TREE_OBJECTID as u64,
            Self::Dev => BTRFS_DEV_TREE_OBJECTID as u64,
            Self::Dir => BTRFS_ROOT_TREE_DIR_OBJECTID as u64,
            Self::Csum => BTRFS_CSUM_TREE_OBJECTID as u64,
            Self::Quota => BTRFS_QUOTA_TREE_OBJECTID as u64,
            Self::Uuid => BTRFS_UUID_TREE_OBJECTID as u64,
            Self::FreeSpace => BTRFS_FREE_SPACE_TREE_OBJECTID as u64,
            Self::BlockGroup => BTRFS_BLOCK_GROUP_TREE_OBJECTID as u64,
            Self::Subvol(i) => i,
        }
    }
}

impl TreeSearchArgs {
    pub fn new(
        tree_id: u64,
        objectids: Range<u64>,
        offsets: Range<u64>,
        transids: Range<u64>,
        types: Range<u32>,
        items: u32,
    ) -> Self {
        let key = btrfs_ioctl_search_key {
            tree_id,
            min_objectid: objectids.start,
            max_objectid: objectids.end,
            min_offset: offsets.start,
            max_offset: offsets.end,
            min_transid: transids.start,
            max_transid: transids.end,
            min_type: types.start,
            max_type: types.end,
            nr_items: items,
            unused: Default::default(),
            unused1: Default::default(),
            unused2: Default::default(),
            unused3: Default::default(),
            unused4: Default::default(),
        };

        Self {
            key,
            size: IOCTL_BUFF_SIZE.try_into().unwrap(),
            buffer: [0; IOCTL_BUFF_SIZE],
        }
    }
}

impl<'a> TreeSearch<'a> {
    pub fn new(
        file: &'a File,
        tree: Tree,
        objectids: Range<u64>,
        offsets: Range<u64>,
        transids: Range<u64>,
        types: Range<u32>,
    ) -> Self {
        let args = TreeSearchArgs::new(tree.into_u64(), objectids, offsets, transids, types, 0);

        Self { args, file, bp: 0 }
    }
}

impl Iterator for TreeSearch<'_> {
    type Item = Result<(Key, Item), nix::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.key.nr_items == 0 {
            self.bp = 0;
            self.args.key.nr_items = u32::MAX;

            match unsafe {
                btrfs_tree_search(
                    self.file.as_raw_fd(),
                    (&mut self.args as *mut TreeSearchArgs).cast::<btrfs_ioctl_search_args_v2>(),
                )
            } {
                Ok(_) => (),
                Err(e) => return Some(Err(e)),
            }

            // if the ioctl returns 0, we are finished
            if self.args.key.nr_items == 0 {
                return None;
            }
        }

        let header = unsafe {
            self.args.buffer[self.bp..]
                .as_ptr()
                .cast::<btrfs_ioctl_search_header>()
                .read_unaligned()
        };

        let key = Key {
            objectid: header.objectid,
            r#type: KeyType::try_from(header.type_).unwrap(),
            offset: header.offset,
        };

        let item = match key.r#type {
            KeyType::RootItem => {
                let root = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_root_item>()
                        .read_unaligned()
                };

                Item::Root(Root::from_c_struct(root))
            }
            KeyType::RootRef | KeyType::RootBackref => {
                let root_ref = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_root_ref>()
                        .read_unaligned()
                };

                let name_offset = self.bp
                    + mem::size_of::<btrfs_ioctl_search_header>()
                    + mem::size_of::<btrfs_root_ref>();

                let slice = unsafe {
                    slice::from_raw_parts(
                        self.args.buffer[name_offset..].as_ptr(),
                        root_ref.name_len as usize,
                    )
                };

                match key.r#type {
                    KeyType::RootRef => Item::RootRef(RootRef::from_c_struct(root_ref, slice)),
                    KeyType::RootBackref => {
                        Item::RootBackRef(RootRef::from_c_struct(root_ref, slice))
                    }
                    _ => unreachable!(),
                }
            }
            KeyType::InodeItem => {
                let inode = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_inode_item>()
                        .read_unaligned()
                };

                Item::Inode(Inode::from_c_struct(inode))
            }
            KeyType::ChunkItem => todo!("chunk item"),
            KeyType::DevItem => todo!("dev item"),
            KeyType::DevExtent => todo!("dev extent item"),
            KeyType::PersistentItem => todo!("persistence item"),
            KeyType::DevReplace => todo!("dev replace item"),
            KeyType::BlockGroupItem => todo!("block group item"),
            KeyType::ExtentData => {
                let file_extent = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_file_extent_item>()
                        .read_unaligned()
                };

                match file_extent.type_ as u32 {
                    BTRFS_FILE_EXTENT_REG | BTRFS_FILE_EXTENT_PREALLOC => {
                        Item::FileExtentReg(FileExtentReg::from_c_struct(file_extent))
                    }
                    BTRFS_FILE_EXTENT_INLINE => {
                        let data = unsafe {
                            let offset = self.bp
                                + mem::size_of::<btrfs_ioctl_search_header>()
                                + mem::size_of::<u64>() * 2
                                + 1;

                            slice::from_raw_parts(
                                self.args.buffer[offset..].as_ptr(),
                                header.len.try_into().unwrap(),
                            )
                        };

                        Item::FileExtentInline(FileExtentInline::from_c_struct_and_data(
                            file_extent,
                            data,
                        ))
                    }

                    _ => unreachable!(),
                }
            }
            KeyType::ExtentItem => todo!("extent item"),
            KeyType::MetadataItem => todo!("metadata item"),
            KeyType::CsumItem => todo!("checksum item"),
            KeyType::FreeSpaceInfo => todo!("free space info item"),
            KeyType::FreeSpaceExtent => todo!("free space extent item"),
            KeyType::FreeSpaceBitmap => todo!("free space bitmap item"),
            KeyType::FreeSpaceHeader => {
                let free_space_header = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_free_space_header>()
                        .read_unaligned()
                };

                Item::FreeSpaceHeader(FreeSpaceHeader::from_c_struct(free_space_header))
            }
            KeyType::DirItem | KeyType::DirIndex | KeyType::XattrItem => {
                let dir = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_dir_item>()
                        .read_unaligned()
                };

                let name_offset = self.bp
                    + mem::size_of::<btrfs_ioctl_search_header>()
                    + mem::size_of::<btrfs_dir_item>();

                let slice = unsafe {
                    slice::from_raw_parts(
                        self.args.buffer[name_offset..].as_ptr(),
                        dir.name_len as usize,
                    )
                };

                match key.r#type {
                    KeyType::DirItem => Item::DirItem(DirItem::from_c_struct(dir, slice)),
                    KeyType::DirIndex => Item::DirIndex(DirIndex::from_c_struct(dir, slice)),
                    _ => unreachable!(),
                }
            }
            KeyType::InodeRef => {
                let inode_ref = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_inode_ref>()
                        .read_unaligned()
                };

                let name_offset = self.bp
                    + mem::size_of::<btrfs_ioctl_search_header>()
                    + mem::size_of::<btrfs_inode_ref>();

                let slice = unsafe {
                    slice::from_raw_parts(
                        self.args.buffer[name_offset..].as_ptr(),
                        inode_ref.name_len as usize,
                    )
                };

                Item::InodeRef(InodeRef::from_c_struct(inode_ref, slice))
            }
            KeyType::InodeExtref => todo!("inode extref item"),
            KeyType::QgroupStatus => todo!("qgroup status item"),
            KeyType::QgroupInfo => todo!("qgroup info item"),
            KeyType::QgroupLimit => todo!("qgroup limit item"),
            KeyType::QgroupRelation => todo!("qgroup relation item"),
            KeyType::OrphanItem => todo!("orphan item"),
            KeyType::DirLogItem => todo!("dir log item"),
            KeyType::TemporaryItem => todo!("balance item"),
            KeyType::UuidSubvol | KeyType::UuidReceivedSubvol => todo!("uuid item"),
            _ => todo!("{:?}", key.r#type),
        };

        self.bp +=
            mem::size_of::<btrfs_ioctl_search_header>() + usize::try_from(header.len).unwrap();
        self.args.key.min_objectid = header.objectid + 1;
        self.args.key.min_offset = header.offset + 1;
        self.args.key.min_type = header.type_ + 1;
        self.args.key.nr_items -= 1;

        Some(Ok((key, item)))
    }
}
