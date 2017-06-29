use cairo::surface::CairoSurface;
use client::{self,Client};
use frame_buffer::FrameBuffer;
use os::{Result,SharedMem};
use types::{Command,Rect};

pub struct Window<'a> {
    client: &'a Client,
    id: usize,
    x: f64,
    y: f64,
    buffer: FrameBuffer,
}

impl<'a> Window<'a> {
    pub fn new(client: &'a Client, x: f64, y: f64, width: f64, height: f64) -> Result<Self> {
        let id = client::alloc_id();
        let shared_mem = SharedMem::new(true)?;
        client.send_command(Command::CreateWindow {
            id,
            pos: Rect {
                x: x,
                y: y,
                width: width,
                height: height,
            },
            shared_mem_handle: shared_mem.handle().get()
        })?;

        let buffer = FrameBuffer::new(width, height, shared_mem)?;
        Ok(Window { client, id, x, y, buffer, })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn invalidate(&self) -> Result<()> {
        self.client.send_command(Command::InvalidateWindow { id: self.id })
    }

    pub fn resize(&mut self, pos: Rect) -> Result<()> {
        self.x = pos.x;
        self.y = pos.y;
        self.buffer.resize(pos.width, pos.height)?;
        self.client.send_command(Command::MoveWindow { id: self.id, pos })
    }

    pub fn create_surface(&mut self) -> CairoSurface {
        self.buffer.create_surface()
    }
}

impl<'a> Drop for Window<'a> {
    fn drop(&mut self) {
        let _ = self.client.send_command(Command::DestroyWindow { id: self.id });
    }
}
