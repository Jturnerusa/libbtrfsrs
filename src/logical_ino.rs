use crate::IOCTL_BUFF_SIZE;
use btrfs_sys::{btrfs_data_container, btrfs_ioctl_logical_ino_args, BTRFS_IOCTL_MAGIC};
use std::{fs::File, os::fd::AsRawFd};

nix::ioctl_readwrite!(
    btrfs_logical_ino,
    BTRFS_IOCTL_MAGIC,
    36,
    btrfs_ioctl_logical_ino_args
);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Container {
    bytes_left: u32,
    bytes_missing: u32,
    elem_cnt: u32,
    elem_missed: u32,
    buff: [u64; IOCTL_BUFF_SIZE],
}

#[derive(Clone, Copy, Debug)]
pub struct LogicalIno<'a> {
    file: &'a File,
    bytenr: u64,
    container: Option<Container>,
    bp: usize,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            bytes_left: Default::default(),
            bytes_missing: Default::default(),
            elem_cnt: Default::default(),
            elem_missed: Default::default(),
            buff: [0; IOCTL_BUFF_SIZE],
        }
    }
}

impl<'a> LogicalIno<'a> {
    pub fn new(file: &'a File, bytenr: u64) -> Self {
        Self {
            file,
            bytenr,
            container: None,
            bp: 0,
        }
    }
}

impl Iterator for LogicalIno<'_> {
    type Item = Result<(u64, u64, u64), nix::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.container.is_none() {
            let mut container = Container::default();

            let mut args = btrfs_ioctl_logical_ino_args {
                logical: self.bytenr,
                size: IOCTL_BUFF_SIZE as u64,
                reserved: Default::default(),
                flags: 0,
                inodes: (&mut container as *mut Container)
                    .cast::<btrfs_data_container>()
                    .addr() as u64,
            };

            match unsafe { btrfs_logical_ino(self.file.as_raw_fd(), &mut args as *mut _) } {
                Ok(_) => (),
                Err(e) => return Some(Err(e)),
            }

            self.container = Some(container);
        }

        let container = self.container.unwrap();

        if container.elem_cnt > 0 {
            let inum = container.buff[self.bp];
            let offset = container.buff[self.bp + 1];
            let root = container.buff[self.bp + 2];

            self.container.as_mut().unwrap().elem_cnt -= 1;
            self.bp += 3;

            Some(Ok((inum, offset, root)))
        } else {
            None
        }
    }
}
