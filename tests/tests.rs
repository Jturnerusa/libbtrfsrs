use core::{error::Error, str};
use std::{
    fs::{self, File},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use libbtrfsrs::SubVolFlag;

const IMAGE_SIZE: u64 = 114294784;

macro_rules! cmd {
    ($cmd:expr, $($arg:expr),+) => {
        {
            let mut cmd = ::std::process::Command::new($cmd);

            $(cmd.arg($arg);)+

            let output = cmd.output().unwrap();

            eprintln!("{}", $cmd);

            if !output.stdout.is_empty() {
                eprintln!("{}", ::std::str::from_utf8(output.stdout.as_slice()).unwrap_or("?"));
            }

            if !output.stderr.is_empty() {
                eprintln!("{}", ::std::str::from_utf8(output.stderr.as_slice()).unwrap_or("?"));
            }

            assert!(output.status.success());

            (
                <::std::ffi::OsString as ::std::os::unix::ffi::OsStringExt>::from_vec(output.stdout),
                <::std::ffi::OsString as ::std::os::unix::ffi::OsStringExt>::from_vec(output.stderr)
            )
        }
    }
}

fn make_test_image(path: &Path, size: u64) {
    cmd!("fallocate", "--length", size.to_string(), path.as_os_str());

    cmd!("mkfs.btrfs", path.as_os_str());
}

fn mount_image(image_path: &Path, mount_path: &Path) -> PathBuf {
    let (loopdev, _) = cmd!("losetup", "--find", "--show", image_path.as_os_str());

    let loopdev_str = str::from_utf8(loopdev.as_os_str().as_bytes())
        .unwrap()
        .trim_end();

    cmd!("mount", loopdev_str, mount_path.as_os_str());

    PathBuf::from(loopdev_str)
}

fn unmount_image(image_path: &Path) {
    cmd!("umount", image_path.as_os_str());
}

fn detach_loop(loop_path: &Path) {
    cmd!("losetup", "--detach", loop_path.as_os_str());
}

fn run_test(test_fn: impl Fn(&Path) -> Result<(), Box<dyn Error>>) {
    let (_, image_path) = nix::unistd::mkstemp("/tmp/imageXXXXXXXX").unwrap();
    let mnt_path = nix::unistd::mkdtemp("/mnt/testXXXXXXXX").unwrap();

    make_test_image(image_path.as_path(), IMAGE_SIZE);
    let loop_path = mount_image(image_path.as_path(), mnt_path.as_path());

    let res = test_fn(mnt_path.as_path());

    unmount_image(mnt_path.as_path());
    detach_loop(loop_path.as_path());

    fs::remove_dir(mnt_path.as_path()).unwrap();
    fs::remove_file(image_path.as_path()).unwrap();

    assert!(res.is_ok());
}

#[test]
fn test_create_subvolume() {
    run_test(|mnt_path: &Path| {
        let mnt_parent = File::open(mnt_path)?;
        let subvol_name = "subvol";

        libbtrfsrs::create_subvolume(&mnt_parent, subvol_name, SubVolFlag::empty())?;

        let subvol_file = File::open(mnt_path.join(subvol_name))?;

        if !libbtrfsrs::is_subvol(&subvol_file)? {
            Err("failed to create subvolume".to_string())?
        }

        Ok(())
    })
}

#[test]
fn test_create_snapshot() {
    run_test(|mnt_path: &Path| {
        let mnt_parent = File::open(mnt_path)?;
        let subvol_name = "subvol";
        let snapshot_name = "snapshot";

        libbtrfsrs::create_subvolume(&mnt_parent, subvol_name, SubVolFlag::empty())?;

        {
            let subvol_file = File::open(mnt_path.join(subvol_name))?;

            if !libbtrfsrs::is_subvol(&subvol_file)? {
                Err("failed to create subvolume".to_string())?
            }

            libbtrfsrs::create_snapshot(
                &mnt_parent,
                &subvol_file,
                snapshot_name,
                SubVolFlag::empty(),
            )?;

            let snapshot_file = File::open(mnt_path.join(snapshot_name))?;

            if !libbtrfsrs::is_subvol(&snapshot_file)? {
                Err("failed to create subvolume".to_string())?
            }
        }

        Ok(())
    })
}

#[test]
fn test_create_ro_snapshot() {
    run_test(|mnt_path: &Path| {
        let mnt_parent = File::open(mnt_path)?;
        let subvol_name = "subvol";
        let snapshot_name = "snapshot";

        libbtrfsrs::create_subvolume(&mnt_parent, subvol_name, SubVolFlag::empty())?;

        {
            let subvol_file = File::open(mnt_path.join(subvol_name))?;

            if !libbtrfsrs::is_subvol(&subvol_file)? {
                Err("failed to create subvolume".to_string())?
            }

            libbtrfsrs::create_snapshot(
                &mnt_parent,
                &subvol_file,
                snapshot_name,
                SubVolFlag::READ_ONLY,
            )?;

            match File::create(mnt_path.join(snapshot_name).as_path().join("testfile")) {
                Ok(_) => Err("snapshot is not read-only".to_string())?,
                Err(e) if matches!(dbg!(e.kind()), std::io::ErrorKind::ReadOnlyFilesystem) => (),
                Err(e) => Err(e)?,
            }
        }

        Ok(())
    })
}
