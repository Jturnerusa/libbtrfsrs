use crate::{
    item::{
        DirIndex, DirItem, FileExtentInline, FileExtentReg, FreeSpaceHeader, Inode, InodeRef, Root,
        RootRef,
    },
    Subvolume,
};

use btrfs_sys::{
    btrfs_dir_item, btrfs_file_extent_item, btrfs_free_space_header, btrfs_inode_item,
    btrfs_inode_ref, btrfs_ioctl_search_args_v2, btrfs_ioctl_search_header, btrfs_ioctl_search_key,
    btrfs_root_item, btrfs_root_ref, BTRFS_BLOCK_GROUP_ITEM_KEY, BTRFS_BLOCK_GROUP_TREE_OBJECTID,
    BTRFS_CHUNK_ITEM_KEY, BTRFS_CHUNK_TREE_OBJECTID, BTRFS_CSUM_TREE_OBJECTID,
    BTRFS_DEV_EXTENT_KEY, BTRFS_DEV_ITEM_KEY, BTRFS_DEV_REPLACE_KEY, BTRFS_DEV_STATS_KEY,
    BTRFS_DEV_TREE_OBJECTID, BTRFS_DIR_INDEX_KEY, BTRFS_DIR_ITEM_KEY, BTRFS_DIR_LOG_ITEM_KEY,
    BTRFS_EXTENT_CSUM_KEY, BTRFS_EXTENT_DATA_KEY, BTRFS_EXTENT_ITEM_KEY,
    BTRFS_EXTENT_TREE_OBJECTID, BTRFS_FILE_EXTENT_INLINE, BTRFS_FILE_EXTENT_PREALLOC,
    BTRFS_FILE_EXTENT_REG, BTRFS_FREE_SPACE_BITMAP_KEY, BTRFS_FREE_SPACE_EXTENT_KEY,
    BTRFS_FREE_SPACE_INFO_KEY, BTRFS_FREE_SPACE_TREE_OBJECTID, BTRFS_FS_TREE_OBJECTID,
    BTRFS_INODE_EXTREF_KEY, BTRFS_INODE_ITEM_KEY, BTRFS_INODE_REF_KEY, BTRFS_IOCTL_MAGIC,
    BTRFS_METADATA_ITEM_KEY, BTRFS_ORPHAN_ITEM_KEY, BTRFS_QGROUP_INFO_KEY, BTRFS_QGROUP_LIMIT_KEY,
    BTRFS_QGROUP_RELATION_KEY, BTRFS_QGROUP_STATUS_KEY, BTRFS_QUOTA_TREE_OBJECTID,
    BTRFS_ROOT_ITEM_KEY, BTRFS_ROOT_REF_KEY, BTRFS_ROOT_TREE_DIR_OBJECTID,
    BTRFS_ROOT_TREE_OBJECTID, BTRFS_TEMPORARY_ITEM_KEY, BTRFS_UUID_KEY_RECEIVED_SUBVOL,
    BTRFS_UUID_KEY_SUBVOL, BTRFS_UUID_TREE_OBJECTID,
};

