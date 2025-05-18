use anyhow::{anyhow, Context, Result};
use memflow::prelude::v1::*;
use memflow_qemu_procfs::{QemuProcfs};
use memflow_win32::{Kernel, Win32Process};
use std::str;

fn main() -> Result<()> {
    println!("[*] Connecting to QEMU VM via memflow-win32...");

    // Using QemuProcfs for connector
    let mut connector = QemuProcfs::new().context("Failed to create QEMU connector")?;

    // Initialize kernel with the connector
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

            // Loop to continuously read memory until a non-zero or valid float is found
            loop {
                let raw: u32 = process
                .virt_mem
                .virt_read(target_address)
                .context("Failed to read memory at target address")?;

                let float_val = f32::from_bits(raw);

                if raw != 0 {
                    println!("[+] Raw: {} | Float: {:.3}", raw, float_val);
                    break; // Exit the loop when a valid value is found
                } else {
                    println!("[*] Raw: 0 (waiting for valid value...)");
                }
            }

            // Reading next 64 bytes as UTF-8 string
            let string_start_address = base_address + 0x500; // Example offset for string (can be adjusted)
            let mut string_buf = vec![0u8; 64]; // Buffer to store UTF-8 bytes

            process
            .virt_mem
            .virt_read_raw_into(string_start_address, &mut string_buf)
            .context("Failed to read memory for UTF-8 string")?;

            // Print the raw bytes for inspection
            println!("[DEBUG] Raw memory bytes: {:?}", string_buf);

            // Try to decode the buffer as UTF-8 string
            match str::from_utf8(&string_buf) {
                Ok(decoded_str) => {
                    println!("[+] Decoded UTF-8 string: {}", decoded_str);
                }
                Err(_) => {
                    // If it fails to decode as UTF-8, print as raw data
                    let ascii_string = string_buf.iter().map(|&b| b as char).collect::<String>();
                    println!("[*] Failed to decode UTF-8 string. Raw data: {}", ascii_string);
                }
            }

            return Ok(());
        }
    }

    Err(anyhow!("[-] Failed to find a valid process named '{}'", target_name))
}
