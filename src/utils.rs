#![allow(unused)]

pub fn u8_to_bits(value: u8) -> Vec<bool> {
    let mut bits = Vec::new();
    for i in 0..8 {
        bits.push(value & (1 << i) != 0);
    }
    bits.reverse(); // 需要反转，因为低位在前
    bits
}

pub fn u64_to_bits(value: u64) -> Vec<bool> {
    let mut bits = Vec::new();
    for i in 0..64 {
        bits.push(value & (1 << i) != 0);
    }
    bits.reverse(); // 需要反转，因为低位在前
    bits
}

pub fn u64_to_u8_array(n: u64) -> [u8; 8] {
    let mut result = [0u8; 8];
    for i in 0..8 {
        result[i] = ((n >> (i * 8)) & 0xFF) as u8;
    }
    result
}

pub fn encode_fixed_length(input: &str, length: usize) -> Result<Vec<u8>, &'static str> {
    let utf8_bytes = input.as_bytes();

    if utf8_bytes.len() > length {
        return Err("Input string exceeds specified length");
    }

    let mut result = vec![0u8; length];
    result[..utf8_bytes.len()].copy_from_slice(utf8_bytes);

    Ok(result)
}

pub fn hex_string_to_bytes(hex_string: &str, length: usize) -> Option<Vec<u8>> {
    // 检查字符串长度是否为偶数
    if hex_string.len() % 2 != 0 {
        return None;
    }

    // 计算字节数组的长度
    let byte_length = hex_string.len() / 2;

    // 检查是否超过指定长度
    if byte_length > length {
        return None;
    }

    // 创建一个新的字节数组
    let mut bytes = vec![0; length];

    // 计算填充0的数量
    let padding = length.saturating_sub(byte_length);

    // 迭代字符串中的每两个字符
    for (i, chunk) in hex_string.as_bytes().chunks(2).enumerate() {
        // 计算当前字节的索引
        let index = i * 2;

        // 将每两个字符解析为一个十六进制数
        let hex_byte = match u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16) {
            Ok(byte) => byte,
            Err(_) => return None, // 解析失败，返回 None
        };

        // 将解析后的字节放入结果数组中
        bytes[padding + index / 2] = hex_byte;
    }

    Some(bytes)
}
