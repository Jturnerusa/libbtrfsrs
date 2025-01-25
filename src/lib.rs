#![allow(dead_code)]

pub mod item;
pub mod le;
pub mod logical_ino;
pub mod tree_search;

pub use btrfs_sys;
pub use logical_ino::LogicalIno;
pub use tree_search::TreeSearch;

use core::{convert::AsRef, ffi::CStr, iter::Iterator, mem, str, time};

use std::{
    ffi::OsStr,
    fs::File,
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::{Path, PathBuf},
};

use btrfs_sys::{
    btrfs_ioctl_get_subvol_info_args, btrfs_ioctl_vol_args_v2,
    btrfs_ioctl_vol_args_v2__bindgen_ty_2, BTRFS_FIRST_FREE_OBJECTID, BTRFS_IOCTL_MAGIC,
    BTRFS_SUBVOL_RDONLY, BTRFS_UUID_SIZE,
};

use bitflags::bitflags;
use nix::libc::BTRFS_SUPER_MAGIC;
use tree_search::{Item, Tree};

const IOCTL_BUFF_SIZE: usize = 2usize.pow(16);

nix::ioctl_read!(
    btrfs_get_subvol_info,
    BTRFS_IOCTL_MAGIC,
    60,
    btrfs_ioctl_get_subvol_info_args
);

nix::ioctl_write_ptr!(
    btrfs_snap_create_v2,
    BTRFS_IOCTL_MAGIC,
    23,
    btrfs_ioctl_vol_args_v2
);

nix::ioctl_write_ptr!(
    btrfs_subvol_create_v2,
    BTRFS_IOCTL_MAGIC,
    24,
    btrfs_ioctl_vol_args_v2
);

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct SubVolFlag: u64 {
        const READ_ONLY = BTRFS_SUBVOL_RDONLY as u64;
    }
}

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
    pub tree_id: u64,
    pub name: PathBuf,
    pub parent_id: u64,
    pub dirid: u64,
    pub generation: u64,
    pub flags: u64,
    pub uuid: Uuid,
    pub parent_uuid: Uuid,
    pub received_uuid: Uuid,
    pub ctransid: u64,
    pub otransid: u64,
    pub stransid: u64,
    pub rtransid: u64,
    pub ctime: time::Duration,
    pub otime: time::Duration,
    pub stime: time::Duration,
    pub rtime: time::Duration,
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

pub fn info(subvol: &File) -> nix::Result<SubvolInfo> {
    let mut args: btrfs_ioctl_get_subvol_info_args = unsafe { mem::zeroed() };

    unsafe { btrfs_get_subvol_info(subvol.as_raw_fd(), &mut args as *mut _)? };

    Ok(SubvolInfo::from_c_struct(args))
}

pub fn create_subvolume<T: AsRef<Path>>(
    parent: &File,
    name: T,
    flags: SubVolFlag,
) -> Result<(), nix::Error> {
    let mut args: btrfs_ioctl_vol_args_v2 = unsafe { mem::zeroed() };

    args.flags = flags.bits();

    let mut name_buf = [0i8; 4040];
    let name_str = name.as_ref().as_os_str();

    for (i, byte) in name_str.as_bytes().iter().enumerate() {
        name_buf[i] = *byte as i8;
    }

    args.__bindgen_anon_2 = btrfs_ioctl_vol_args_v2__bindgen_ty_2 { name: name_buf };

    unsafe { btrfs_subvol_create_v2(parent.as_raw_fd(), &args as *const _)? };

    Ok(())
}

pub fn create_snapshot<T: AsRef<Path>>(
    parent: &File,
    subvol: &File,
    name: T,
    flags: SubVolFlag,
) -> Result<(), nix::Error> {
    let mut name_buf = [0i8; 4040];

    for (i, byte) in name.as_ref().as_os_str().as_bytes().iter().enumerate() {
        name_buf[i] = *byte as i8;
    }

    let args = btrfs_ioctl_vol_args_v2 {
        fd: subvol.as_raw_fd() as i64,
        transid: Default::default(),
        flags: flags.bits(),
        __bindgen_anon_1: btrfs_sys::btrfs_ioctl_vol_args_v2__bindgen_ty_1 {
            unused: Default::default(),
        },
        __bindgen_anon_2: btrfs_sys::btrfs_ioctl_vol_args_v2__bindgen_ty_2 { name: name_buf },
    };

    unsafe { btrfs_snap_create_v2(parent.as_raw_fd(), &args as *const _)? };

    Ok(())
}

pub fn is_subvol(file: &File) -> nix::Result<bool> {
    let statfs = nix::sys::statfs::fstatfs(file)?;
    let stat = nix::sys::stat::fstat(file.as_raw_fd())?;

    Ok(statfs.filesystem_type().0 == BTRFS_SUPER_MAGIC
        && stat.st_ino == BTRFS_FIRST_FREE_OBJECTID as u64
        && stat.st_mode & nix::sys::stat::SFlag::S_IFMT.bits()
            == nix::sys::stat::SFlag::S_IFDIR.bits())
}

fn get_subvolume_name_from_id(id: u64, root: &File) -> Result<Option<PathBuf>, nix::Error> {
    for item in TreeSearch::search_all(root, Tree::Root) {
        match item {
            Ok((key, Item::RootBackRef(root))) if key.objectid == id => {
                return Ok(Some(root.name.clone()))
            }
            Ok(_) => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(None)
}
