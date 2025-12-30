use super::NpaHead;

pub fn decrypt_header(current_number: u32, current_file: u32, header: &NpaHead, add_if_encrypted: bool) -> u8 {
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
