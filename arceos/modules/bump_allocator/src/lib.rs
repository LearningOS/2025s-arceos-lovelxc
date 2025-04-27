#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;
use memory_addr::align_up;
/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
// const PAGE_SIZE: usize = 0x1000;
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    byte_pos: usize,
    page_pos: usize,
    count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    /// Create a new early allocator with given memory range [start, end)
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            byte_pos: 0,
            page_pos: 0,
            count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.byte_pos = start;
        self.page_pos = start + size;
        self.count = 0;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        unimplemented!()
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let start = align_up(self.byte_pos, layout.align());
        let next = start + layout.size();
        if next > self.page_pos {
            Err(AllocError::NoMemory)
        } else {
            self.byte_pos = next;
            self.count += 1;
            NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            // Free all bytes allocations
            self.byte_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.byte_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_pos - self.byte_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_pages(&mut self, num_pages: usize, _align_pow2: usize) -> AllocResult<usize> {
        let next = self.page_pos - num_pages * PAGE_SIZE;
        if next < self.byte_pos {
            Err(AllocError::NoMemory)
        } else {
            self.page_pos = next;
            Ok(self.page_pos)
        }
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Pages are never freed in early allocator
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.page_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.page_pos - self.byte_pos) / PAGE_SIZE
    }
}
