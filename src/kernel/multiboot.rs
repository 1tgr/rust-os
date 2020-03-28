#![allow(non_camel_case_types)]
#![allow(dead_code)]
use core::mem;

pub type multiboot_uint8_t = ::libc::c_uchar;
pub type multiboot_uint16_t = ::libc::c_ushort;
pub type multiboot_uint32_t = ::libc::c_uint;
pub type multiboot_uint64_t = ::libc::c_ulonglong;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_multiboot_header {
    pub magic: multiboot_uint32_t,
    pub flags: multiboot_uint32_t,
    pub checksum: multiboot_uint32_t,
    pub header_addr: multiboot_uint32_t,
    pub load_addr: multiboot_uint32_t,
    pub load_end_addr: multiboot_uint32_t,
    pub bss_end_addr: multiboot_uint32_t,
    pub entry_addr: multiboot_uint32_t,
    pub mode_type: multiboot_uint32_t,
    pub width: multiboot_uint32_t,
    pub height: multiboot_uint32_t,
    pub depth: multiboot_uint32_t,
}
impl Clone for Struct_multiboot_header {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_header {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct Struct_multiboot_aout_symbol_table {
    pub tabsize: multiboot_uint32_t,
    pub strsize: multiboot_uint32_t,
    pub addr: multiboot_uint32_t,
    pub reserved: multiboot_uint32_t,
}
impl Clone for Struct_multiboot_aout_symbol_table {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_aout_symbol_table {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
pub type multiboot_aout_symbol_table_t = Struct_multiboot_aout_symbol_table;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_multiboot_elf_section_header_table {
    pub num: multiboot_uint32_t,
    pub size: multiboot_uint32_t,
    pub addr: multiboot_uint32_t,
    pub shndx: multiboot_uint32_t,
}
impl Clone for Struct_multiboot_elf_section_header_table {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_elf_section_header_table {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
pub type multiboot_elf_section_header_table_t = Struct_multiboot_elf_section_header_table;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_multiboot_info {
    pub flags: multiboot_uint32_t,
    pub mem_lower: multiboot_uint32_t,
    pub mem_upper: multiboot_uint32_t,
    pub boot_device: multiboot_uint32_t,
    pub cmdline: multiboot_uint32_t,
    pub mods_count: multiboot_uint32_t,
    pub mods_addr: multiboot_uint32_t,
    pub u: Union_Unnamed1,
    pub mmap_length: multiboot_uint32_t,
    pub mmap_addr: multiboot_uint32_t,
    pub drives_length: multiboot_uint32_t,
    pub drives_addr: multiboot_uint32_t,
    pub config_table: multiboot_uint32_t,
    pub boot_loader_name: multiboot_uint32_t,
    pub apm_table: multiboot_uint32_t,
    pub vbe_control_info: multiboot_uint32_t,
    pub vbe_mode_info: multiboot_uint32_t,
    pub vbe_mode: multiboot_uint16_t,
    pub vbe_interface_seg: multiboot_uint16_t,
    pub vbe_interface_off: multiboot_uint16_t,
    pub vbe_interface_len: multiboot_uint16_t,
    pub framebuffer_addr: multiboot_uint64_t,
    pub framebuffer_pitch: multiboot_uint32_t,
    pub framebuffer_width: multiboot_uint32_t,
    pub framebuffer_height: multiboot_uint32_t,
    pub framebuffer_bpp: multiboot_uint8_t,
    pub framebuffer_type: multiboot_uint8_t,
    pub _bindgen_data_1_: [u32; 2usize],
}
impl Struct_multiboot_info {
    pub unsafe fn framebuffer_palette_addr(&mut self) -> *mut multiboot_uint32_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(0))
    }
    pub unsafe fn framebuffer_palette_num_colors(&mut self) -> *mut multiboot_uint16_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(4))
    }
    pub unsafe fn framebuffer_red_field_position(&mut self) -> *mut multiboot_uint8_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(0))
    }
    pub unsafe fn framebuffer_red_mask_size(&mut self) -> *mut multiboot_uint8_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(1))
    }
    pub unsafe fn framebuffer_green_field_position(&mut self) -> *mut multiboot_uint8_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(2))
    }
    pub unsafe fn framebuffer_green_mask_size(&mut self) -> *mut multiboot_uint8_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(3))
    }
    pub unsafe fn framebuffer_blue_field_position(&mut self) -> *mut multiboot_uint8_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(4))
    }
    pub unsafe fn framebuffer_blue_mask_size(&mut self) -> *mut multiboot_uint8_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_1_);
        mem::transmute(raw.offset(5))
    }
}
impl Clone for Struct_multiboot_info {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_info {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct Union_Unnamed1 {
    pub _bindgen_data_: [u32; 4usize],
}
impl Union_Unnamed1 {
    pub unsafe fn aout_sym(&mut self) -> *mut multiboot_aout_symbol_table_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_);
        mem::transmute(raw.offset(0))
    }
    pub unsafe fn elf_sec(&mut self) -> *mut multiboot_elf_section_header_table_t {
        let raw: *mut u8 = mem::transmute(&self._bindgen_data_);
        mem::transmute(raw.offset(0))
    }
}
impl Clone for Union_Unnamed1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Union_Unnamed1 {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
pub type multiboot_info_t = Struct_multiboot_info;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_multiboot_color {
    pub red: multiboot_uint8_t,
    pub green: multiboot_uint8_t,
    pub blue: multiboot_uint8_t,
}
impl Clone for Struct_multiboot_color {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_color {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
#[repr(C, packed)]
#[derive(Copy)]
pub struct Struct_multiboot_mmap_entry {
    pub size: multiboot_uint32_t,
    pub addr: multiboot_uint64_t,
    pub len: multiboot_uint64_t,
    pub _type: multiboot_uint32_t,
}
impl Clone for Struct_multiboot_mmap_entry {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_mmap_entry {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
pub type multiboot_memory_map_t = Struct_multiboot_mmap_entry;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_multiboot_mod_list {
    pub mod_start: multiboot_uint32_t,
    pub mod_end: multiboot_uint32_t,
    pub cmdline: multiboot_uint32_t,
    pub pad: multiboot_uint32_t,
}
impl Clone for Struct_multiboot_mod_list {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_mod_list {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
pub type multiboot_module_t = Struct_multiboot_mod_list;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_multiboot_apm_info {
    pub version: multiboot_uint16_t,
    pub cseg: multiboot_uint16_t,
    pub offset: multiboot_uint32_t,
    pub cseg_16: multiboot_uint16_t,
    pub dseg: multiboot_uint16_t,
    pub flags: multiboot_uint16_t,
    pub cseg_len: multiboot_uint16_t,
    pub cseg_16_len: multiboot_uint16_t,
    pub dseg_len: multiboot_uint16_t,
}
impl Clone for Struct_multiboot_apm_info {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Struct_multiboot_apm_info {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
