use flate2::read::ZlibDecoder;
use lz4::block::decompress_to_buffer as LZ4_decompress_to_buffer;
use std::io::Read;
use xz2::read::XzDecoder;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

// Note: this contains ZL[src][dst] where src and dst are 3 bytes each.
const HEADER_SIZE: usize = 9;

// because each zipped block contains:
// - the size of the input data
// - the size of the compressed data
// where each size is saved on 3 bytes, the maximal size
// of each block can not be bigger than 16Mb.
#[allow(dead_code)]
const K_MAX_COMPRESSED_BLOCK_SIZE: usize = 0xffffff;

#[allow(dead_code)]
#[derive(PartialEq)]
enum Kind {
    #[allow(dead_code)]
    Inherit = -1,
    UseGlobal = 0,
    Zlib = 1,
    Lzma = 2,
    OldCompression = 3,
    LZ4 = 4,
    Zstd = 5,
    UndefinedCompression = 6,
}

// kindOf returns the kind of compression algorithm.
fn kind_of(buf: &[u8]) -> Kind {
    match (buf[0] as char, buf[1] as char) {
        ('Z', 'L') => Kind::Zlib,
        ('X', 'Z') => Kind::Lzma,
        ('L', '4') => Kind::LZ4,
        ('Z', 'S') => Kind::Zstd,
        ('C', 'S') => Kind::OldCompression,

        _ => Kind::UndefinedCompression,
    }
}

pub fn decompress(dst: &mut [u8], mut src: &[u8]) -> Result<usize> {
    let mut beg: usize = 0;
    let mut end: i64 = 0;
    let buflen = dst.len() as i64;
    let mut hdr = [0_u8; HEADER_SIZE];

    while end < buflen {
        src.read_exact(&mut hdr)?;

        let srcsz = hdr[3] as usize | (hdr[4] as usize) << 8 | (hdr[5] as usize) << 16;
        let tgtsz = hdr[6] as usize | (hdr[7] as usize) << 8 | (hdr[8] as usize) << 16;
        end += tgtsz as i64;

        let block_dst = &mut dst[beg..beg + tgtsz];

        match kind_of(hdr.as_ref()) {
            Kind::Inherit => {
                unimplemented!()
            }
            Kind::UseGlobal => {
                unimplemented!()
            }

            Kind::Zlib => {
                let mut d = ZlibDecoder::new(&src[..srcsz]);
                d.read_exact(block_dst)?;
                src = &src[srcsz..];
            }
            Kind::Lzma => {
                let mut d = XzDecoder::new(&src[..srcsz]);
                d.read_exact(block_dst)?;
                src = &src[srcsz..];
            }
            Kind::OldCompression => {
                unimplemented!()
            }
            Kind::LZ4 => {
                // The first 8 bytes are an LZ4 block sub-header (already counted in srcsz).
                // The actual compressed payload starts at offset 8.
                LZ4_decompress_to_buffer(&src[8..srcsz], Some(tgtsz as i32), block_dst)?;
                src = &src[srcsz..];
            }
            Kind::Zstd => {
                unimplemented!()
            }
            Kind::UndefinedCompression => {
                unimplemented!()
            }
        }

        beg += tgtsz;
    }

    Ok(beg)
}

fn root_compress_algo_level(algo: i32) -> (Kind, i32) {
    let kind = algo / 100;
    let level = algo % 100;
    let kind = match kind {
        0 => Kind::UseGlobal,
        1 => Kind::Zlib,
        2 => Kind::Lzma,
        3 => Kind::OldCompression,
        4 => Kind::LZ4,
        5 => Kind::Zstd,
        _ => Kind::UndefinedCompression,
    };
    (kind, level)
}

pub fn compress(src: Vec<u8>, compression: i32) -> Result<Vec<u8>> {
    assert_eq!(compression, 1);

    let (kind, level) = root_compress_algo_level(compression);

    if kind == Kind::UseGlobal || level == 0 || src.len() < 512 {
        // no compression
        // let mut dst = Vec::new();
        //std::io::copy(src, &mut dst)?;
        return Ok(src);
    }

    unimplemented!();

    // let mut dst = Vec::new();
    // {
    //     let mut w = flate2::write::ZlibEncoder::new(&mut dst, flate2::Compression::default());
    //     std::io::copy(&mut src, &mut w)?;
    // }
    //
    // Ok(dst)
}
