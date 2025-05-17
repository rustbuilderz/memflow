use anyhow::Result;
use memflow::types::{PhysicalAddress, Address};
use memflow_qemu_procfs::QemuProcfs;
use memflow::PhysicalMemory;

fn main() -> Result<()> {
    // Create QemuProcfs instance (use new() or with_guest_name())
    let mut mem = QemuProcfs::new()?;

    // Example: read 0x1000 physical memory from address 0x1000
    let addr = PhysicalAddress::from(0x1000u64);
    let mut buffer = [0u8; 4096];
    mem.phys_read_raw_into(addr, &mut buffer)?;

    println!("Read memory: {:x?}", &buffer[..16]);

    Ok(())
}
