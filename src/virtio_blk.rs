use crate::{
    memory::{alloc_pages, PAGE_SIZE},
    println,
    types::PhysAddr,
    utils::align_up,
};
use core::{
    mem, ptr,
    sync::atomic::{compiler_fence, Ordering::SeqCst},
};

pub const SECTOR_SIZE: u32 = 512;
const VIRTQ_ENTRY_NUM: usize = 16;
const VIRTIO_DEVICE_BLK: u32 = 2;
pub const VIRTIO_BLK_PADDR: PhysAddr = PhysAddr(0x1000_1000);
const VIRTIO_REG_MAGIC: u64 = 0x00;
const VIRTIO_REG_VERSION: u64 = 0x04;
const VIRTIO_REG_DEVICE_ID: u64 = 0x08;
const VIRTIO_REG_QUEUE_SEL: u64 = 0x30;
// const VIRTIO_REG_QUEUE_NUM_MAX: u64 = 0x34;
const VIRTIO_REG_QUEUE_NUM: u64 = 0x38;
const VIRTIO_REG_QUEUE_ALIGN: u64 = 0x3c;
const VIRTIO_REG_QUEUE_PFN: u64 = 0x40;
// const VIRTIO_REG_QUEUE_READY: u64 = 0x44;
const VIRTIO_REG_QUEUE_NOTIFY: u64 = 0x50;
const VIRTIO_REG_DEVICE_STATUS: u64 = 0x70;
const VIRTIO_REG_DEVICE_CONFIG: u64 = 0x100;
const VIRTIO_STATUS_ACK: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
const VIRTIO_STATUS_FEAT_OK: u32 = 8;
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;
// const VIRTQ_AVAIL_F_NO_INTERRUPT: u64 = 1;
const VIRTIO_BLK_T_IN: u64 = 0;
const VIRTIO_BLK_T_OUT: u64 = 1;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; VIRTQ_ENTRY_NUM],
    used_event: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; VIRTQ_ENTRY_NUM],
    avail_event: u16,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct Virtq {
    desc: [VirtqDesc; VIRTQ_ENTRY_NUM], // size(0x100), align(0x1000)
    avail: VirtqAvail,                  // size(0x26)
    pad: [u8; 0xeda],                   // size(0x1000 - (0x100 + 0x26) = 0xeda)
    used: VirtqUsed,                    // size(_), align(0x1000)

    queue_idx: u32,
    used_idx: *mut u16,
    last_used_idx: u16,
}

impl Virtq {
    unsafe fn init(index: u32) -> *mut Self {
        let virtq_paddr =
            alloc_pages(align_up(mem::size_of::<Virtq>() as u64, PAGE_SIZE) / PAGE_SIZE);
        let vq = virtq_paddr.to_u64() as *mut Virtq;
        let virtq = vq.as_mut().unwrap();
        virtq.queue_idx = index;
        let used_idx = (&mut (virtq.used) as *const VirtqUsed as *const u8)
            .offset(mem::offset_of!(VirtqUsed, idx) as isize);
        virtq.used_idx = used_idx as *mut u16;

        // 1. Select the queue writing its index (first queue is 0) to QueueSel.
        reg_write32(VIRTIO_REG_QUEUE_SEL, index);
        // 5. Notify the device about the queue size by writing the size to QueueNum.
        reg_write32(VIRTIO_REG_QUEUE_NUM, VIRTQ_ENTRY_NUM as u32);
        // 6. Notify the device about the used alignment by writing its value in bytes to QueueAlign.
        reg_write32(VIRTIO_REG_QUEUE_ALIGN, 0);
        // 7. Write the physical number of the first page of the queue to the QueuePFN register.
        reg_write32(VIRTIO_REG_QUEUE_PFN, virtq_paddr.to_u64() as u32);

        vq
    }

    fn kick(vq: *mut Virtq, desc_index: u32) {
        let vq = unsafe { vq.as_mut().unwrap() };

        vq.avail.ring[vq.avail.idx as usize % VIRTQ_ENTRY_NUM] = desc_index as u16;
        vq.avail.idx += 1;
        compiler_fence(SeqCst);
        unsafe {
            reg_write32(VIRTIO_REG_QUEUE_NOTIFY, vq.queue_idx);
        }
        vq.last_used_idx += 1;
    }

