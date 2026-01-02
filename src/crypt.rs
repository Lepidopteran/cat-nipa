use super::*;

pub fn decrypt_header(
    current_number: u32,
    current_file: u32,
    header: &NpaHead,
    add_if_encrypted: bool,
) -> u8 {
    let mut key = 0xFCu32.wrapping_mul(current_number);

    let temp = if add_if_encrypted && header.encrypted {
        header.key_1.wrapping_add(header.key_2)
    } else {
        header.key_1.wrapping_mul(header.key_2)
    };

    key = key
        .wrapping_sub(temp >> 24)
        .wrapping_sub(temp >> 16)
        .wrapping_sub(temp >> 8)
        .wrapping_sub(temp & 0xFF);

    key = key
        .wrapping_sub(current_file >> 24)
        .wrapping_sub(current_file >> 16)
        .wrapping_sub(current_file >> 8)
        .wrapping_sub(current_file & 0xFF);

    (key & 0xFF) as u8
}

pub fn decrypt_data(entry: &NpaEntry, header: &NpaHead, game: Game) -> u8 {
    let mut key_1: u32 = match game {
        Game::Axanael
        | Game::Kikokugai
        | Game::SonicomiTrialTwo
        | Game::Sonicomi
        | Game::LostX
        | Game::DramaticalMurder
        | Game::DramaticalMurderReConnect
        | Game::MuramasaSS => 0x20101118,
        Game::Totono => 0x12345678,
        _ => 0x87654321,
    };

    for byte in entry.un_decoded_file_path.as_bytes() {
        key_1 -= *byte as u32;
    }

    let key_2 = header.key_1.wrapping_mul(header.key_2);

    let mut key = key_1.wrapping_mul(entry.name_length);

    if game != Game::Lamento && game != Game::LamentoTrail {
        key = key.wrapping_add(key_2);
        key = key.wrapping_mul(entry.original_size);
    }

    key as u8
}
