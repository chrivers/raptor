use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::error::{DResult, DockerError};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Digest {
    Sha256([u8; 32]),
}

impl Digest {
    pub fn parse(value: &str) -> DResult<Self> {
        let (typ, hex) = value.split_once(':').ok_or(DockerError::DigestError)?;

        match typ {
            "sha256" => {
                if hex.len() != 64 {
                    return Err(DockerError::DigestError);
                }

                let mut hash = [0u8; 32];
                hex::decode_to_slice(hex, &mut hash)?;

                Ok(Self::Sha256(hash))
            }
            _ => Err(DockerError::DigestError),
        }
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sha256(hash) => {
                write!(f, "sha256:{}", hex::encode(hash))
            }
        }
    }
}

impl Debug for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sha256(hash) => f.debug_tuple("Sha256").field(&hex::encode(hash)).finish(),
        }
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Sha256(hash) => {
                serializer.serialize_str(&format!("sha256:{}", hex::encode(hash)))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = String::deserialize(deserializer)?;

        Self::parse(&value).map_err(Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::digest::Digest;

    // simple pseudo-random generator, used to provide non-pathological test cases
    #[allow(clippy::needless_range_loop)]
    fn simple_rand(data: &mut [u8]) {
        let mut acc: u64 = 0x10001;
        for i in 0..data.len() {
            acc = acc.wrapping_mul(1337).wrapping_add(i as u64);
            data[i] = (acc & 0xFF) as u8;
        }
    }

    #[test]
    fn sha256_parse() {
        let mut data = [0; 32];
        simple_rand(&mut data);

        let src = Digest::Sha256(data);
        let spec = src.to_string();
        let dst = Digest::parse(&spec).unwrap();

        assert_eq!(src, dst);
    }

    #[test]
    fn sha256_parse_roundtrip() {
        let mut src = [0; 32];
        simple_rand(&mut src);

        let spec = format!("sha256:{}", hex::encode(src));
        let Digest::Sha256(dst) = Digest::parse(&spec).unwrap();

        assert_eq!(src, dst);
    }
}
