use std::{
    ffi::OsString,
    io::{Read, Seek, SeekFrom},
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
};

pub mod crypt_keys;

mod crypt;
mod util;

use crypt_keys::*;
use flate2::read::ZlibDecoder;
use log::debug;
use strum_macros::EnumIter;
use util::{read_u8, read_u32_le};

use crate::crypt::{decrypt_data, decrypt_header};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, EnumIter)]
#[non_exhaustive]
pub enum Game {
    ChaosHead,
    ChaosHeadTrailOne,
    ChaosHeadTrailTwo,
    MuramasaTrail,
    Muramasa,
    Sumaga,
    Django,
    DjangoTrial,
    Lamento,
    LamentoTrail,
    SweetPool,
    SumagaSpecial,
    Demonbane,
    MuramasaAD,
    Axanael,
    Kikokugai,
    SonicomiTrialTwo,
    SumagaThreePercent,
    Sonicomi,
    LostX,
    LostXTrailer,
    DramaticalMurder,
    Totono,
    DramaticalMurderReConnect,
    MuramasaSS,
}

impl Game {
    pub fn encryption_key(self) -> [u8; 256] {
        match self {
            Game::ChaosHead => CHAOS_HEAD,
            Game::ChaosHeadTrailOne => CHAOS_HEAD_TRAIL_1,
            Game::ChaosHeadTrailTwo => CHAOS_HEAD_TRAIL_2,
            Game::MuramasaTrail => MURAMASA_TRIAL,
            Game::Muramasa => MURAMASA,
            Game::Sumaga => SUMAGA,
            Game::Django => ZOKU_SATSURIKU_NO_DJANGO,
            Game::DjangoTrial => ZOKU_SATSURIKU_NO_DJANGO_TRIAL,
            Game::Lamento => LAMENTO_BEYOND_THE_VOID,
            Game::LamentoTrail => LAMENTO_BEYOND_THE_VOID_TRIAL,
            Game::SweetPool => SWEET_POOL,
            Game::SumagaSpecial => SUMAGA_SPECIAL,
            Game::Demonbane => DEMONBANE_THE_BEST,
            Game::MuramasaAD => MURAMASA_AD,
            Game::Axanael => AXANAEL_TRIAL,
            Game::Kikokugai => KIKOKUGAI_N2SYSTEM,
            Game::SonicomiTrialTwo => SONICOMI_TRIAL_2,
            Game::SumagaThreePercent => SUMAGA_3_PERCENT_TRIAL,
            Game::Sonicomi => SONICOMI,
            Game::LostX => GUILTY_CROWN_LOST_XMAS,
            Game::LostXTrailer => GUILTY_CROWN_LOST_XMAS_TRAILER,
            Game::DramaticalMurder => DRAMATICAL_MURDER,
            Game::Totono => TOTONO,
            Game::DramaticalMurderReConnect => DRAMATICAL_MURDER_RE_CONNECT,
            Game::MuramasaSS => MURAMASA_SS,
        }
    }
}

#[derive(Debug)]
pub struct NpaHead {
    pub head: [u8; 7],
    pub key_1: u32,
    pub key_2: u32,
    pub encrypted: bool,
    pub compressed: bool,
    pub file_count: u32,
    pub folder_count: u32,
    pub total_count: u32,
    pub start: u32,
}

#[derive(Debug, Clone, Default)]
pub struct NpaEntry {
    pub name_length: u32,
    pub type_: u8,
    pub file_id: u32,
    pub offset: u32,
    pub compressed_size: u32,
    pub original_size: u32,
    pub un_decoded_file_path: OsString,
    pub file_path: PathBuf,
}

impl NpaEntry {
    pub fn is_directory(&self) -> bool {
        self.type_ == 1
    }
}

pub fn parse_head<R: Read>(reader: &mut R) -> Result<NpaHead, std::io::Error> {
    let mut magic = [0u8; 7];
    reader.read_exact(&mut magic)?;

    let key_1 = util::read_u32_le(reader)?;
    let key_2 = util::read_u32_le(reader)?;
    let compressed = util::read_u8(reader)? == 1;
    let encrypted = util::read_u8(reader)? == 1;
    let total_count = util::read_u32_le(reader)?;
    let folder_count = util::read_u32_le(reader)?;
    let file_count = util::read_u32_le(reader)?;
    reader.read_exact(&mut [0u8; 8])?;

    let start = util::read_u32_le(reader)?;

    let header = NpaHead {
        head: magic,
        key_1,
        key_2,
        compressed,
        encrypted,
        total_count,
        folder_count,
        file_count,
        start,
    };

    debug!("Header: {:?}", header);

    Ok(header)
}

