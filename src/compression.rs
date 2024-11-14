use std::io::{self, Cursor};
use zstd::stream::{encode_all as zstd_compress, decode_all as zstd_decompress};

// Compress data using zstd with compression level 3
pub fn compress_data(data: &[u8]) -> io::Result<Vec<u8>> {
    let compressed_data = zstd_compress(Cursor::new(data), 3)?;
    Ok(compressed_data)
}

// Decompress data using zstd
pub fn uncompress_data(data: &[u8]) -> io::Result<Vec<u8>> {
    let decompressed_data = zstd_decompress(Cursor::new(data))?;
    Ok(decompressed_data)
}