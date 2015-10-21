use core::mem;
use core::slice;
use core::usize;
use mutex::Mutex;
use phys_mem;
use prelude::*;
use ptr::{self,Align};
use syscall::{ErrNum,Result};

extern {
    static kernel_end: u8;
}

struct Block {
    ptr: *mut u8,
    len: usize,
    free: bool
}

struct VirtualState {
    blocks: Vec<Block>
}

impl VirtualState {
    pub fn alloc(&mut self, len: usize) -> Result<&'static mut [u8]> {
        let unaligned_len = len;
        let len = Align::up(len, phys_mem::PAGE_SIZE);

        let pos =
            match self.blocks.iter().position(|block| block.free && block.len >= len) {
                Some(pos) => pos,
                None => { return Err(ErrNum::OutOfMemory) }
            };

        let (orig_len, orig_ptr) = {
            let block1 = &mut self.blocks[pos];
            let orig_len = mem::replace(&mut block1.len, len);
            block1.free = false;
            (orig_len, block1.ptr)
        };

        let block2 = Block {
            ptr: unsafe { orig_ptr.offset(len as isize) },
            len: orig_len - len,
            free: true
        };

        self.blocks.insert(pos + 1, block2);
        Ok(unsafe { slice::from_raw_parts_mut(orig_ptr, unaligned_len) })
    }

    pub fn reserve(&mut self, slice: &mut [u8]) -> bool {
        let (ptr, len) = (slice.as_mut_ptr(), slice.len());
        let (ptr, len) = Align::range(ptr, len, phys_mem::PAGE_SIZE);
        log!("reserve({:p}..{:p})", ptr, unsafe { ptr.offset(len as isize) });

        let pos =
            match self.blocks.iter().position(|block| block.free && block.ptr <= ptr && block.len >= len) {
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
            free: false
        };

        let block2 = Block {
            ptr: unsafe { ptr.offset(len as isize) },
            len: orig_len - len - len0,
            free: true
        };

        self.blocks.insert(pos + 1, block1);
        self.blocks.insert(pos + 2, block2);
        true
    }

    pub fn free(&mut self, ptr: *mut u8) -> bool {
        let pos =
            match self.blocks.iter().position(|block| block.ptr == ptr) {
                Some(pos) => pos,
                None => { return false }
            };

        self.blocks[pos].free = true;

        if pos < self.blocks.len() - 1 && self.blocks[pos + 1].free {
            let block2 = self.blocks.remove(pos + 1);
            let block1 = &mut self.blocks[pos];
            block1.len += block2.len;
        }

        true
    }
}

pub struct VirtualTree {
    state: Mutex<VirtualState>
}

impl VirtualTree {
    pub fn new() -> VirtualTree {
        VirtualTree {
            state: Mutex::new(VirtualState {
                blocks: vec![Block {
                    ptr: 0 as *mut u8,
                    len: usize::MAX,
                    free: true
                }]
            })
        }
    }

    pub fn for_kernel() -> VirtualTree {
        let tree = VirtualTree::new();
        let two_meg = 2 * 1024 * 1024;
        let kernel_end_ptr = Align::up(&kernel_end as *const u8, 4 * two_meg);
        let identity = unsafe { slice::from_raw_parts_mut(0 as *mut u8, kernel_end_ptr as usize) };
        tree.reserve(identity);
        tree
    }

    pub fn block_count(&self) -> usize {
        lock!(self.state).blocks.len()
    }

    pub fn alloc(&self, len: usize) -> Result<&mut [u8]> {
        let slice: &'static mut [u8] = try!(lock!(self.state).alloc(len));
        let slice: &mut [u8] = unsafe { mem::transmute(slice) };
        Ok(slice)
    }

    pub fn reserve(&self, slice: &mut [u8]) -> bool {
        lock!(self.state).reserve(slice)
    }

    pub fn free(&self, p: *mut u8) -> bool {
        lock!(self.state).free(p)
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

           let slice = tree.alloc(4096).unwrap();
           assert_eq!(0 as *const u8, slice.as_ptr());
           assert_eq!(2, tree.block_count());

           assert!(tree.free(slice.as_mut_ptr()));
           assert_eq!(1, tree.block_count());
       }

       fn can_reserve_alloc_free() {
           let tree = VirtualTree::new();
           assert_eq!(1, tree.block_count());

           assert!(tree.reserve(unsafe { slice::from_raw_parts_mut(0 as *mut u8, 4096) }));
           assert_eq!(3, tree.block_count());

           let slice = tree.alloc(4096).unwrap();
           assert_eq!(4, tree.block_count());
           assert_eq!(4096 as *const u8, slice.as_ptr());

           assert!(tree.free(slice.as_mut_ptr()));
           assert_eq!(3, tree.block_count());
       }
    }
}
