use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

pub struct BootInfoFrameAllocator<T> {
    usable_frames: T,
}

fn usable_frames(memory_map: &'static MemoryRegions) -> impl Iterator<Item = PhysFrame> {
    // get usable regions from memory map
    let regions = memory_map.iter();
    let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
    // map each region to its address range
    let addr_ranges = usable_regions.map(|r| r.start..r.end);
    // transform to an iterator of frame start addresses
    let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
    // create `PhysFrame` types from the start addresses
    frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
}

impl BootInfoFrameAllocator<()> {
    pub unsafe fn new(
        memory_map: &'static MemoryRegions,
    ) -> BootInfoFrameAllocator<impl Iterator<Item = PhysFrame>> {
        BootInfoFrameAllocator {
            usable_frames: usable_frames(memory_map),
        }
    }
}

#[deny(unsafe_op_in_unsafe_fn)]
unsafe impl<I: Iterator<Item = PhysFrame>> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<I> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.usable_frames.next()
    }
}

#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

#[deny(unsafe_op_in_unsafe_fn)]
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}
