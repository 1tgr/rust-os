use core::mem;
use core::str;
use ptr::Align;

#[repr(C)]
struct Header {
    pub filename: [u8; 100],
    pub mode: [u8; 8],
    pub uid: [u8; 8],
    pub gid: [u8; 8],
    pub size: [u8; 12],
    pub mtime: [u8; 12],
    pub chksum: [u8; 8],
    pub typeflag: [u8; 1],
}

impl Header {
    pub fn parse_size(&self) -> usize {
        let mut size = 0usize;
        let mut j = 11;
        let mut count = 1;

        while j > 0 {
            size += (self.size[j - 1] - ('0' as u8)) as usize * count;
            j -= 1;
            count *= 8;
        }

        size
    }
}

fn nul_terminate(s: &[u8]) -> &[u8] {
    match s.iter().position(|b| *b == 0) {
        Some(index) => &s[0..index],
        None => s
    }
}

pub fn locate<'a>(data: &'a [u8], filename: &str) -> Option<&'a [u8]> {
    let mut offset = 0;
    while offset < data.len() {
        let header = unsafe {
            let header_slice = &data[offset .. offset + mem::size_of::<Header>()];
            &*(header_slice.as_ptr() as *const Header)
        };

        let header_filename = nul_terminate(&header.filename[..]);
        if header_filename.len() == 0 {
            break;
        }

        offset += 512;

        let size = header.parse_size();
        if let Ok(header_filename) = str::from_utf8(header_filename) {
            if header_filename == filename {
                return Some(&data[offset .. offset + size]);
            }
        }

        offset += Align::up(size, 512);
    }

    None
}
