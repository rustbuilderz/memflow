use anyhow::Result;
use memflow::mem::virt_mem::VirtualMemory;
use memflow_win32::win32::Kernel;
use memflow_qemu_procfs::QemuProcfs;

fn main() -> Result<()> {
    let mut connector = QemuProcfs::new()?;
    let mut kernel = Kernel::builder(&mut connector).build()?;

    let mut process = kernel.into_process("Muck.exe")?;

    let modules = process.module_list()?;
    let module = modules.iter()
    .find(|m| m.name.to_ascii_lowercase().contains("mono-2.0-bdwgc"))
    .expect("mono-2.0-bdwgc.dll not found");

    let mut addr = module.base + 0x00496DA8;
    let offsets = [0x64, 0xB8, 0x20, 0x60, 0xB0, 0xE20, 0x70];

    let mut virt = process.virt_mem;

    for &offset in &offsets[..offsets.len() - 1] {
        addr = virt.virt_read_addr64(addr + offset)?;
    }

    let final_value: i32 = virt.virt_read(addr + offsets[offsets.len() - 1])?;
    println!("INF-STAMINA Value: {}", final_value);

    Ok(())
}