use core::{convert::TryFrom, mem, slice, unreachable};
use std::{ops::Range, os::fd::AsRawFd};

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
    BtrfsInodeItemKey = 1,
    BtrfsInodeRefKey = 12,
    BtrfsInodeExtrefKey = 13,
    BtrfsXattrItemKey = 24,
    BtrfsVerityDescItemKey = 36,
    BtrfsVerityMerkleItemKey = 37,
    BtrfsOrphanItemKey = 48,
    BtrfsDirLogItemKey = 60,
    BtrfsDirLogIndexKey = 72,
    BtrfsDirItemKey = 84,
    BtrfsDirIndexKey = 96,
    BtrfsExtentDataKey = 108,
    BtrfsCsumItemKey = 120,
    BtrfsExtentCsumKey = 128,
    BtrfsRootItemKey = 132,
    BtrfsRootBackrefKey = 144,
    BtrfsRootRefKey = 156,
    BtrfsExtentItemKey = 168,
    BtrfsMetadataItemKey = 169,
    BtrfsTreeBlockRefKey = 176,
    BtrfsExtentDataRefKey = 178,
    BtrfsExtentRefV0Key = 180,
    BtrfsSharedBlockRefKey = 182,
    BtrfsSharedDataRefKey = 184,
    BtrfsBlockGroupItemKey = 192,
    BtrfsFreeSpaceInfoKey = 198,
    BtrfsFreeSpaceExtentKey = 199,
    BtrfsFreeSpaceBitmapKey = 200,
    BtrfsDevExtentKey = 204,
    BtrfsDevItemKey = 216,
    BtrfsChunkItemKey = 228,
    BtrfsQgroupStatusKey = 240,
    BtrfsQgroupInfoKey = 242,
    BtrfsQgroupLimitKey = 244,
    BtrfsQgroupRelationKey = 246,
    BtrfsTemporaryItemKey = 248,
    BtrfsPersistentItemKey = 249,
    BtrfsDevReplaceKey = 250,
    BtrfsUuidKeySubvol = 251,
    BtrfsUuidKeyReceivedSubvol = 252,
    BtrfsStringItemKey = 253,
}

#[derive(Clone, Copy, Debug)]
pub struct Key {
    objectid: u64,
    r#type: KeyType,
    offset: u64,
}

#[derive(Debug)]
pub struct TreeSearch<'a> {
    args: TreeSearchArgs,
    subvol: Subvolume<'a>,
    bp: usize,
}

