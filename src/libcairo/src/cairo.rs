use crate::bindings::*;
use crate::surface::CairoSurface;
use crate::CairoObj;
use alloc::string::ToString;
use core::ops::Deref;
use core::ptr::NonNull;

pub struct Cairo(CairoObj<cairo_t>);

impl Cairo {
    pub fn new(surface: CairoSurface) -> Self {
        Cairo(CairoObj::wrap(unsafe { cairo_create(surface.as_ptr()) }))
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

    pub fn show_text(&self, text: &str) -> &Self {
        let mut buf;
        let text = if text.len() == 0 || !text.ends_with('\0') {
            buf = text.to_string();
            buf.push_str("\0");
            buf.as_str()
        } else {
            text
        };

        unsafe { cairo_show_text(self.0.as_ptr(), text.as_bytes().as_ptr() as *const i8) };
        self
    }
}

impl Deref for Cairo {
    type Target = NonNull<cairo_t>;

    fn deref(&self) -> &NonNull<cairo_t> {
        &self.0
    }
}
