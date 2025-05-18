use anyhow::{Context, Result};
use memflow::mem::VirtualMemory;
use memflow_qemu_procfs::QemuProcfs;
use memflow_win32::win32::{Kernel, Win32Process};

fn main() -> Result<()> {
    println!("[*] Connecting to QEMU VM 'win10v2'...");
    let mut connector = QemuProcfs::new()?;
    println!("[+] Connected.");

    let mut kernel = Kernel::builder(&mut connector).build()?;
    println!("[+] Kernel initialized.");

    let target_proc = "r5apex_dx12";

    println!("[*] Searching for process starting with '{}'", target_proc);
    let proc_list = kernel.process_info_list().context("Failed to list processes")?;

    for proc in proc_list {
        let name = proc.name.to_lowercase();
        println!("[DEBUG] Found process: {}", name);

        if name.starts_with(target_proc) {
            println!("[+] Match found: {}", name);
            println!("[*] Attempting to attach to process '{}'...", proc.name);

            let mut process = kernel
            .into_process(&proc.name)
            .context("[ERR] Failed to attach to process")?;

            println!("[+] Successfully attached to {}", name);

            let module = process
            .module_info(&proc.name)
            .context("Failed to retrieve module info")?;

            let base_address = module.base;
            let offset = 0x481;
            let target_address = base_address + offset;

            println!("[*] Module base: {:#X}", base_address);
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

    println!("[-] Failed to find a matching process.");
    Ok(())
}