impl TryFrom<u32> for KeyType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::BtrfsInodeItemKey,
            12 => Self::BtrfsInodeRefKey,
            13 => Self::BtrfsInodeExtrefKey,
            24 => Self::BtrfsXattrItemKey,
            36 => Self::BtrfsVerityDescItemKey,
            37 => Self::BtrfsVerityMerkleItemKey,
            48 => Self::BtrfsOrphanItemKey,
            60 => Self::BtrfsDirLogItemKey,
            72 => Self::BtrfsDirLogIndexKey,
            84 => Self::BtrfsDirItemKey,
            96 => Self::BtrfsDirIndexKey,
            108 => Self::BtrfsExtentDataKey,
            120 => Self::BtrfsCsumItemKey,
            128 => Self::BtrfsExtentCsumKey,
            132 => Self::BtrfsRootItemKey,
            144 => Self::BtrfsRootBackrefKey,
            156 => Self::BtrfsRootRefKey,
            168 => Self::BtrfsExtentItemKey,
            169 => Self::BtrfsMetadataItemKey,
            176 => Self::BtrfsTreeBlockRefKey,
            178 => Self::BtrfsExtentDataRefKey,
            180 => Self::BtrfsExtentRefV0Key,
            182 => Self::BtrfsSharedBlockRefKey,
            184 => Self::BtrfsSharedDataRefKey,
            192 => Self::BtrfsBlockGroupItemKey,
            198 => Self::BtrfsFreeSpaceInfoKey,
            199 => Self::BtrfsFreeSpaceExtentKey,
            200 => Self::BtrfsFreeSpaceBitmapKey,
            204 => Self::BtrfsDevExtentKey,
            216 => Self::BtrfsDevItemKey,
            228 => Self::BtrfsChunkItemKey,
            240 => Self::BtrfsQgroupStatusKey,
            242 => Self::BtrfsQgroupInfoKey,
            244 => Self::BtrfsQgroupLimitKey,
            246 => Self::BtrfsQgroupRelationKey,
            248 => Self::BtrfsTemporaryItemKey,
            249 => Self::BtrfsPersistentItemKey,
            250 => Self::BtrfsDevReplaceKey,
            251 => Self::BtrfsUuidKeySubvol,
            252 => Self::BtrfsUuidKeyReceivedSubvol,
            253 => Self::BtrfsStringItemKey,
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
        subvol: Subvolume<'a>,
        tree: Tree,
        objectids: Range<u64>,
        offsets: Range<u64>,
        transids: Range<u64>,
        types: Range<u32>,
    ) -> Self {
        let args = TreeSearchArgs::new(tree.into_u64(), objectids, offsets, transids, types, 0);

        Self {
            args,
            subvol,
            bp: 0,
        }
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
                    self.subvol.as_file().as_raw_fd(),
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

        let item = match header.type_ {
            BTRFS_ROOT_ITEM_KEY => {
                let root = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_root_item>()
                        .read_unaligned()
                };

                Item::Root(Root::from_c_struct(root))
            }
            BTRFS_ROOT_REF_KEY => {
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

                Item::RootRef(RootRef::from_c_struct(root_ref, slice))
            }
            BTRFS_INODE_ITEM_KEY => {
                let inode = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_inode_item>()
                        .read_unaligned()
                };

                Item::Inode(Inode::from_c_struct(inode))
            }
            BTRFS_CHUNK_ITEM_KEY => todo!("chunk item"),
            BTRFS_DEV_ITEM_KEY => todo!("dev item"),
            BTRFS_DEV_EXTENT_KEY => todo!("dev extent item"),
            BTRFS_DEV_STATS_KEY => todo!("dev stats item"),
            BTRFS_DEV_REPLACE_KEY => todo!("dev replace item"),
            BTRFS_BLOCK_GROUP_ITEM_KEY => todo!("block group item"),
            BTRFS_EXTENT_DATA_KEY => {
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
            BTRFS_EXTENT_ITEM_KEY => todo!("extent item"),
            BTRFS_METADATA_ITEM_KEY => todo!("metadata item"),
            BTRFS_EXTENT_CSUM_KEY => todo!("checksum item"),
            BTRFS_FREE_SPACE_INFO_KEY => todo!("free space info item"),
            BTRFS_FREE_SPACE_EXTENT_KEY => todo!("free space extent item"),
            BTRFS_FREE_SPACE_BITMAP_KEY => todo!("free space bitmap item"),
            0 => {
                let free_space_header = unsafe {
                    self.args.buffer[self.bp + mem::size_of::<btrfs_ioctl_search_header>()..]
                        .as_ptr()
                        .cast::<btrfs_free_space_header>()
                        .read_unaligned()
                };

                Item::FreeSpaceHeader(FreeSpaceHeader::from_c_struct(free_space_header))
            }
            BTRFS_DIR_ITEM_KEY | BTRFS_DIR_INDEX_KEY => {
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

                match header.type_ {
                    BTRFS_DIR_ITEM_KEY => Item::DirItem(DirItem::from_c_struct(dir, slice)),
                    BTRFS_DIR_INDEX_KEY => Item::DirIndex(DirIndex::from_c_struct(dir, slice)),
                    _ => unreachable!(),
                }
            }
            BTRFS_INODE_REF_KEY => {
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
            BTRFS_INODE_EXTREF_KEY => todo!("inode extref item"),
            BTRFS_QGROUP_STATUS_KEY => todo!("qgroup status item"),
            BTRFS_QGROUP_INFO_KEY => todo!("qgroup info item"),
            BTRFS_QGROUP_LIMIT_KEY => todo!("qgroup limit item"),
            BTRFS_QGROUP_RELATION_KEY => todo!("qgroup relation item"),
            BTRFS_ORPHAN_ITEM_KEY => todo!("orphan item"),
            BTRFS_DIR_LOG_ITEM_KEY => todo!("dir log item"),
            BTRFS_TEMPORARY_ITEM_KEY => todo!("balance item"),
            BTRFS_UUID_KEY_SUBVOL | BTRFS_UUID_KEY_RECEIVED_SUBVOL => todo!("uuid item"),
            _ => unreachable!(),
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
