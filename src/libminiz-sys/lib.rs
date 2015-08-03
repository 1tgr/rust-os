#![crate_name = "miniz_sys"]
#![allow(non_camel_case_types)]

extern crate libc;

pub const MZ_NO_FLUSH: libc::c_int = 0;
pub const MZ_SYNC_FLUSH: libc::c_int = 2;
pub const MZ_FINISH: libc::c_int = 4;

pub const MZ_OK: libc::c_int = 0;
pub const MZ_STREAM_END: libc::c_int = 1;
pub const MZ_NEED_DICT: libc::c_int = 2;
pub const MZ_ERRNO: libc::c_int = -1;
pub const MZ_STREAM_ERROR: libc::c_int = -2;
pub const MZ_DATA_ERROR: libc::c_int = -3;
pub const MZ_MEM_ERROR: libc::c_int = -4;
pub const MZ_BUF_ERROR: libc::c_int = -5;
pub const MZ_VERSION_ERROR: libc::c_int = -6;
pub const MZ_PARAM_ERROR: libc::c_int = -10000;

pub const MZ_DEFLATED: libc::c_int = 8;
pub const MZ_DEFAULT_WINDOW_BITS: libc::c_int = 15;
pub const MZ_DEFAULT_STRATEGY: libc::c_int = 0;

#[repr(C)]
pub struct mz_stream {
    pub next_in: *const u8,
    pub avail_in: libc::c_uint,
    pub total_in: libc::c_ulong,

    pub next_out: *mut u8,
    pub avail_out: libc::c_uint,
    pub total_out: libc::c_ulong,

    pub msg: *const u8,
    pub state: *mut mz_internal_state,

    pub zalloc: Option<mz_alloc_func>,
    pub zfree: Option<mz_free_func>,
    pub opaque: *mut libc::c_void,

    pub data_type: libc::c_int,
    pub adler: libc::c_ulong,
    pub reserved: libc::c_ulong,
}

pub enum mz_internal_state {}

pub type mz_alloc_func = extern fn(*mut libc::c_void,
                                   libc::size_t,
                                   libc::size_t) -> *mut libc::c_void;
pub type mz_free_func = extern fn(*mut libc::c_void, *mut libc::c_void);
pub type mz_realloc_func = extern fn(*mut libc::c_void,
                                     *mut libc::c_void,
                                     libc::size_t,
                                     libc::size_t) -> *mut libc::c_void;

pub const MZ_ZIP_MAX_IO_BUF_SIZE: libc::c_uint = 65536;
pub const MZ_ZIP_MAX_ARCHIVE_FILENAME_SIZE: libc::c_uint = 260;
pub const MZ_ZIP_MAX_ARCHIVE_FILE_COMMENT_SIZE: libc::c_uint = 256;

#[repr(C)]
pub struct mz_zip_archive_file_stat {
    pub file_index: libc::c_uint,
    pub central_dir_ofs: libc::c_uint,
    pub version_made_by: libc::c_ushort,
    pub version_needed: libc::c_ushort,
    pub bit_flag: libc::c_ushort,
    pub method: libc::c_ushort,
    pub crc32: libc::c_uint,
    pub comp_size: libc::c_ulonglong,
    pub uncomp_size: libc::c_ulonglong,
    pub internal_attr: libc::c_ushort,
    pub external_attr: libc::c_uint,
    pub local_header_ofs: libc::c_ulonglong,
    pub comment_size: libc::c_uint,
    pub filename: [u8; 260usize],
    pub comment: [u8; 256usize],
}

pub type mz_file_read_func = extern fn(opaque: *mut libc::c_void,
                                       file_ofs: libc::c_ulonglong,
                                       buf: *mut libc::c_void, n: libc::size_t) -> libc::size_t;
pub type mz_file_write_func = extern fn(opaque: *mut libc::c_void,
                                        file_ofs: libc::c_ulonglong,
                                        buf: *const libc::c_void,
                                        n: libc::size_t) -> libc::size_t;

pub enum mz_zip_internal_state { }

pub const MZ_ZIP_MODE_INVALID: libc::c_uint = 0;
pub const MZ_ZIP_MODE_READING: libc::c_uint = 1;
pub const MZ_ZIP_MODE_WRITING: libc::c_uint = 2;
pub const MZ_ZIP_MODE_WRITING_HAS_BEEN_FINALIZED: libc::c_uint = 3;

#[repr(C)]
pub struct mz_zip_archive {
    pub archive_size: libc::c_ulonglong,
    pub central_directory_file_ofs: libc::c_ulonglong,
    pub total_files: libc::c_uint,
    pub zip_mode: libc::c_uint,
    pub file_offset_alignment: libc::c_uint,
    pub alloc: mz_alloc_func,
    pub free: mz_free_func,
    pub realloc: mz_realloc_func,
    pub alloc_opaque: *mut libc::c_void,
    pub read: Option<mz_file_read_func>,
    pub write: Option<mz_file_write_func>,
    pub io_opaque: *mut libc::c_void,
    pub state: *mut mz_zip_internal_state,
}