pub fn read_entries<R: Read>(
    reader: &mut R,
    header: &NpaHead,
    add_bytes_if_encrypted: bool,
) -> Result<Vec<NpaEntry>, std::io::Error> {
    let mut entries = Vec::with_capacity(header.total_count as usize);

    for i in 0..header.total_count as usize {
        let entry = read_entry(reader, i, header, add_bytes_if_encrypted)?;

        entries.push(entry);
    }

    Ok(entries)
}

pub fn read_entry<R: Read>(
    reader: &mut R,
    index: usize,
    header: &NpaHead,
    add_bytes_if_encrypted: bool,
) -> Result<NpaEntry, std::io::Error> {
    let nlength = read_u32_le(reader)? as usize;

    let mut file_name = vec![0u8; nlength];
    reader.read_exact(&mut file_name)?;

    for (x, byte) in file_name.iter_mut().enumerate() {
        *byte = byte.wrapping_add(decrypt_header(
            x as u32,
            index as u32,
            header,
            add_bytes_if_encrypted,
        ));
    }

    let decoded_path = util::decode_text(&file_name);

    if decoded_path.had_errors() {
        log::warn!("Failed to cleanly decode path: {}", decoded_path.text());
    }

    let file_path = decoded_path
        .text()
        .split('\\')
        .filter(|s| !s.is_empty())
        .fold(PathBuf::new(), |p, c| p.join(c));

    let type_ = read_u8(reader)?;
    let file_id = read_u32_le(reader)?;
    let offset = read_u32_le(reader)?;
    let compressed_size = read_u32_le(reader)?;
    let original_size = read_u32_le(reader)?;
    let un_decoded_file_path = OsString::from_vec(file_name);

    Ok(NpaEntry {
        name_length: nlength as u32,
        type_,
        file_id,
        offset,
        compressed_size,
        original_size,
        un_decoded_file_path,
        file_path,
    })
}

