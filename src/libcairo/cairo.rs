use bindings::*;
use CairoObj;
use collections::string::ToString;
use core::ops::Deref;
use surface::CairoSurface;

pub struct Cairo(CairoObj<cairo_t>);

impl Cairo {
    pub fn new(surface: CairoSurface) -> Self {
        Cairo(CairoObj::wrap(unsafe { cairo_create(*surface) }))
    }

    pub fn fill(&self) -> &Self {
        unsafe { cairo_fill(*self.0) }
        self
    }

    pub fn move_to(&self, x: f64, y: f64) -> &Self {
        unsafe { cairo_move_to(*self.0, x, y) };
        self
    }

    pub fn paint(&self) -> &Self {
        unsafe { cairo_paint(*self.0) }
        self
    }

    pub fn rectangle(&self, x: f64, y: f64, width: f64, height: f64) -> &Self {
        unsafe { cairo_rectangle(*self.0, x, y, width, height) };
        self
    }

    pub fn set_source_rgb(&self, r: f64, g: f64, b: f64) -> &Self {
        unsafe { cairo_set_source_rgb(*self.0, r, g, b) };
        self
    }

    pub fn set_source_surface(&self, surface: CairoSurface, x: f64, y: f64) -> &Self {
        unsafe { cairo_set_source_surface(*self.0, *surface, x, y) };
        self
    }

    pub fn show_text(&self, text: &str) -> &Self {
        let mut buf;
        let text =
            if text.len() == 0 || text.char_at_reverse(0) != '\0' {
                buf = text.to_string();
                buf.push_str("\0");
                buf.as_str()
            } else {
                text
            };

        unsafe { cairo_show_text(*self.0, text.as_bytes().as_ptr() as *const i8) };
        self
    }
}

impl Deref for Cairo {
    type Target = *mut cairo_t;

    fn deref(&self) -> &*mut cairo_t {
        &*self.0
    }
}
