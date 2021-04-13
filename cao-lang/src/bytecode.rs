use std::convert::TryInto;

pub trait TriviallyEncodable: Sized + Copy {}
impl<T: Sized + Copy> TriviallyEncodable for T {}

pub fn write_to_vec<T: TriviallyEncodable>(val: T, out: &mut Vec<u8>) {
    let len = out.len();
    let size = std::mem::size_of::<T>();
    out.resize(len + size, 0);
    unsafe {
        let ptr = out.as_mut_ptr().add(len);
        std::ptr::write_unaligned(ptr as *mut T, val);
    }
}

/// return the number of bytes read
pub fn read_from_bytes<T: TriviallyEncodable>(bts: &[u8]) -> Option<(usize, T)> {
    let size = std::mem::size_of::<T>();
    if bts.len() < size {
        return None;
    }
    unsafe { Some((size, *(bts.as_ptr() as *const T))) }
}

pub fn encode_str(s: &str, out: &mut Vec<u8>) {
    let len: u32 = s
        .len()
        .try_into()
        .expect("Failed to cast string len to u32");
    write_to_vec(len, out);
    out.extend_from_slice(s.as_bytes());
}

pub fn decode_str(bts: &[u8]) -> Option<(usize, &str)> {
    let (sl, len): (_, u32) = read_from_bytes(bts)?;
    if bts.len() - sl < len as usize {
        return None;
    }
    let bts = &bts[sl..sl + len as usize];
    Some((sl + len as usize, std::str::from_utf8(bts).ok()?))
}
