// Mapper 000: https://wiki.nesdev.com/w/index.php/NROM

use crate::mappers::{Mapping, NametableMirror};
use crate::memory::{initialized_mem, mem, Mem};

// Mapper 000 supports ROM sizes of either 16 or 32 KB.
enum RomSize {
    Sixteen,
    ThirtyTwo,
}

pub struct Nrom {
    rom_size: RomSize,
    prg_ram: Option<Mem>,
    prg_rom: Mem,
    chr_rom: Mem,
    internal_vram: Mem,
    nametable_mirror: NametableMirror
}

impl Nrom {
    pub fn new(header: &[u8], rom_sections: &[u8]) -> Nrom {
        let prg_rom_size = header[4] as u16;
        let chr_rom_size = header[5] as u16;
        let prg_ram = match (header[6] & 0b0000_0100) == 1 {
            true => Some(initialized_mem(0x2000)),
            false => None,
        };
        let nametable_mirror = match (header[6] & 0b0000_0001) == 1 {
            true => NametableMirror::Vertical,
            false => NametableMirror::Horizontal
        };
        println!(
            "PRG ROM size: 0x{:X?}, CHR ROM size: 0x{:X?}, contains PRG RAM: {:?}, nametable mirroring: {:?}",
            prg_rom_size * 0x4000,
            chr_rom_size * 0x2000,
            prg_ram.is_some(),
            nametable_mirror
        );

        if (header[6] & 0b0000_1000) != 0 {
            unimplemented!("omg i have no idea what to do with a trainer");
        }

        let (rom_size, prg_rom, chr_rom) = match prg_rom_size {
            1 => (
                RomSize::Sixteen,
                &rom_sections[0..0x4000],
                &rom_sections[0x4000..(0x4000 + (0x2000 * chr_rom_size)) as usize],
            ),
            2 => (
                RomSize::ThirtyTwo,
                &rom_sections[0..0x8000],
                &rom_sections[0x8000..(0x8000 + (0x2000 * chr_rom_size)) as usize],
            ),
            _ => panic!(),
        };
        Nrom {
            rom_size,
            prg_ram,
            prg_rom: mem(prg_rom),
            chr_rom: mem(chr_rom),
            internal_vram: initialized_mem(0x1000),
            nametable_mirror
        }
    }

    fn mirrored_addr(&self, addr: u16) -> usize {
        self.nametable_mirror.mirrored_addr(addr) - 0x2000
    }

    #[cfg(test)]
    pub fn test_mapper(prg_rom: &[u8], chr_rom: &[u8]) -> Nrom {
        Nrom {
            rom_size: RomSize::Sixteen,
            prg_ram: None,
            prg_rom: mem(prg_rom),
            chr_rom: mem(chr_rom),
            internal_vram: initialized_mem(0x1000),
            nametable_mirror: NametableMirror::Horizontal
        }
    }
}

impl Mapping for Nrom {
    fn get_cpu_space(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x401F => panic!("Address {:X?} not handled by mappers!", addr),
            0x4020...0x5FFF => panic!("Address {:X?} unused by this mapper!", addr),
            0x6000...0x7FFF => self.prg_ram.as_ref().expect("ROM without RAM tried to read it!")[(addr - 0x6000) as usize],
            0x8000...0xBFFF => self.prg_rom[(addr - 0x8000) as usize],
            0xC000...0xFFFF => match self.rom_size {
                RomSize::Sixteen => self.prg_rom[(addr - 0xC000) as usize],
                RomSize::ThirtyTwo => self.prg_rom[(addr - 0x8000) as usize],
            },
        }
    }

    fn set_cpu_space(&mut self, addr: u16, value: u8) {
        match addr {
            0x6000...0x7FFF => self.prg_ram.as_mut().expect("ROM without RAM tried to write it!")[(addr - 0x6000) as usize] = value,
            _ => panic!("Tried to write to CPU address space outside RAM! (addr {:04X?})", addr),
        }
    }

    fn get_ppu_space(&self, addr: u16) -> u8 {
        match addr {
            0x0 ... 0x1FFF => self.chr_rom[addr as usize],
            0x2000 ... 0x2FFF => self.internal_vram[self.mirrored_addr(addr)],
            0x3000 ... 0x3EFF => self.internal_vram[(addr - 0x3000) as usize],
            _ => unimplemented!()
        }
    }

    fn set_ppu_space(&mut self, addr: u16, value: u8) {
        match addr {
            0x0 ... 0x1FFF => self.chr_rom[addr as usize] = value, // sometimes RAM, sometimes ROM
            0x2000 ... 0x2FFF => {
                let addr = self.mirrored_addr(addr);
                self.internal_vram[addr] = value
            },
            0x3000 ... 0x3EFF => self.internal_vram[(addr - 0x3000) as usize] = value,
            _ => unimplemented!()
        }
    }
}