pub fn read_entry_data<R: Read + Seek>(
    reader: &mut R,
    header: &NpaHead,
    entry: &NpaEntry,
    game: Game,
) -> Result<Vec<u8>, std::io::Error> {
    log::debug!("Reading \"{}\"", entry.file_path.display());
    reader.seek(SeekFrom::Start((entry.offset + header.start + 0x29) as u64))?;

    let mut buffer = vec![0u8; entry.compressed_size as usize];
    reader.read_exact(&mut buffer)?;

    if header.encrypted {
        let key = decrypt_data(entry, header, game);
        let mut len = 0x1000;

        if game != Game::Lamento && game != Game::LamentoTrail {
            len += entry.un_decoded_file_path.len() as u32;
        }

        for x in 0..entry.compressed_size.min(len) {
            buffer[x as usize] = match game {
                Game::Lamento | Game::LamentoTrail => {
                    game.encryption_key()[buffer[x as usize] as usize].wrapping_sub(key)
                }

                Game::Totono => {
                    let mut r = buffer[x as usize];
                    r = game.encryption_key()[r as usize];
                    r = game.encryption_key()[r as usize];
                    r = game.encryption_key()[r as usize];
                    r = !r;

                    r.wrapping_sub(key).wrapping_sub(x as u8)
                }

                _ => game.encryption_key()[buffer[x as usize] as usize]
                    .wrapping_sub(key)
                    .wrapping_sub(x as u8),
            }
        }
    }

    if header.compressed {
        let mut z_buffer = Vec::with_capacity(entry.original_size as usize);
        let mut decoder = ZlibDecoder::new(reader);

        debug!("Decompressing \"{}\"", entry.file_path.display());

        decoder.read_to_end(&mut z_buffer)?;

        if z_buffer.len() != entry.original_size as usize {
            log::warn!(
                "Warning while decompressing \"{}\": decompressed size ({}) != expected size ({})",
                entry.file_path.display(),
                z_buffer.len(),
                entry.original_size
            );
        }

        buffer = z_buffer;
    }

    let extension = entry
        .file_path
        .extension()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Entry has no extension")
        })?
        .to_string_lossy()
        .to_lowercase();

    let can_infer = infer::is_supported(extension.as_str());

    if !can_infer {
        debug!("Decoding \"{}\"", entry.file_path.display());

        let result = util::decode_text(&buffer);
        if result.had_errors() {
            log::warn!("Failed to cleanly decode file: {}", result.text());
        }

        buffer = result.text().as_bytes().to_vec();
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use log::{debug, info};
    use std::{
        fs::{DirEntry, File},
        path::PathBuf,
    };
    use strum::IntoEnumIterator;
    use test_log::test;

    use super::*;

    fn archives() -> impl Iterator<Item = DirEntry> {
        let test_dir = PathBuf::from(format!(
            "{}/test_data/",
            std::env::var("CARGO_MANIFEST_DIR").unwrap()
        ));

        let dir = test_dir.read_dir().unwrap();

        dir.filter_map(Result::ok)
            .filter(|e| !e.path().is_dir() && e.path().extension() == Some("npa".as_ref()))
    }

    #[test]
    #[ignore]
    fn test_read_entry_data() {
        assert!(archives().all(|entry| {
            let path = entry.path();

            info!("Reading \"{}\"...", path.display());

            Game::iter().any(|game| {
                debug!("Trying with {game:?}");
                let mut reader = File::open(&path).unwrap();
                let head = parse_head(&mut reader).unwrap();

                let file_entry = (0..head.total_count).find_map(|index| {
                    let entry = read_entry(
                        &mut reader,
                        index as usize,
                        &head,
                        game == Game::Lamento || game == Game::LamentoTrail,
                    )
                    .unwrap();

                    (!entry.is_directory()).then_some(entry)
                });

                file_entry.is_some_and(|entry| {
                    if entry.file_path.extension().is_none() {
                        log::warn!("No extension for \"{}\"", entry.file_path.display());

                        return false;
                    }

                    let extension = entry
                        .file_path
                        .extension()
                        .expect("Entry has no extension")
                        .to_string_lossy()
                        .to_lowercase();

                    let data = read_entry_data(&mut reader, &head, &entry, game);

                    if data.is_err() {
                        log::error!(
                            "Error reading entry data for \"{}\" - {}:\n\t{:#?}",
                            entry.file_path.display(),
                            data.err().unwrap(),
                            entry
                        );

                        return false;
                    }

                    data.is_ok_and(|data| {
                        if infer::is_supported(extension.as_str()) {
                            debug!("Validating \"{}\" for file type", entry.file_path.display());

                            infer::is(&data, extension.as_str())
                        } else {
                            debug!("Validating \"{}\" for UTF-8", entry.file_path.display());
                            std::str::from_utf8(&data).is_ok()
                        }
                    })
                })
            })
        }));
    }

    // NOTE: I'm not sure any of these tests are automatable, so I'm leaving it to manually checking at the output.

    #[test]
    #[ignore]
    fn test_parse_head() {
        archives().for_each(|entry| {
            let path = entry.path();

            info!("Reading \"{}\"...", path.display());

            let mut reader = std::fs::File::open(path).unwrap();
            let head = parse_head(&mut reader).unwrap();

            debug!("{:#?}", head);
        });
    }

    #[test]
    #[ignore]
    fn test_read_entry() {
        archives().for_each(|entry| {
            let path = entry.path();

            info!("Reading \"{}\"...", path.display());

            let mut add_bytes_reader = File::open(&path).unwrap();
            let mut normal_reader = File::open(&path).unwrap();

            let add_head = parse_head(&mut add_bytes_reader).unwrap();
            let normal_head = parse_head(&mut normal_reader).unwrap();

            let add_bytes_entry = read_entry(&mut add_bytes_reader, 0, &add_head, true).unwrap();
            let normal_entry = read_entry(&mut normal_reader, 0, &normal_head, false).unwrap();

            debug!(
                "\nOne of the entry names should be normal.\n\tDecrypted by Adding Bytes: {:?}, Is Directory: {} \n\tDecrypted by Multiplying Bytes: {:?}, Is Directory: {}",
                add_bytes_entry.file_path.display(),
                add_bytes_entry.is_directory(),
                normal_entry.file_path.display(),
                normal_entry.is_directory()
            );
        });
    }
}
