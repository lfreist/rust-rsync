use adler32::RollingAdler32;
use md5::Digest;
use md5 as MD5;

pub fn adler32(data: &str) -> u32 {
    let mut hasher = RollingAdler32::new();
    hasher.update_buffer(&data.as_bytes());
    hasher.hash()
}

pub fn md5(data: &str) -> Digest {
    MD5::compute(&data.as_bytes())
}