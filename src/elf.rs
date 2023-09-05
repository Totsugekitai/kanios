use crate::{memory::PAGE_SIZE, types::PhysAddr, utils::align_up};
use core::slice;

#[repr(C, packed)]
#[derive(Debug)]
pub struct ElfHeader {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl ElfHeader {
    pub fn count_pages(&self) -> usize {
        let phoff = self.e_phoff as isize;
        let phnum = self.e_phnum as isize;
        let phdr = unsafe { (self as *const ElfHeader as *const u8).offset(phoff) }
            as *const ProgramHeader;
        let start_vaddr = unsafe { phdr.as_ref().unwrap().p_vaddr };
        let mut end_vaddr = start_vaddr;
        for i in 0..phnum {
            let phdr = unsafe { phdr.offset(i).as_ref().unwrap() };
            let vaddr = phdr.p_vaddr;
            let memsz = phdr.p_memsz;
            if end_vaddr < vaddr + memsz {
                end_vaddr = vaddr + memsz;
            }
        }

        (align_up(end_vaddr - start_vaddr, PAGE_SIZE) / PAGE_SIZE) as usize
    }

    pub unsafe fn load(&self, page_paddr: PhysAddr) {
        let phoff = self.e_phoff as isize;
        let phnum = self.e_phnum as isize;
        let phdr = unsafe { (self as *const ElfHeader as *const u8).offset(phoff) }
            as *const ProgramHeader;

        let mut paddr = page_paddr;
        for i in 0..phnum {
            let phdr = unsafe { phdr.offset(i).as_ref().unwrap() };
            let off = phdr.p_offset;
            let filesz = phdr.p_filesz;
            let data = slice::from_raw_parts(
                (self as *const ElfHeader as *const u8).offset(off as isize),
                filesz as usize,
            );
            let pa_slice = slice::from_raw_parts_mut(paddr.to_u64() as *mut u8, filesz as usize);
            pa_slice.copy_from_slice(data);

            paddr += PhysAddr(filesz);
        }
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}
