//REFERENCE MATERIALS: https://wiki.osdev.org/Pci

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::asm;
use volatile::ReadWrite;
use x86_64::instructions::port::{Port, PortGeneric, ReadWriteAccess};
use crate::println;

//This module is a stub.
mod pci_scsi;

pub struct PCIDevice{
    bus: u8,
    device: u8,
    vendor_id: u16,
    device_id: u16,
    class_id: u8,
    subclass_id: u8,
}

pub fn detect_pci_compat_bios()->bool{
    /*
    //this would in theory, according to the wiki, check if pci is possible. However, this leads to a double fault.
    //FIXME: PCI configuration space detection
    let mut al = 0u8;
    let mut ah = 0u8;
    unsafe {
        asm!(
        "cli",
        "int 0x1a",
        "mov {l}, al",
        "mov {h}, ah",
        "sti",
        l = in(reg_byte) al,
        h = in(reg_byte) ah,
        )
    }
    let ax: u16 = ((ah as u16) << 8) | (al as u16);
    println!("after pci check: ax == {}",ax);
     */
    true
}

///Read a word from PCI config. Adapted from https://wiki.osdev.org/Pci#The_PCI_Bus.
pub fn pci_config_read_word(bus: u8, slot: u8, func: u8, offset: u8) ->u16{
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

fn get_vendor_id(bus: u8, device: u8, function: u8) -> u16{
    pci_config_read_word(bus, device, function, 0)
}
fn get_device_id(bus: u8, device: u8, function: u8) -> u16{
    pci_config_read_word(bus, device, function, 0x2)
}
fn check_pci_device_class(bus: u8, device: u8, function: u8) -> u8{
    pci_config_read_word(bus, device, function, 0xA).to_le_bytes()[0]
}
fn check_pci_device_subclass(bus: u8, device: u8, function: u8) -> u8{
    pci_config_read_word(bus, device, function, 0xA).to_le_bytes()[1]
}

fn check_device(bus: u8, device: u8) -> Option<PCIDevice>{
    //FIXME: multiple function devices
    let vendor_id = get_vendor_id(bus, device, 0);
    if vendor_id == 0xFFFF{
        return None;
    }
    //Device vendor is valid
    let device_id = get_device_id(bus, device, 0);
    let class_id = check_pci_device_class(bus, device, 0);
    let subclass_id = check_pci_device_subclass(bus, device, 0);
    println!("PCI DEVICE FOUND, vendor id = {:#x}, device id = {:#x}, class = {:#x}, subclass = {:#x}", vendor_id, device_id, class_id, subclass_id);
    Some(PCIDevice{
        bus,
        device,
        vendor_id,
        device_id,
        class_id,
        subclass_id
    })
}
pub fn check_all_pci_devices() -> Box<Vec<PCIDevice>>{
    let mut devices:Vec<PCIDevice> = Vec::new();
    for bus in 0..=255u8{
        for device in 0..32u8{
            match check_device(bus, device){
                Some(d) => devices.push(d),
                None => {}
            };
        }
    }
    Box::new(devices)
}