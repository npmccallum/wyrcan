// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2021 Profian, Inc.

use std::ffi::CString;
use std::fs::File;
use std::os::unix::prelude::*;
use std::path::PathBuf;

use super::Command;

use structopt::StructOpt;

/// Load a kernel to be executed on reboot
#[derive(StructOpt, Debug)]
pub struct Kexec {
    /// The path to the kernel to load
    #[structopt(long, short)]
    kernel: PathBuf,

    /// The path to the initrd to load
    #[structopt(long, short)]
    initrd: PathBuf,

    /// The kernel command line to use after reboot
    #[structopt(long, short)]
    cmdline: String,
}

impl Command for Kexec {
    fn execute(self) -> anyhow::Result<()> {
        let kernel = File::open(self.kernel)?;
        let initrd = File::open(self.initrd)?;
        let cmdline = CString::new(self.cmdline)?;

        let kernel = kernel.as_raw_fd() as usize;
        let initrd = initrd.as_raw_fd();
        let cmdline = cmdline.to_bytes_with_nul();
        let retval: usize;

        #[cfg(target_arch = "aarch64")]
        unsafe {
            asm!(
                "svc #0",
                in("w8") libc::SYS_kexec_file_load,
                inout("x0") kernel => retval,
                in("x1") initrd,
                in("x2") cmdline.len(),
                in("x3") cmdline.as_ptr(),
                in("x4") 0,
                in("x5") 0,
            );
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            asm!(
                "syscall",
                inout("rax") libc::SYS_kexec_file_load => retval,
                in("rdi") kernel,
                in("rsi") initrd,
                in("rdx") cmdline.len(),
                in("r10") cmdline.as_ptr(),
                in("r8") 0,
                in("r9") 0,
            );
        }

        if retval > -4096isize as usize {
            let code = -(retval as isize) as i32;
            return Err(std::io::Error::from_raw_os_error(code).into());
        }

        Ok(())
    }
}
