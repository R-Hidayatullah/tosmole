#![allow(dead_code)]

pub(crate) fn decrypt_string(data: &[u8]) -> String {
    let xor_key = 1;
    let decrypted_data: Vec<u8> = data.iter().map(|&byte| byte ^ xor_key).collect();

    String::from_utf8(decrypted_data)
        .unwrap()
        .to_string()
        .trim_end_matches('\u{1}')
        .to_string()
}
