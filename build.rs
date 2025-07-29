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

// ############ MTOOLS ############

fn create_empty_fat32_image(path: &str, total_sectors: u32) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::{Write, Seek, SeekFrom};

    let bytes_per_sector = 512u16;
    let sectors_per_cluster = 8u8;
    let reserved_sectors = 32u16;
    let num_fats = 2u8;
    let sectors_per_fat = 100u32; // valeur arbitraire correcte pour 512MB
    let root_dir_cluster = 2u32;
    let volume_label = b"NO NAME    ";

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;

    // === 1. Boot Sector (LBA 0) ===
    let mut sector = [0u8; 512];
    sector[0] = 0xEB;
    sector[1] = 0x58;
    sector[2] = 0x90;
    sector[3..11].copy_from_slice(b"MSDOS5.0");

    sector[11..13].copy_from_slice(&bytes_per_sector.to_le_bytes());
    sector[13] = sectors_per_cluster;
    sector[14..16].copy_from_slice(&reserved_sectors.to_le_bytes());
    sector[16] = num_fats;
    sector[17..19].copy_from_slice(&0u16.to_le_bytes()); // root entries (FAT32 = 0)
    sector[19..21].copy_from_slice(&0u16.to_le_bytes()); // total sectors 16-bit
    sector[21] = 0xF8; // media descriptor
    sector[22..24].copy_from_slice(&0u16.to_le_bytes()); // sectors per FAT (FAT12/16)

    sector[32..36].copy_from_slice(&sectors_per_fat.to_le_bytes());
    sector[36..38].copy_from_slice(&0x0000u16.to_le_bytes()); // flags
    sector[40..42].copy_from_slice(&0x0000u16.to_le_bytes()); // FS version
    sector[44..48].copy_from_slice(&root_dir_cluster.to_le_bytes());

    sector[48..50].copy_from_slice(&1u16.to_le_bytes()); // FSInfo
    sector[50..52].copy_from_slice(&6u16.to_le_bytes()); // Backup boot sector

    sector[64..72].copy_from_slice(volume_label); // "NO NAME    "
    sector[72..80].copy_from_slice(b"FAT32   ");
    sector[510] = 0x55;
    sector[511] = 0xAA;

    file.seek(SeekFrom::Start(0))?;
    file.write_all(&sector)?;

    // === 2. FSInfo (LBA 1) ===
    let mut fsinfo = [0u8; 512];
    fsinfo[0..4].copy_from_slice(b"RRaA");
    fsinfo[484..488].copy_from_slice(b"rrAa");
    fsinfo[488..492].copy_from_slice(&(0xFFFFFFFFu32).to_le_bytes()); // free clusters
    fsinfo[492..496].copy_from_slice(&(0xFFFFFFFFu32).to_le_bytes()); // next free
    fsinfo[510] = 0x55;
    fsinfo[511] = 0xAA;

    file.seek(SeekFrom::Start(1 * 512))?;
    file.write_all(&fsinfo)?;

    // === 3. FAT tables ===
    let fat1_start = reserved_sectors as u64 * 512;
    let fat_size_bytes = sectors_per_fat as usize * 512;

    let mut fat = vec![0u8; fat_size_bytes];
    fat[0] = 0xF8;
    fat[1] = 0xFF;
    fat[2] = 0xFF;
    fat[3] = 0x0F;

    file.seek(SeekFrom::Start(fat1_start))?;
    file.write_all(&fat)?;
    file.write_all(&fat)?;

    Ok(())
}

fn create_directory(path: &str, cluster_lba: u64, name: &str) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};

    let name = format!("{:<11}", name.to_uppercase().replace("/", "")).into_bytes(); // 8.3 format
    let mut entry = [0u8; 32];
    entry[0..11].copy_from_slice(&name[0..11.min(name.len())]);
    entry[11] = 0x10; // ATTR_DIRECTORY
    entry[26..28].copy_from_slice(&2u16.to_le_bytes()); // cluster low
    entry[20..22].copy_from_slice(&0u16.to_le_bytes()); // cluster high

    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    file.seek(SeekFrom::Start(cluster_lba * 512))?;
    file.write_all(&entry)?;

    Ok(())
}

fn copy_file_to_fat(path: &str, cluster_lba: u64, fat_filename: &str, source_file: &str) -> std::io::Result<()> {
    use std::fs::{OpenOptions, File};
    use std::io::{Seek, SeekFrom, Read, Write};

    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut src = File::open(source_file)?;
    let mut data = Vec::new();
    src.read_to_end(&mut data)?;

    // Write FAT directory entry
    let name = format!("{:<11}", fat_filename.to_uppercase().replace("/", "")).into_bytes();
    let mut entry = [0u8; 32];
    entry[0..11].copy_from_slice(&name[0..11.min(name.len())]);
    entry[11] = 0x20; // ATTR_ARCHIVE
    entry[28..32].copy_from_slice(&(data.len() as u32).to_le_bytes());
    entry[26..28].copy_from_slice(&2u16.to_le_bytes()); // cluster low
    entry[20..22].copy_from_slice(&0u16.to_le_bytes()); // cluster high

    file.seek(SeekFrom::Start(cluster_lba * 512))?;
    file.write_all(&entry)?;

    // Write actual file content to cluster area (ex: LBA 2*sectors_per_cluster + data region)
    let data_region_start_lba = 32 + 2 * 100; // reserved + 2 FATs
    let first_data_sector = data_region_start_lba + (2 - 2) * 8; // cluster 2
    file.seek(SeekFrom::Start(first_data_sector * 512))?;
    file.write_all(&data)?;

    Ok(())
}
