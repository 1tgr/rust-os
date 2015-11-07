use bindings::*;
use surface::CairoSurface;
use CairoObj;
use core::ops::Deref;

pub struct Cairo(CairoObj<cairo_t>);

impl Cairo {
    pub fn new(surface: CairoSurface) -> Self {
        Cairo(CairoObj::wrap(unsafe { cairo_create(*surface) }))
    }

    pub fn fill(&self) -> &Self {
        unsafe { cairo_fill(*self.0) }
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
}

impl Deref for Cairo {
    type Target = *mut cairo_t;

    fn deref(&self) -> &*mut cairo_t {
        &*self.0
    }
}