    fn is_busy(&self) -> bool {
        // self.last_used_idx != unsafe { *self.used_idx }
        self.last_used_idx != self.used.idx
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct VirtioBlkReq {
    type_: u32,
    reserved: u32,
    sector: u64,
    data: [u8; 512],
    status: u8,
}

// unsafe fn reg_read8(offset: u64) -> u8 {
//     ((VIRTIO_BLK_PADDR + offset) as *const u8).read_volatile()
// }

unsafe fn reg_read32(offset: u64) -> u32 {
    ((VIRTIO_BLK_PADDR.to_u64() + offset) as *const u32).read_volatile()
}

unsafe fn reg_read64(offset: u64) -> u64 {
    ((VIRTIO_BLK_PADDR.to_u64() + offset) as *const u64).read_volatile()
}

unsafe fn reg_write32(offset: u64, value: u32) {
    ((VIRTIO_BLK_PADDR.to_u64() + offset) as *mut u32).write_volatile(value);
}

unsafe fn reg_fetch_and_or32(offset: u64, value: u32) {
    reg_write32(offset, reg_read32(offset) | value);
}

static mut BLK_REQUEST_VQ: *mut Virtq = ptr::null_mut();
static mut BLK_REQ: *mut VirtioBlkReq = ptr::null_mut();
static mut BLK_REQ_PADDR: PhysAddr = PhysAddr(0);
static mut BLK_CAPACITY: u32 = 0;

pub unsafe fn init() {
    assert!(reg_read32(VIRTIO_REG_MAGIC) == 0x74726976);
    assert!(reg_read32(VIRTIO_REG_VERSION) == 1);
    assert!(reg_read32(VIRTIO_REG_DEVICE_ID) == VIRTIO_DEVICE_BLK);

    // 1. Reset the device.
    reg_write32(VIRTIO_REG_DEVICE_STATUS, 0);
    // 2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the device.
    reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_ACK);
    // 3. Set the DRIVER status bit: the guest OS knows how to drive the device.
    reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER);
    // 5. Set the FEATURES_OK status bit.
    reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_FEAT_OK);
    // 7. Perform device-specific setup, including discovery of virtqueues for the device
    BLK_REQUEST_VQ = Virtq::init(0);
    // 8. Set the DRIVER_OK status bit.
    reg_write32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER_OK);

    // ディスク容量を取得
    BLK_CAPACITY = reg_read64(VIRTIO_REG_DEVICE_CONFIG + 0) as u32 * SECTOR_SIZE;
    println!("virtio-blk: capacity is {BLK_CAPACITY} bytes");

    // デバイスへの処理要求を格納する領域を確保
    BLK_REQ_PADDR =
        alloc_pages(align_up(mem::size_of::<VirtioBlkReq>() as u64, PAGE_SIZE) / PAGE_SIZE);
    BLK_REQ = BLK_REQ_PADDR.to_u64() as *mut VirtioBlkReq;
}

pub unsafe fn read_write_disk(buf: *mut u8, sector: u32, is_write: bool) -> Result<(), ()> {
    if sector >= BLK_CAPACITY / SECTOR_SIZE {
        println!(
            "virtio: tried to read/write sector={sector}, but capacity is {}",
            BLK_CAPACITY / SECTOR_SIZE
        );
        return Err(());
    }

    // リクエストを構築する
    let blk_req = BLK_REQ.as_mut().unwrap();
    blk_req.sector = sector as u64;
    blk_req.type_ = if is_write {
        VIRTIO_BLK_T_OUT as u32
    } else {
        VIRTIO_BLK_T_IN as u32
    };
    if is_write {
        ptr::copy(
            buf,
            &mut blk_req.data as *mut [u8] as *mut u8,
            SECTOR_SIZE as usize,
        );
    }

    // virtqueueのディスクリプタを構築する
    let vq = BLK_REQUEST_VQ.as_mut().unwrap();
    vq.desc[0].addr = BLK_REQ_PADDR.to_u64();
    vq.desc[0].len = (mem::size_of::<u32>() * 2 + mem::size_of::<u64>()) as u32;
    vq.desc[0].flags = VIRTQ_DESC_F_NEXT;
    vq.desc[0].next = 1;

    vq.desc[1].addr = BLK_REQ_PADDR.to_u64() + mem::offset_of!(VirtioBlkReq, data) as u64;
    vq.desc[1].len = SECTOR_SIZE;
    vq.desc[1].flags = VIRTQ_DESC_F_NEXT | if is_write { 0 } else { VIRTQ_DESC_F_WRITE };
    vq.desc[1].next = 2;

    vq.desc[2].addr = BLK_REQ_PADDR.to_u64() + mem::offset_of!(VirtioBlkReq, status) as u64;
    vq.desc[2].len = mem::size_of::<u8>() as u32;
    vq.desc[2].flags = VIRTQ_DESC_F_WRITE;

    // デバイスに新しいリクエストがあることを通知する
    Virtq::kick(vq, 0);

    while vq.is_busy() {}

    // 0でない値が帰ってきたらエラー
    if blk_req.status != 0 {
        println!(
            "virtio: warn: failed to read/write sector={sector}, status={}",
            blk_req.status
        );
        return Err(());
    }

    // 読み込み処理の場合は、バッファにデータをコピーする
    if !is_write {
        ptr::copy(
            &blk_req.data as *const [u8] as *const u8,
            buf,
            SECTOR_SIZE as usize,
        );
    }

    Ok(())
}
