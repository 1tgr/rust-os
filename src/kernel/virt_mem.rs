use core::mem;
use core::slice;
use core::usize;
use spin::Mutex;
use phys_mem;
use prelude::*;
use ptr::{self,Align};
use syscall::{ErrNum,Result};

struct Block<T> {
    ptr: *mut u8,
    len: usize,
    tag: Option<T>
}

struct VirtualState<T> {
    blocks: Vec<Block<T>>
}

impl<T> VirtualState<T> {
    pub fn alloc(&mut self, len: usize, tag: T) -> Result<&'static mut [u8]> {
        let unaligned_len = len;
        let len = Align::up(len, phys_mem::PAGE_SIZE);

        let pos =
            match self.blocks.iter().position(|block| block.tag.is_none() && block.len >= len) {
                Some(pos) => pos,
                None => { return Err(ErrNum::OutOfMemory) }
            };

        let (orig_len, orig_ptr) = {
            let block1 = &mut self.blocks[pos];
            let orig_len = mem::replace(&mut block1.len, len);
            block1.tag = Some(tag);
            (orig_len, block1.ptr)
        };

        let block2 = Block {
            ptr: unsafe { orig_ptr.offset(len as isize) },
            len: orig_len - len,
            tag: None
        };

        self.blocks.insert(pos + 1, block2);
        Ok(unsafe { slice::from_raw_parts_mut(orig_ptr, unaligned_len) })
    }

    pub fn reserve(&mut self, slice: &mut [u8], tag: T) -> bool {
        let (ptr, len) = (slice.as_mut_ptr(), slice.len());
        let (ptr, len) = Align::range(ptr, len, phys_mem::PAGE_SIZE);

        let pos =
            match self.blocks.iter().position(|block| block.tag.is_none() && block.ptr <= ptr && block.len >= len) {
                Some(pos) => pos,
                None => { return false }
            };

        let (orig_len, len0) = {
            let block0 = &mut self.blocks[pos];
            let len0 = ptr::bytes_between(block0.ptr, ptr);
            let orig_len = mem::replace(&mut block0.len, len0);
            (orig_len, len0)
        };

        let block1 = Block {
            ptr: ptr,
            len: len,
            tag: Some(tag)
        };

        let block2 = Block {
            ptr: unsafe { ptr.offset(len as isize) },
            len: orig_len - len - len0,
            tag: None
        };

        self.blocks.insert(pos + 1, block1);
        self.blocks.insert(pos + 2, block2);
        true
    }

    fn find_block_position(&self, ptr: *mut u8) -> Option<usize> {
        self.blocks.iter().position(|block| block.ptr <= ptr && ptr < unsafe { block.ptr.offset(block.len as isize) })
    }

    pub fn free(&mut self, ptr: *mut u8) -> bool {
        let pos =
            match self.find_block_position(ptr) {
                Some(pos) => pos,
                None => { return false }
            };

        self.blocks[pos].tag = None;

        if pos < self.blocks.len() - 1 && self.blocks[pos + 1].tag.is_none() {
            let block2 = self.blocks.remove(pos + 1);
            let block1 = &mut self.blocks[pos];
            block1.len += block2.len;
        }

        true
    }

    pub fn tag_at(&self, ptr: *mut u8) -> Option<(&'static mut [u8], &T)> {
        match self.find_block_position(ptr) {
            Some(pos) => {
                let block = &self.blocks[pos];
                let slice = unsafe { slice::from_raw_parts_mut(block.ptr, block.len) };
                block.tag.as_ref().map(|tag| (slice, tag))
            },

            None => None
        }
    }
}

pub struct VirtualTree<T> {
    state: Mutex<VirtualState<T>>
}

impl<T> VirtualTree<T> {
    pub fn new() -> Self {
        VirtualTree {
            state: Mutex::new(VirtualState {
                blocks: vec![Block {
                    ptr: 0 as *mut u8,
                    len: usize::MAX,
                    tag: None
                }]
            })
        }
    }

    pub fn block_count(&self) -> usize {
        lock!(self.state).blocks.len()
    }

    pub fn alloc(&self, len: usize, tag: T) -> Result<&mut [u8]> {
        let slice: &'static mut [u8] = lock!(self.state).alloc(len, tag)?;
        let slice: &mut [u8] = unsafe { mem::transmute(slice) };
        Ok(slice)
    }

    pub fn reserve(&self, slice: &mut [u8], tag: T) -> bool {
        lock!(self.state).reserve(slice, tag)
    }

    pub fn free(&self, p: *mut u8) -> bool {
        lock!(self.state).free(p)
    }
}

impl<T: Clone> VirtualTree<T> {
    pub fn tag_at(&self, p: *mut u8) -> Option<(&'static mut [u8], T)> {
        lock!(self.state).tag_at(p).map(|(slice, tag)| {
            let slice: &mut [u8] = unsafe { mem::transmute(slice) };
            let tag = (*tag).clone();
            (slice, tag)
        })
    }
}

#[cfg(feature = "test")]
pub mod test {
    use core::slice;
    use super::*;

    test! {
       fn can_alloc_free() {
           let tree = VirtualTree::new();
           assert_eq!(1, tree.block_count());

           let slice = tree.alloc(4096, ()).unwrap();
           assert_eq!(0 as *const u8, slice.as_ptr());
           assert_eq!(2, tree.block_count());

           //tree.tag_at(slice.as_mut_ptr()).unwrap();

           assert!(tree.free(slice.as_mut_ptr()));
           assert_eq!(1, tree.block_count());
       }

       fn can_reserve_alloc_free() {
           let tree = VirtualTree::new();
           assert_eq!(1, tree.block_count());

           assert!(tree.reserve(unsafe { slice::from_raw_parts_mut(0 as *mut u8, 4096) }, ()));
           assert_eq!(3, tree.block_count());

           //tree.tag_at(0 as *mut u8).unwrap();

           let slice = tree.alloc(4096, ()).unwrap();
           assert_eq!(4, tree.block_count());
           assert_eq!(4096 as *const u8, slice.as_ptr());

           assert!(tree.free(slice.as_mut_ptr()));
           assert_eq!(3, tree.block_count());
       }
    }
}
