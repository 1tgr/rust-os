use crate::bindings::*;
use crate::surface::{CairoSurface, CairoSurfaceMut};
use crate::CairoObj;
use alloc::borrow::Cow;
use alloc::string::ToString;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ptr::NonNull;
use libc::c_char;

fn to_cstr(s: &str) -> Cow<str> {
    if s.len() == 0 || !s.ends_with('\0') {
        let mut buf = s.to_string();
        buf.push_str("\0");
        Cow::Owned(buf)
    } else {
        Cow::Borrowed(s)
    }
}

pub struct Cairo<'a>(CairoObj<cairo_t>, PhantomData<CairoSurfaceMut<'a>>);

impl<'a> Cairo<'a> {
    pub fn new(surface: CairoSurfaceMut) -> Self {
        Cairo(CairoObj::wrap(unsafe { cairo_create(surface.as_ptr()) }), PhantomData)
    }

    pub fn save(&self) -> &Self {
        unsafe { cairo_save(self.0.as_ptr()) }
        self
    }

    pub fn restore(&self) -> &Self {
        unsafe { cairo_restore(self.0.as_ptr()) }
        self
    }

    pub fn fill(&self) -> &Self {
        unsafe { cairo_fill(self.0.as_ptr()) }
        self
    }

    pub fn clip(&self) -> &Self {
        unsafe { cairo_clip(self.0.as_ptr()) }
        self
    }

    pub fn clip_preserve(&self) -> &Self {
        unsafe { cairo_clip_preserve(self.0.as_ptr()) }
        self
    }

    pub fn reset_clip(&self) -> &Self {
        unsafe { cairo_reset_clip(self.0.as_ptr()) }
        self
    }

    pub fn new_path(&self) -> &Self {
        unsafe { cairo_new_path(self.0.as_ptr()) }
        self
    }

    pub fn new_sub_path(&self) -> &Self {
        unsafe { cairo_new_sub_path(self.0.as_ptr()) }
        self
    }

    pub fn close_path(&self) -> &Self {
        unsafe { cairo_close_path(self.0.as_ptr()) }
        self
    }

    pub fn move_to(&self, x: f64, y: f64) -> &Self {
        unsafe { cairo_move_to(self.0.as_ptr(), x, y) };
        self
    }

    pub fn line_to(&self, x: f64, y: f64) -> &Self {
        unsafe { cairo_line_to(self.0.as_ptr(), x, y) };
        self
    }

    pub fn rel_line_to(&self, dx: f64, dy: f64) -> &Self {
        unsafe { cairo_rel_line_to(self.0.as_ptr(), dx, dy) };
        self
    }

    pub fn translate(&self, tx: f64, ty: f64) -> &Self {
        unsafe { cairo_translate(self.0.as_ptr(), tx, ty) };
        self
    }

    pub fn paint(&self) -> &Self {
        unsafe { cairo_paint(self.0.as_ptr()) }
        self
    }

    pub fn stroke(&self) -> &Self {
        unsafe { cairo_stroke(self.0.as_ptr()) }
        self
    }

    pub fn rectangle(&self, x: f64, y: f64, width: f64, height: f64) -> &Self {
        unsafe { cairo_rectangle(self.0.as_ptr(), x, y, width, height) };
        self
    }

    pub fn set_source_rgb(&self, r: f64, g: f64, b: f64) -> &Self {
        unsafe { cairo_set_source_rgb(self.0.as_ptr(), r, g, b) };
        self
    }

    pub fn set_source_surface(&self, surface: &CairoSurface, x: f64, y: f64) -> &Self {
        unsafe { cairo_set_source_surface(self.0.as_ptr(), surface.as_ptr(), x, y) };
        self
    }

    pub fn font_extents(&self) -> cairo_font_extents_t {
        let mut extents = MaybeUninit::uninit();
        unsafe {
            cairo_font_extents(self.0.as_ptr(), extents.as_mut_ptr());
            extents.assume_init()
        }
    }

    pub fn text_extents(&self, text: &str) -> cairo_text_extents_t {
        let text = to_cstr(text);
        let mut extents = MaybeUninit::uninit();
        unsafe {
            cairo_text_extents(self.0.as_ptr(), text.as_ptr() as *const c_char, extents.as_mut_ptr());
            extents.assume_init()
        }
    }

    pub fn show_text(&self, text: &str) -> &Self {
        let text = to_cstr(text);
        unsafe { cairo_show_text(self.0.as_ptr(), text.as_ptr() as *const c_char) };
        self
    }
}

impl<'a> Deref for Cairo<'a> {
    type Target = NonNull<cairo_t>;

    fn deref(&self) -> &NonNull<cairo_t> {
        &self.0
    }
}
