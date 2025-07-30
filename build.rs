//! WARNING: the GPT implementation is specific to UEFI boot and works only with the given size
//! (512MiB). Do not use it for any purpose.
//!
//! TODO: generalize the GPT implementation
//! TODO: replace the mtools commands with pure Rust

use std::env::var;
use std::fs::{self, create_dir_all};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = "target/boot/x86_64/uefi.img";
    let tmp_path = "target/boot/x86_64/.tmp.img";
    let bootloader_path =
        var("CARGO_BIN_FILE_BOOTLOADER_X86_64_UEFI_bootloader-x86_64-uefi").unwrap();

    let _ = create_dir_all("target/boot/x86_64");
    let _ = fs::remove_file(out_path);
    let _ = fs::remove_file(tmp_path);

    write_zeros(out_path, 512 * 1024 * 1024)?;

    write_zeros(tmp_path, 843_709 * 512)?;

    create_gpt_disk(out_path)?;

    run(Command::new("mformat").args(["-i", tmp_path, "-F"]))?;

    run(Command::new("mmd").args(["-i", tmp_path, "::/EFI"]))?;

    run(Command::new("mmd").args(["-i", tmp_path, "::/EFI/BOOT"]))?;

    run(Command::new("mcopy").args([
        "-i",
        tmp_path,
        "-s",
        &bootloader_path,
        "::/EFI/BOOT/BOOTX64.EFI",
    ]))?;

    copy_with_seek(tmp_path, out_path, 2048 * 512)?;

    let _ = fs::remove_file(tmp_path);

    println!("✔️ Image EFI créée avec succès : {}", out_path);
    Ok(())
}

fn run(cmd: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
    println!("→ Exécution: {:?}", cmd);
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        Err(format!("La commande a échoué: {:?}", cmd).into())
    } else {
        Ok(())
    }
}

// ########### DD ###########

fn write_zeros(path: &str, size_in_bytes: u64) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.seek(SeekFrom::Start(size_in_bytes - 1))?;
    file.write_all(&[0])?;
    Ok(())
}

fn copy_with_seek(src_path: &str, dst_path: &str, offset_bytes: u64) -> io::Result<()> {
    let mut src = File::open(src_path)?;
    let mut dst = OpenOptions::new().write(true).read(true).open(dst_path)?;

    dst.seek(SeekFrom::Start(offset_bytes))?;

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = src.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dst.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}

// ############ FGDISK ############

const SECTOR_SIZE: usize = 512;
const NUM_PART_ENTRIES: u32 = 128;
const PARTITION_ENTRY_SIZE: u32 = 128;
const PARTITION_ARRAY_SIZE: usize = (NUM_PART_ENTRIES as usize) * (PARTITION_ENTRY_SIZE as usize);
const PRIMARY_HEADER_LBA: u64 = 1;
const PARTITION_ENTRIES_LBA: u64 = 2;
const FIRST_USABLE_LBA: u64 = 2048;
const LAST_USABLE_LBA: u64 = 206847;
const BACKUP_HEADER_LBA: u64 = 1048575;

const EFI_PART_TYPE_GUID: [u8; 16] = [
    0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B,
];

fn crc32(data: &[u8]) -> u32 {
    const POLY: u32 = 0xEDB88320;
    let mut table = [0u32; 256];
    for i in 0..256 {
        let mut crc = i as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                POLY ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
        table[i] = crc;
    }

    let mut crc = 0xFFFF_FFFF;
    for &byte in data {
        let idx = ((crc ^ (byte as u32)) & 0xFF) as usize;
        crc = table[idx] ^ (crc >> 8);
    }

    !crc
}

