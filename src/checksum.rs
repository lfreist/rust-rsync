use adler32::RollingAdler32;
use md5 as MD5;

pub fn adler32(data: &[u8]) -> u32 {
    let mut hasher = RollingAdler32::new();
    hasher.update_buffer(&data);
    hasher.hash()
}

pub fn md5(data: &[u8]) -> String {
    let data = MD5::compute(&data);
    hex::encode(data.0)
}