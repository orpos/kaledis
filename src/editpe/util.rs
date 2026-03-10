use core::{
    any::type_name,
    ops::{Add, Rem, Sub},
};

use zerocopy::FromBytes;

use super::ReadError;

pub fn read<T: FromBytes + Copy>(resource: &[u8]) -> Result<T, ReadError> {
    T::read_from_prefix(resource)
        .map_err(|_| ReadError(type_name::<T>().to_string()))
        .map(|(value, _)| value)
}

pub fn aligned_to<T: Add<Output = T> + Sub<Output = T> + Rem<Output = T> + Eq + Copy + Default>(
    value: T,
    alignment: T,
) -> T {
    if value % alignment == T::default() {
        return value;
    }
    value + alignment - (value % alignment)
}

pub fn read_u16_string(data: &[u8]) -> Result<String, ReadError> {
    let mut string = String::new();
    for i in 0..(data.len() / 2) {
        let c = read::<u16>(&data[i * 2..])?;
        if c == 0 {
            break;
        }
        string.push(core::char::from_u32(c as u32).unwrap());
    }
    Ok(string)
}

pub fn string_to_u16<S: AsRef<str>>(string: S) -> Vec<u8> {
    let string = string.as_ref();
    let mut data = Vec::with_capacity(string.len() * 2 + 2);
    data.extend(string.encode_utf16().flat_map(|c| c.to_le_bytes().to_vec()));
    data.extend(Vec::from([0, 0]));
    data
}
