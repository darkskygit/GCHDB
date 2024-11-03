use super::*;
use lazy_static::lazy_static;
use sha3::{
    digest::{ExtendableOutput, Reset, Update},
    Shake256,
};
use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[diesel(table_name = blobs)]
pub struct Blob {
    pub hash: i64,
    pub blob: Vec<u8>,
}

impl Blob {
    pub fn new(blob: Vec<u8>) -> Self {
        Self {
            hash: Self::get_hash(&blob),
            blob,
        }
    }

    fn get_hash(data: &[u8]) -> i64 {
        lazy_static! {
            static ref HASHER: Arc<Mutex<Shake256>> = Arc::new(Mutex::new(Shake256::default()));
        }
        i64::from_ne_bytes(
            {
                let mut hasher = HASHER.lock().unwrap();
                hasher.update(data.as_ref());
                let ret = hasher.clone();
                hasher.reset();
                let mut buf = vec![0u8; 8];
                if ret.finalize_xof().read(&mut buf).is_err() {
                    buf = vec![0u8, 8];
                }
                buf
            }
            .as_slice()
            .try_into()
            .expect("slice with incorrect length"),
        )
    }
}