fn create_gpt_disk(path: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;

    let mut mbr = [0u8; SECTOR_SIZE];
    mbr[0x1BE + 4] = 0xEE;
    mbr[0x1BE + 8..0x1BE + 12].copy_from_slice(&1u32.to_le_bytes());
    mbr[0x1BE + 12..0x1BE + 16].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    mbr[510] = 0x55;
    mbr[511] = 0xAA;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&mbr)?;

    let mut part_entry = [0u8; PARTITION_ENTRY_SIZE as usize];
    part_entry[0..16].copy_from_slice(&EFI_PART_TYPE_GUID);
    let fake_guid = [0xAA; 16];
    part_entry[16..32].copy_from_slice(&fake_guid);
    part_entry[32..40].copy_from_slice(&FIRST_USABLE_LBA.to_le_bytes());
    part_entry[40..48].copy_from_slice(&LAST_USABLE_LBA.to_le_bytes());
    let label_utf16: Vec<u8> = "EFI System Partition"
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    part_entry[56..56 + label_utf16.len()].copy_from_slice(&label_utf16);

    let mut partition_entries = vec![0u8; PARTITION_ARRAY_SIZE];
    partition_entries[..128].copy_from_slice(&part_entry);
    let partition_array_crc = crc32(&partition_entries);

    let mut gpt_header = [0u8; SECTOR_SIZE];
    gpt_header[0..8].copy_from_slice(b"EFI PART");
    gpt_header[8..12].copy_from_slice(&[0x00, 0x00, 0x01, 0x00]);
    gpt_header[12..16].copy_from_slice(&(92u32).to_le_bytes());
    gpt_header[24..32].copy_from_slice(&PRIMARY_HEADER_LBA.to_le_bytes());
    gpt_header[32..40].copy_from_slice(&BACKUP_HEADER_LBA.to_le_bytes());
    gpt_header[40..48].copy_from_slice(&FIRST_USABLE_LBA.to_le_bytes());
    gpt_header[48..56].copy_from_slice(&LAST_USABLE_LBA.to_le_bytes());
    gpt_header[56..72].copy_from_slice(&fake_guid);
    gpt_header[72..80].copy_from_slice(&PARTITION_ENTRIES_LBA.to_le_bytes());
    gpt_header[80..84].copy_from_slice(&NUM_PART_ENTRIES.to_le_bytes());
    gpt_header[84..88].copy_from_slice(&PARTITION_ENTRY_SIZE.to_le_bytes());
    gpt_header[88..92].copy_from_slice(&partition_array_crc.to_le_bytes());

    let mut header_crc = gpt_header.clone();
    header_crc[16..20].fill(0);
    let gpt_header_crc = crc32(&header_crc[..92]);
    gpt_header[16..20].copy_from_slice(&gpt_header_crc.to_le_bytes());

    file.seek(SeekFrom::Start(PARTITION_ENTRIES_LBA * SECTOR_SIZE as u64))?;
    file.write_all(&partition_entries)?;

    file.seek(SeekFrom::Start(PRIMARY_HEADER_LBA * SECTOR_SIZE as u64))?;
    file.write_all(&gpt_header)?;

    let backup_part_entries_lba =
        BACKUP_HEADER_LBA - ((PARTITION_ARRAY_SIZE as u64) / SECTOR_SIZE as u64);

    file.seek(SeekFrom::Start(
        backup_part_entries_lba * SECTOR_SIZE as u64,
    ))?;
    file.write_all(&partition_entries)?;

    let mut backup_header = gpt_header.clone();
    backup_header[24..32].copy_from_slice(&BACKUP_HEADER_LBA.to_le_bytes());
    backup_header[32..40].copy_from_slice(&PRIMARY_HEADER_LBA.to_le_bytes());
    backup_header[72..80].copy_from_slice(&backup_part_entries_lba.to_le_bytes());

    let mut crc_buf = backup_header.clone();
    crc_buf[16..20].fill(0);
    let backup_crc = crc32(&crc_buf[..92]);
    backup_header[16..20].copy_from_slice(&backup_crc.to_le_bytes());

    file.seek(SeekFrom::Start(BACKUP_HEADER_LBA * SECTOR_SIZE as u64))?;
    file.write_all(&backup_header)?;

    Ok(())
}
