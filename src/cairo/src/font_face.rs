use crate::bindings::*;
use crate::CairoObj;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;
use freetype::bindings::*;
use freetype::Face;
use libc::c_void;

pub struct FontFace<'a>(CairoObj<cairo_font_face_t>, PhantomData<Face<'a>>);

impl<'a> FontFace<'a> {
    pub fn from_freetype(face: &Face) -> Self {
        let font_face = unsafe {
            let ft_face = face.to_owned_ptr();
            let font_face = cairo_ft_font_face_create_for_ft_face(ft_face, 0);
            static KEY: cairo_user_data_key_t = cairo_user_data_key_t { unused: 0 };

            unsafe extern "C" fn destroy(ptr: *mut c_void) {
                let ft_face = ptr as *const c_void as FT_Face;
                FT_Done_Face(ft_face);
            }

            cairo_font_face_set_user_data(font_face, &KEY, ft_face as *const c_void as *mut c_void, Some(destroy));
            font_face
        };

        Self(CairoObj::wrap(font_face), PhantomData)
    }
}

impl<'a> Deref for FontFace<'a> {
    type Target = NonNull<cairo_font_face_t>;

    fn deref(&self) -> &NonNull<cairo_font_face_t> {
        &self.0
    }
}
