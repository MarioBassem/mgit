pub struct Blob {
    pub data: Vec<u8>,
}

pub fn decode_blob(data: Vec<u8>) -> Blob {
    Blob { data }
}

pub fn encode_blob(blob: Blob) -> Vec<u8> {
    blob.data
}
