use crate::bindings::*;
use core::mem::MaybeUninit;

pub struct FreeType {
    library: FT_Library,
}

impl FreeType {
    pub fn new() -> Self {
        let mut library = MaybeUninit::zeroed();

        let library = unsafe {
            FT_Init_FreeType(library.as_mut_ptr());
            library.assume_init()
        };

        Self { library }
    }
}

impl Drop for FreeType {
    fn drop(&mut self) {
        unsafe {
            FT_Done_FreeType(self.library);
        }
    }
}

impl FreeType {
    pub fn borrow_ptr(&self) -> FT_Library {
        self.library
    }
}
