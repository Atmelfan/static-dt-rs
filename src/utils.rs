
pub fn read_fdt_u32(buf: &[u8], offs: usize) -> u32 {
    (buf[offs+0] as u32) << 24
        | (buf[offs+1] as u32) << 16
        | (buf[offs+2] as u32) << 8
        | (buf[offs+3] as u32) << 0
}

pub fn read_fdt_u64(buf: &[u8], offs: usize) -> u64 {
    (buf[offs + 0] as u64) << 56
        | (buf[offs + 1] as u64) << 48
        | (buf[offs + 2] as u64) << 40
        | (buf[offs + 3] as u64) << 32
        | (buf[offs + 4] as u64) << 24
        | (buf[offs + 5] as u64) << 16
        | (buf[offs + 6] as u64) << 8
        | (buf[offs + 7] as u64) << 0
}

pub fn get_fdt_string(buf: &[u8], offs: usize) -> Option<&[u8]> {
    for (i, c) in buf[offs..].iter().enumerate() {
        if *c == 0u8 {
            return Some(&buf[offs..offs+i])
        }
    }
    None
}