use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::unix::prelude::*;
use std::path::PathBuf;

use super::Command;

use structopt::StructOpt;

const SYS_KEXEC_FILE_LOAD: usize = 320;

fn kexec<K: AsRawFd, I: AsRawFd>(kernel: K, initrd: I, cmdline: &CStr) -> std::io::Result<()> {
    let kernel = kernel.as_raw_fd() as usize;
    let initrd = initrd.as_raw_fd();
    let cmdline = cmdline.to_bytes_with_nul();
    let retval: usize;

    #[cfg(target_arch = "aarch64")]
    unsafe {
        asm!(
            "svc #0",
            in("x8") SYS_KEXEC_FILE_LOAD,
            inout("x0") kernel => retval,
            in("x1") initrd,
            in("x2") cmdline.len(),
            in("x3") cmdline.as_ptr(),
            in("x4") 0,
            in("x5") 0,
        );
    }

    eprintln!("{}", retval as isize);
    if retval > -4096isize as usize {
        let code = -(retval as isize) as i32;
        return Err(std::io::Error::from_raw_os_error(code));
    }

    Ok(())
}

#[derive(StructOpt, Debug)]
pub struct Kexec {
    #[structopt(long, short)]
    kernel: PathBuf,
    #[structopt(long, short)]
    initrd: PathBuf,
    #[structopt(long, short)]
    cmdline: String,
}

impl Command for Kexec {
    fn execute(self) -> anyhow::Result<()> {
        let kernel = File::open(self.kernel)?;
        let initrd = File::open(self.initrd)?;
        let cmdline = CString::new(self.cmdline)?;

        kexec(kernel, initrd, &cmdline)?;
        Ok(())
    }
}
