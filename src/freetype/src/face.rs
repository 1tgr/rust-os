use crate::bindings::*;
use crate::freetype::FreeType;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::mem::MaybeUninit;

pub struct Face<'a> {
    pub(crate) face: FT_Face,
    pub(crate) _data: Cow<'a, [u8]>,
}

impl<'a> Drop for Face<'a> {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Face(self.face);
        }
    }
}

impl<'a> Face<'a> {
    fn new(ft: &mut FreeType, data: Cow<'a, [u8]>, face_index: usize) -> Self {
        let mut face = MaybeUninit::zeroed();

        let face = unsafe {
            FT_New_Memory_Face(
                ft.borrow_ptr(),
                data.as_ptr(),
                data.len() as i64,
                face_index as i64,
                face.as_mut_ptr(),
            );
            face.assume_init()
        };

        Self { face, _data: data }
    }

    pub fn from_slice(ft: &mut FreeType, data: &'a [u8], face_index: usize) -> Face<'a> {
        Self::new(ft, Cow::Borrowed(data), face_index)
    }

    pub fn set_char_size(
        &mut self,
        char_width: f64,
        char_height: f64,
        horz_resolution: u32,
        vert_resolution: u32,
    ) -> &mut Self {
        unsafe {
            FT_Set_Char_Size(
                self.face,
                (char_width * 64.0) as FT_F26Dot6,
                (char_height * 64.0) as FT_F26Dot6,
                horz_resolution,
                vert_resolution,
            );
        }
        self
    }

    pub fn borrow_ptr(&self) -> FT_Face {
        self.face
    }

    pub fn to_owned_ptr(&self) -> FT_Face {
        unsafe {
            FT_Reference_Face(self.face);
        }
        self.face
    }
}

impl Face<'static> {
    pub fn from_vec(ft: &mut FreeType, data: Vec<u8>, face_index: usize) -> Self {
        Self::new(ft, Cow::Owned(data), face_index)
    }
}
