use volatile::ReadWrite;
use x86_64::instructions::port::{Port, PortGeneric, ReadWriteAccess};

//This module is a stub.
mod pci_scsi;

///Read a word from PCI config. Adapted from https://wiki.osdev.org/Pci#The_PCI_Bus.
pub fn pciConfigReadWord(bus: u8, slot: u8, func: u8, offset: u8)->u16{
    let long_bus = bus as u32;
    let long_slot = slot as u32;
    let long_func = func as u32;
    let mut tmp = 0u16;
    //Create the address as per figure 1 at the link above.
    let address = ((long_bus << 16) | (long_slot << 11)
        | (long_func << 8) | (offset & 0xFC) as u32 | (0x80000000 as u32)) as u32;
    //Port to be used for the address.
    let mut port1:PortGeneric<u32, ReadWriteAccess> = Port::new(0xCF8);
    unsafe {
        //Write the address
        port1.write(address);
    }
    //Port to be used for the data.
    let mut port2: PortGeneric<u32, ReadWriteAccess> = Port::new(0xCFC);
    let inl = unsafe { port2.read() };
    tmp = ((inl >> ((offset & 2) * 8)) & 0xFFFF) as u16;
    tmp
}