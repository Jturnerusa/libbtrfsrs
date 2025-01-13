#![allow(dead_code)]

pub mod item;
pub mod le;
pub mod tree_search;

use core::{ffi::CStr, mem, time};
use std::{
    ffi::OsStr,
    fs::File,
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::PathBuf,
};

pub use btrfs_sys;
use btrfs_sys::{
    btrfs_ioctl_get_subvol_info_args, BTRFS_FIRST_FREE_OBJECTID, BTRFS_IOCTL_MAGIC, BTRFS_UUID_SIZE,
};
use nix::libc::BTRFS_SUPER_MAGIC;
pub use tree_search::TreeSearch;

nix::ioctl_read!(
    btrfs_get_subvol_info,
    BTRFS_IOCTL_MAGIC,
    60,
    btrfs_ioctl_get_subvol_info_args
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compression {
    None,
    Zlib,
    Lzo,
    Zstd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid(pub [u8; BTRFS_UUID_SIZE as usize]);

#[derive(Clone, Debug)]
pub struct SubvolInfo {
    tree_id: u64,
    name: PathBuf,
    parent_id: u64,
    dirid: u64,
    generation: u64,
    flags: u64,
    uuid: Uuid,
    parent_uuid: Uuid,
    received_uuid: Uuid,
    ctransid: u64,
    otransid: u64,
    stransid: u64,
    rtransid: u64,
    ctime: time::Duration,
    otime: time::Duration,
    stime: time::Duration,
    rtime: time::Duration,
}

pub struct Subvolume<'a>(&'a File);

impl<'a> Subvolume<'a> {
    pub fn new(file: &'a File) -> Result<Option<Self>, nix::Error> {
        match is_subvol(file) {
            Ok(true) => Ok(Some(Self(file))),
            Ok(false) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn info(&self) -> nix::Result<SubvolInfo> {
        let mut args: btrfs_ioctl_get_subvol_info_args = unsafe { mem::zeroed() };

        unsafe { btrfs_get_subvol_info(self.0.as_raw_fd(), &mut args as *mut _)? };

        Ok(SubvolInfo::from_c_struct(args))
    }
}

impl SubvolInfo {
    pub(crate) fn from_c_struct(info: btrfs_ioctl_get_subvol_info_args) -> Self {
        Self {
            tree_id: info.treeid,
            parent_id: info.parent_id,
            name: {
                let cstr = unsafe { &CStr::from_ptr(info.name.as_slice().as_ptr()) };
                let osstr = <OsStr as OsStrExt>::from_bytes(cstr.to_bytes());
                PathBuf::from(osstr)
            },
            dirid: info.dirid,
            generation: info.generation,
            flags: info.flags,
            uuid: Uuid(info.uuid),
            parent_uuid: Uuid(info.parent_uuid),
            received_uuid: Uuid(info.received_uuid),
            ctransid: info.ctransid,
            otransid: info.otransid,
            stransid: info.stransid,
            rtransid: info.rtransid,
            ctime: time::Duration::from_secs(info.ctime.sec)
                + time::Duration::from_nanos(info.ctime.nsec as u64),
            otime: time::Duration::from_secs(info.otime.sec)
                + time::Duration::from_nanos(info.otime.nsec as u64),
            stime: time::Duration::from_secs(info.stime.sec)
                + time::Duration::from_nanos(info.stime.nsec as u64),
            rtime: time::Duration::from_secs(info.rtime.sec)
                + time::Duration::from_nanos(info.rtime.nsec as u64),
        }
    }
}

fn is_subvol(file: &File) -> nix::Result<bool> {
    let statfs = nix::sys::statfs::fstatfs(file)?;
    let stat = nix::sys::stat::fstat(file.as_raw_fd())?;

    Ok(statfs.filesystem_type().0 == BTRFS_SUPER_MAGIC
        && stat.st_ino == BTRFS_FIRST_FREE_OBJECTID as u64
        && stat.st_mode & nix::sys::stat::SFlag::S_IFMT.bits()
            == nix::sys::stat::SFlag::S_IFDIR.bits())
}
