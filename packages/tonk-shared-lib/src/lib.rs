use bincode::{config, Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Location(pub String, pub String, pub String, pub String);

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Player {
    pub id: String,
    pub location: Location
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Building {
    pub id: String,
    pub location: Location
}

pub fn serialize_struct<T: Encode>(obj: T) -> Vec<u8> {
    let config = config::standard();
    bincode::encode_to_vec(&obj, config).unwrap()
}

pub fn deserialize_struct<T: Decode>(vec: Vec<u8>) -> T {
    let config = config::standard();
    let (decoded, len): (T, usize) = bincode::decode_from_slice(&vec, config).unwrap();
    decoded
}