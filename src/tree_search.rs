use crate::{item::FileExtentInline, item::FileExtentReg};

use btrfs_sys::{
    btrfs_file_extent_item, btrfs_ioctl_search_header, btrfs_ioctl_search_key,
    BTRFS_EXTENT_DATA_KEY, BTRFS_FILE_EXTENT_INLINE, BTRFS_FILE_EXTENT_PREALLOC,
    BTRFS_FILE_EXTENT_REG, BTRFS_IOCTL_MAGIC,
};

use std::{fs::File, mem, ops::Range, os::fd::AsRawFd, slice};

const IOCTL_BUFF_SIZE: usize = 2usize.pow(16);

nix::ioctl_readwrite!(btrfs_tree_search, BTRFS_IOCTL_MAGIC, 17, TreeSearchArgs);

#[repr(C)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct TreeSearchArgs {
    key: btrfs_ioctl_search_key,
    size: u64,
    buffer: [u8; IOCTL_BUFF_SIZE],
}

#[derive(Clone, Debug)]
pub enum Item {
    FileExtentReg(FileExtentReg),
    FileExtentInline(FileExtentInline),
}

#[derive(Debug)]
pub struct TreeSearch {
    args: TreeSearchArgs,
    file: File,
    bp: usize,
    items: u32,
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

impl TreeSearch {
    pub fn new(
        file: File,
        tree_id: u64,
        objectids: Range<u64>,
        offsets: Range<u64>,
        transids: Range<u64>,
        types: Range<u32>,
        items: u32,
    ) -> Self {
        let args = TreeSearchArgs::new(tree_id, objectids, offsets, transids, types, 0);

        Self {
            args,
            file,
            bp: 0,
            items,
        }
    }
}

impl Iterator for TreeSearch {
    type Item = Result<Item, nix::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.key.nr_items == 0 {
            self.bp = 0;
            self.args.key.nr_items = self.items;

            match unsafe { btrfs_tree_search(self.file.as_raw_fd(), &mut self.args as *mut _) } {
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

        let item = match header.type_ {
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
            _ => todo!("{}", header.type_),
        };

        self.bp +=
            mem::size_of::<btrfs_ioctl_search_header>() + usize::try_from(header.len).unwrap();

        self.args.key.nr_items -= 1;
        self.args.key.min_offset += 1;

        Some(Ok(item))
    }
}
