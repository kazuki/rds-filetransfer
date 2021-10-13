pub const HEADER_SIZE: usize = 4;

pub struct Header {
    pub seq: u32,
    pub size: u16,
}

pub fn parse_header(b: &[u8]) -> Header {
    let x = (b[0] as u32)
        | ((b[1] as u32) << 8)
        | ((b[2] as u32) << 16)
        | ((b[3] as u32) << 24);
    Header {
        seq: x >> 12,
        size: (x & 0xfff) as u16
    }
}

pub fn build_header(h: Header, output: &mut [u8]) {
    let x = (h.seq << 12) | (h.size as u32);
    output[0] = (x & 0xff) as u8;
    output[1] = ((x >> 8) & 0xff) as u8;
    output[2] = ((x >> 16) & 0xff) as u8;
    output[3] = ((x >> 24) & 0xff) as u8;
}
