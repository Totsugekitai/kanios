use crate::{
    println,
    utils::{align_up, oct2int},
    virtio_blk::{self, read_write_disk, SECTOR_SIZE},
};
use core::{mem, slice};

const FILES_MAX: usize = 3;
const FILE_DATA_MAX: usize = 64 * 1024; // 64KB
const DISK_MAX_SIZE: usize = align_up(
    (mem::size_of::<File>() * FILES_MAX) as u64,
    SECTOR_SIZE as u64,
) as usize;

#[repr(C, packed)]
#[derive(Debug)]
struct TarHeader {
    pub name: [u8; 100],
    pub mode: [u8; 8],
    pub uid: [u8; 8],
    pub gid: [u8; 8],
    pub size: [u8; 12],
    pub mtime: [u8; 12],
    pub checksum: [u8; 8],
    pub type_: u8,
    pub linkname: [u8; 100],
    pub magic: [u8; 6],
    pub version: [u8; 2],
    pub uname: [u8; 32],
    pub gname: [u8; 32],
    pub devmajor: [u8; 8],
    pub devminor: [u8; 8],
    pub prefix: [u8; 155],
    padding: [u8; 12],
    pub data: [u8; 0],
}

#[derive(Debug, Clone, Copy)]
pub struct File {
    pub in_use: bool,              // このファイルエントリが使われているか
    pub name: [u8; 100],           // ファイル名
    pub data: [u8; FILE_DATA_MAX], // ファイルの内容
    pub size: usize,               // ファイルサイズ
}

impl File {
    const fn new() -> Self {
        Self {
            in_use: false,
            name: [0; 100],
            data: [0; FILE_DATA_MAX],
            size: 0,
        }
    }
}

static mut FILES: [File; FILES_MAX] = [File::new(); FILES_MAX];
static mut DISK: [u8; DISK_MAX_SIZE] = [0; DISK_MAX_SIZE];

pub unsafe fn init() {
    let mut sector = 0;
    while sector < mem::size_of_val(&DISK) / SECTOR_SIZE as usize {
        virtio_blk::read_write_disk(
            &mut DISK[sector * SECTOR_SIZE as usize],
            sector as u32,
            false,
        );
        sector += 1;
    }

    let mut off = 0;
    for i in 0..FILES_MAX {
        let header = (&mut DISK[off] as *mut u8 as *mut TarHeader)
            .as_mut()
            .unwrap();

        let name = header.name.as_ascii().unwrap().as_str();
        if name.is_empty() {
            break;
        }

        let magic_ascii = header.magic.as_ascii().unwrap();
        let magic = &magic_ascii.as_str()[0..(magic_ascii.len() - 1)];
        if magic != "ustar" {
            panic!("invalid tar header: magic=\"{magic}\"");
        }

        let filesz = oct2int(
            &header.size as *const [u8] as *const u8,
            mem::size_of_val(&header.size),
        ) as usize;

        let file = &mut FILES[i];
        file.in_use = true;
        file.name[0..name.len()].copy_from_slice(name.as_bytes());
        if filesz > 0 {
            file.data[0..filesz].copy_from_slice(slice::from_raw_parts(
                &header.data as *const [u8] as *const u8,
                filesz,
            ));
        }
        file.size = filesz;
        println!(
            "file: {}, size={}",
            file.name.as_ascii().unwrap().as_str(),
            file.size,
        );

        off += align_up(
            (mem::size_of::<TarHeader>() + filesz) as u64,
            SECTOR_SIZE as u64,
        ) as usize;
    }
}

pub unsafe fn flush() {
    let mut off = 0;
    for file_i in 0..FILES_MAX {
        let file = &mut FILES[file_i];
        if !file.in_use {
            continue;
        }

        let header = (&mut DISK[off] as *mut u8 as *mut TarHeader)
            .as_mut()
            .unwrap();
        let name = &file.name;
        header.name[0..file.name.len()].copy_from_slice(name);
        let mode = b"0000644\0";
        header.mode[0..mode.len()].copy_from_slice(mode);
        let magic = b"ustar\0";
        header.magic[0..magic.len()].copy_from_slice(magic);
        let version = b"00";
        header.version[0..version.len()].copy_from_slice(version);
        header.type_ = b'0';

        // ファイルサイズを8進数文字列に変換
        let mut filesz = file.size;
        for i in 0..(header.size.len() - 1) {
            header.size[(header.size.len() - 2) - i] = (filesz % 8) as u8 + b'0';
            filesz /= 8;
        }
        header.size[header.size.len() - 1] = b'\0';

        // チェックサムを計算
        let mut checksum = b' ' as usize * mem::size_of_val(&header.checksum);
        checksum += header.name.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.mode.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.uid.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.gid.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.size.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.mtime.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.type_ as usize;
        checksum += header.linkname.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.magic.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.version.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.uname.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.gname.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.devmajor.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.devminor.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.prefix.iter().fold(0, |sum, i| sum + *i as usize);

        for i in 0..6 {
            header.checksum[(header.checksum.len() - 3) - i] = (checksum % 8) as u8 + b'0';
            checksum /= 8;
        }

        // ファイルデータをコピー
        let header_data =
            slice::from_raw_parts_mut(&mut header.data as *mut [u8] as *mut u8, file.size);
        if file.size > 0 {
            header_data.copy_from_slice(&file.data[0..file.size]);
        }

        off += align_up(
            (mem::size_of::<TarHeader>() + file.size) as u64,
            SECTOR_SIZE as u64,
        ) as usize;
    }

    // DISK変数の内容をディスクに書き込む
    for sector in 0..(mem::size_of_val(&DISK) / SECTOR_SIZE as usize) {
        read_write_disk(
            &mut DISK[sector * SECTOR_SIZE as usize] as &mut u8,
            sector as u32,
            true,
        );
    }

    println!("wrote {} bytes to disk", mem::size_of_val(&DISK));
}

pub fn lookup(filename: &str) -> Result<*mut File, ()> {
    for i in 0..FILES_MAX {
        let file = unsafe { &FILES[i] };
        let name = &file.name.as_ascii().unwrap().as_str()[0..filename.len()];
        if name == filename {
            return Ok(file as *const File as *mut File);
        }
    }
    Err(())
}
