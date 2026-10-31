use brotli::Decompressor;
use flate2::bufread::ZlibDecoder;
use std::io::Read;

/// Upper bound for a single decompressed danmaku frame.
///
/// The danmaku stream is compressed with a ~1000x amplification factor, so
/// feeding `read_to_end` a hostile payload lets a ~68 KiB frame force a
/// ~70 MiB allocation. Legitimate frames are a few KiB, so this generous
/// bound costs nothing in practice.
const MAX_DECOMPRESSED_SIZE: u64 = 8 * 1024 * 1024;

fn read_limited<R: Read>(reader: R, format: &str) -> Result<Vec<u8>, String> {
    let mut decompressed = Vec::new();

    // Read one byte past the limit so we can distinguish "exactly at the
    // limit" from "over the limit" instead of silently truncating.
    reader
        .take(MAX_DECOMPRESSED_SIZE + 1)
        .read_to_end(&mut decompressed)
        .map_err(|e| e.to_string())?;

    if decompressed.len() as u64 > MAX_DECOMPRESSED_SIZE {
        return Err(format!(
            "{} payload exceeds the {} byte limit",
            format, MAX_DECOMPRESSED_SIZE
        ));
    }

    Ok(decompressed)
}

pub fn brotli_decode(data: Vec<u8>) -> Result<Vec<u8>, String> {
    read_limited(Decompressor::new(&data[..], 4096), "brotli")
}

pub fn zlib_decode(data: Vec<u8>) -> Result<Vec<u8>, String> {
    read_limited(ZlibDecoder::new(&data[..]), "zlib")
}
