use anyhow::{anyhow, Context, Result};
use memflow::prelude::v1::*;
use memflow_qemu_procfs::QemuProcfs;
use memflow_win32::{Kernel, Win32Process};

fn main() -> Result<()> {
    println!("[*] Connecting to QEMU VM via memflow-win32...");
    let mut connector = QemuProcfs::new()?;

    let mut kernel = Kernel::builder(&mut connector)
    .build()
    .context("Failed to initialize Win32 kernel")?;
    println!("[+] Kernel initialized.");

    let target_name = "r5apex_dx12";

    println!("[*] Searching for process starting with '{}'", target_name);
    let proc_list = kernel.process_info_list().context("Failed to list processes")?;

    for proc in proc_list {
        let name = proc.name.to_lowercase();
        println!("[DEBUG] PID: {} Name: {} DTB: {:#X} SectionBase: {:#X}", proc.pid, name, proc.dtb, proc.section_base);

        if name.starts_with(target_name) {
            println!("[+] Found match: {}", name);
            if proc.dtb.as_u64() == 0 {
                println!("[-] Skipping process '{}': invalid DTB.", name);
                continue;
            }

            let base_address = proc.section_base;

            println!("[*] Attempting to attach to process '{}' using with_kernel_ref...", proc.name);
            let mut process = Win32Process::with_kernel_ref(&mut kernel, proc);
            println!("[+] Successfully attached to '{}'", name);

            let offset = 0x481;
            let target_address = base_address + offset;

            println!("[*] Base Address: {:#X}", base_address);
            println!(
                "[*] Reading from offset 0x{:X} -> Addr: {:#X}",
                offset, target_address
            );

            let raw: u32 = process
            .virt_mem
            .virt_read(target_address)
            .context("Failed to read memory at target address")?;

            let float_val = f32::from_bits(raw);
            println!("[+] Raw: {} | Float: {:.3}", raw, float_val);

            return Ok(());
        }
    }

    Err(anyhow!("[-] Failed to find a valid process named '{}'", target_name))
}
