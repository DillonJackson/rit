use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flate2::{write::ZlibEncoder, read::ZlibDecoder, Compression as FlateCompression};
use zstd::stream::{encode_all as zstd_compress, decode_all as zstd_decompress};
use lz4::{EncoderBuilder as Lz4Encoder, Decoder as Lz4Decoder};
use brotli::CompressorWriter as BrotliCompressor;
use brotli::Decompressor as BrotliDecompressor;
use std::fs::File;
use std::io::{Read, Write, Cursor};

// Function to read a file and return its contents as a Vec<u8>
fn read_file(file_path: &str) -> Vec<u8> {
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Failed to read file");
    data
}

// Function to compress using zlib (flate2)
fn compress_zlib(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
    encoder.write_all(data).expect("Failed to compress with zlib");
    encoder.finish().expect("Failed to finish zlib compression")
}

fn decompress_zlib(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(Cursor::new(data));
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data).expect("Failed to decompress with zlib");
    decompressed_data
}

// Function to compress using zstd
fn compress_zstd(data: &[u8]) -> Vec<u8> {
    zstd_compress(Cursor::new(data), 3).expect("Failed to compress with zstd")
}

fn decompress_zstd(data: &[u8]) -> Vec<u8> {
    zstd_decompress(Cursor::new(data)).expect("Failed to decompress with zstd")
}

// Function to compress using lz4
fn compress_lz4(data: &[u8]) -> Vec<u8> {
    let mut encoder = Lz4Encoder::new().level(4).build(Vec::new()).unwrap();
    encoder.write_all(data).expect("Failed to compress with lz4");
    let (compressed, _result) = encoder.finish();
    compressed
}

fn decompress_lz4(data: &[u8]) -> Vec<u8> {
    let mut decoder = Lz4Decoder::new(Cursor::new(data)).unwrap();
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data).expect("Failed to decompress with lz4");
    decompressed_data
}

// Function to compress using brotli
fn compress_brotli(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let mut compressor = BrotliCompressor::new(&mut compressed, 4096, 11, 22);
        compressor.write_all(data).expect("Failed to compress with brotli");
    } // The compressor goes out of scope and is dropped here
    compressed // Now you can return the compressed data
}


fn decompress_brotli(data: &[u8]) -> Vec<u8> {
    let mut decompressed_data = Vec::new();
    let mut decompressor = BrotliDecompressor::new(Cursor::new(data), 4096);
    decompressor.read_to_end(&mut decompressed_data).expect("Failed to decompress with brotli");
    decompressed_data
}

// Benchmark function to test compression speed, decompression speed, and compression ratio
fn benchmark_compression_algorithms(c: &mut Criterion) {
    let file_path = "D:/repos/rit/benchmarks/files/output-onlinefiletools.txt"; // Specify the file path
    let data = read_file(file_path);

    // Benchmark zlib (flate2)
    // c.bench_function("zlib_compression", |b| b.iter(|| {
    //     let compressed = compress_zlib(black_box(&data));
    //     let compression_ratio = compressed.len() as f64 / data.len() as f64;
    //     println!("Zlib Compression Ratio: {:.2}", compression_ratio);
    // }));

    // c.bench_function("zlib_decompression", |b| b.iter(|| {
    //     let compressed = compress_zlib(&data);
    //     decompress_zlib(black_box(&compressed));
    // }));

    // Benchmark zstd
    // c.bench_function("zstd_compression", |b| b.iter(|| {
    //     let compressed = compress_zstd(black_box(&data));
    //     let compression_ratio = compressed.len() as f64 / data.len() as f64;
    //     println!("Zstd Compression Ratio: {:.2}", compression_ratio);
    // }));

    // c.bench_function("zstd_decompression", |b| b.iter(|| {
    //     let compressed = compress_zstd(&data);
    //     decompress_zstd(black_box(&compressed));
    // }));

    // Benchmark lz4
    // c.bench_function("lz4_compression", |b| b.iter(|| {
    //     let compressed = compress_lz4(black_box(&data));
    //     let compression_ratio = compressed.len() as f64 / data.len() as f64;
    //     println!("LZ4 Compression Ratio: {:.2}", compression_ratio);
    // }));

    // c.bench_function("lz4_decompression", |b| b.iter(|| {
    //     let compressed = compress_lz4(&data);
    //     decompress_lz4(black_box(&compressed));
    // }));

    // Benchmark brotli
    c.bench_function("brotli_compression", |b| b.iter(|| {
        let compressed = compress_brotli(black_box(&data));
        let compression_ratio = compressed.len() as f64 / data.len() as f64;
        println!("Brotli Compression Ratio: {:.2}", compression_ratio);
    }));

    c.bench_function("brotli_decompression", |b| b.iter(|| {
        let compressed = compress_brotli(&data);
        decompress_brotli(black_box(&compressed));
    }));
}

criterion_group!(benches, benchmark_compression_algorithms);
criterion_main!(benches);