pub const MZ_ZIP_FLAG_CASE_SENSITIVE: libc::c_uint = 256;
pub const MZ_ZIP_FLAG_IGNORE_PATH: libc::c_uint = 512;
pub const MZ_ZIP_FLAG_COMPRESSED_DATA: libc::c_uint = 1024;
pub const MZ_ZIP_FLAG_DO_NOT_SORT_CENTRAL_DIRECTORY: libc::c_uint = 2048;

#[link(name = "miniz")]
extern {
    pub fn mz_deflateInit2(stream: *mut mz_stream,
                           level: libc::c_int,
                           method: libc::c_int,
                           window_bits: libc::c_int,
                           mem_level: libc::c_int,
                           strategy: libc::c_int) -> libc::c_int;
    pub fn mz_deflate(stream: *mut mz_stream, flush: libc::c_int) -> libc::c_int;
    pub fn mz_deflateEnd(stream: *mut mz_stream) -> libc::c_int;

    pub fn mz_inflateInit2(stream: *mut mz_stream,
                           window_bits: libc::c_int) -> libc::c_int;
    pub fn mz_inflate(stream: *mut mz_stream, flush: libc::c_int) -> libc::c_int;
    pub fn mz_inflateEnd(stream: *mut mz_stream) -> libc::c_int;

    pub fn mz_crc32(crc: libc::c_ulong, ptr: *const u8,
                    len: libc::size_t) -> libc::c_ulong;

    pub fn mz_zip_reader_init(zip: *mut mz_zip_archive, size: libc::c_ulonglong,
                              flags: libc::c_uint) -> bool;
    pub fn mz_zip_reader_init_mem(zip: *mut mz_zip_archive,
                                  mem: *const libc::c_void, size: libc::size_t,
                                  flags: libc::c_uint) -> bool;
    pub fn mz_zip_reader_get_num_files(zip: *mut mz_zip_archive) -> libc::c_uint;
    pub fn mz_zip_reader_file_stat(zip: *mut mz_zip_archive,
                                   file_index: libc::c_uint,
                                   stat: *mut mz_zip_archive_file_stat) -> bool;
    pub fn mz_zip_reader_is_file_a_directory(zip: *mut mz_zip_archive,
                                             file_index: libc::c_uint) -> bool;
    pub fn mz_zip_reader_is_file_encrypted(zip: *mut mz_zip_archive,
                                           file_index: libc::c_uint) -> bool;
    pub fn mz_zip_reader_get_filename(zip: *mut mz_zip_archive,
                                      file_index: libc::c_uint,
                                      filename: *mut u8,
                                      filename_buf_size: libc::c_uint) -> libc::c_uint;
    pub fn mz_zip_reader_locate_file(zip: *mut mz_zip_archive,
                                     name: *const u8,
                                     comment: *const u8,
                                     flags: libc::c_uint) -> libc::c_int;
    pub fn mz_zip_reader_extract_to_mem_no_alloc(zip: *mut mz_zip_archive,
                                                 file_index: libc::c_uint,
                                                 buf: *mut libc::c_void,
                                                 buf_size: libc::size_t,
                                                 flags: libc::c_uint,
                                                 user_read_buf: *mut libc::c_void,
                                                 user_read_buf_size: libc::size_t) -> bool;
    pub fn mz_zip_reader_extract_file_to_mem_no_alloc(zip: *mut mz_zip_archive,
                                                      filename: *const u8,
                                                      buf: *mut libc::c_void,
                                                      buf_size: libc::size_t,
                                                      flags: libc::c_uint,
                                                      user_read_buf: *mut libc::c_void,
                                                      user_read_buf_size: libc::size_t) -> bool;
    pub fn mz_zip_reader_extract_to_mem(zip: *mut mz_zip_archive,
                                        file_index: libc::c_uint,
                                        buf: *mut libc::c_void,
                                        buf_size: libc::size_t, flags: libc::c_uint) -> bool;
    pub fn mz_zip_reader_extract_file_to_mem(zip: *mut mz_zip_archive,
                                             filename: *const u8,
                                             buf: *mut libc::c_void,
                                             buf_size: libc::size_t, flags: libc::c_uint) -> bool;
    pub fn mz_zip_reader_extract_to_heap(zip: *mut mz_zip_archive,
                                         file_index: libc::c_uint,
                                         size: *mut libc::size_t, flags: libc::c_uint) -> *mut libc::c_void;
    pub fn mz_zip_reader_extract_file_to_heap(zip: *mut mz_zip_archive,
                                              filename: *const u8,
                                              size: *mut libc::size_t,
                                              flags: libc::c_uint) -> *mut libc::c_void;
    pub fn mz_zip_reader_extract_to_callback(zip: *mut mz_zip_archive,
                                             file_index: libc::c_uint,
                                             callback: mz_file_write_func,
                                             opaque: *mut libc::c_void,
                                             flags: libc::c_uint) -> bool;
    pub fn mz_zip_reader_extract_file_to_callback(zip: *mut mz_zip_archive,
                                                  filename: *const u8,
                                                  callback: mz_file_write_func,
                                                  opaque: *mut libc::c_void,
                                                  flags: libc::c_uint) -> bool;
    pub fn mz_zip_reader_end(zip: *mut mz_zip_archive) -> bool;
}
