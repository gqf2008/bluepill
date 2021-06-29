//#define STARTADDR 0x0800F000 //STM32F103C8T6适用

#![no_std]

use stm32f1xx_hal::pac::FLASH;

pub struct Flash {
    flash: FLASH,
    pub sector: u8,
}

#[derive(Debug, Clone)]
pub struct FlashError {
    status: u16,
}

// ///All errors contain raw value of the FLASH_SR status register (lower 16b)
// impl Flash {
//     pub fn new(flash: FLASH, sector: u8) -> Self {
//         debug_assert!(sector < 12, "invalid sector {}", sector);

//         let flash = Flash { flash, sector };

//         unsafe { flash.init() };

//         flash
//     }

//     unsafe fn init(&self) {
//         self.flash.keyr.write(|w| w.key().bits(0x4567_0123));
//         self.flash.keyr.write(|w| w.key().bits(0xCDEF_89AB));

//         self.flash.cr.modify(|_, w| w.psize().psize32());
//     }

//     pub fn erase(&self) -> Result<(), u16> {
//         while self.flash.sr.read().bsy().bit_is_set() {}

//         self.flash.cr.modify(|_, w| {
//             w.ser().set_bit();
//             unsafe { w.snb().bits(self.sector) }
//         });
//         self.flash.cr.modify(|_, w| w.strt().set_bit());

//         while self.flash.sr.read().bsy().bit_is_set() {}

//         self.flash.cr.modify(|_, w| w.strt().clear_bit());

//         let status = self.flash.sr.read();
//         if status.wrperr().bit_is_set() {
//             self.flash.sr.modify(|_, w| w.wrperr().set_bit());
//             return Err(status.bits() as u16);
//         }

//         Ok(())
//     }

//     fn get_address(&self, offset: usize, access_size: usize) -> usize {
//         let (size, address) = match self.sector {
//             //RM0090 Rev 18 page 75
//             0..=3 => (0x4000, 0x0800_0000 + self.sector as usize * 0x4000),
//             4 => (0x1_0000, 0x0801_0000),
//             5..=11 => (
//                 0x2_0000,
//                 0x0802_0000 + (self.sector - 5) as usize * 0x2_0000,
//             ),
//             _ => panic!("invalid sector {}", self.sector),
//         };

//         debug_assert!(offset + access_size < size, "access beyond sector limits");

//         address + offset
//     }

//     pub fn write<T>(&self, offset: usize, data: &T) -> Result<(), u16> {
//         let size = core::mem::size_of::<T>();
//         let src_ptr = (data as *const T) as *const u32;
//         let dest_ptr = Flash::get_address(self, offset, size) as *mut u32;

//         debug_assert!(size % 4 == 0, "data size not 4-byte aligned");
//         debug_assert!(src_ptr as usize % 4 == 0, "data address not 4-byte aligned");

//         while self.flash.sr.read().bsy().bit_is_set() {}

//         //check if register operations can be moved out of the loop
//         for i in 0..size as isize / 4 {
//             self.flash.cr.modify(|_, w| w.pg().set_bit());
//             unsafe {
//                 *dest_ptr.offset(i) = *src_ptr.offset(i);
//             }
//             while self.flash.sr.read().bsy().bit_is_set() {}

//             let status = self.flash.sr.read();
//             if status.wrperr().bit_is_set()
//                 || status.pgaerr().bit_is_set()
//                 || status.pgperr().bit_is_set()
//                 || status.pgserr().bit_is_set()
//             {
//                 self.flash.sr.write(|w| unsafe { w.bits(0xFFFF) });
//                 return Err(status.bits() as u16);
//             }
//         }

//         self.flash.cr.modify(|_, w| w.pg().clear_bit());

//         Ok(())
//     }

//     pub fn read<T>(&self, offset: usize) -> T {
//         let size = core::mem::size_of::<T>();
//         let ptr = Flash::get_address(self, offset, size) as *const u8;
//         unsafe { core::ptr::read(ptr as *const _) }
//     }

//     pub fn read_into<T>(&self, offset: usize, dest: &mut T) {
//         let size = core::mem::size_of::<T>();
//         let ptr = Flash::get_address(self, offset, size) as *const u8;
//         unsafe {
//             core::ptr::copy_nonoverlapping(ptr as *const _, dest, 1);
//         };
//     }
// }
