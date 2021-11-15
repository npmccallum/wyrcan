use std::io::Write;
use std::str::FromStr;

use anyhow::Result;
use digest::{generic_array::GenericArray, Digest as D};
use serde::Deserialize;
use sha2::{Sha224, Sha256, Sha384, Sha512};

use crate::iotools::Validatable;

#[inline(always)]
fn dehex(byte: u8) -> Result<u8, Invalid> {
    Ok(match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => return Err(Invalid::Encoding),
    })
}

#[derive(Clone, Debug)]
enum Inner {
    Sha224(Sha224, GenericArray<u8, <Sha224 as D>::OutputSize>),
    Sha256(Sha256, GenericArray<u8, <Sha256 as D>::OutputSize>),
    Sha384(Sha384, GenericArray<u8, <Sha384 as D>::OutputSize>),
    Sha512(Sha512, GenericArray<u8, <Sha512 as D>::OutputSize>),
}

impl AsRef<[u8]> for Inner {
    fn as_ref(&self) -> &[u8] {
        match self {
            Inner::Sha224(.., a) => a.as_ref(),
            Inner::Sha256(.., a) => a.as_ref(),
            Inner::Sha384(.., a) => a.as_ref(),
            Inner::Sha512(.., a) => a.as_ref(),
        }
    }
}

impl AsMut<[u8]> for Inner {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Inner::Sha224(.., a) => a.as_mut(),
            Inner::Sha256(.., a) => a.as_mut(),
            Inner::Sha384(.., a) => a.as_mut(),
            Inner::Sha512(.., a) => a.as_mut(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Invalid {
    Algorithm,
    Encoding,
    Length,
}

impl std::error::Error for Invalid {}
impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Invalid::Algorithm => f.write_str("invalid digest algorithm"),
            Invalid::Encoding => f.write_str("invalid digest encoding"),
            Invalid::Length => f.write_str("invalid digest length"),
        }
    }
}

impl FromStr for Inner {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (mut i, h) = if let Some((lhs, rhs)) = s.find(':').map(|x| s.split_at(x)) {
            let inner = if lhs.eq_ignore_ascii_case("sha224") {
                Inner::Sha224(Sha224::new(), Default::default())
            } else if lhs.eq_ignore_ascii_case("sha256") {
                Inner::Sha256(Sha256::new(), Default::default())
            } else if lhs.eq_ignore_ascii_case("sha384") {
                Inner::Sha384(Sha384::new(), Default::default())
            } else if lhs.eq_ignore_ascii_case("sha512") {
                Inner::Sha512(Sha512::new(), Default::default())
            } else {
                return Err(Invalid::Algorithm);
            };

            (inner, &rhs[1..])
        } else if s.len() == 64 {
            (Inner::Sha256(Sha256::new(), Default::default()), s)
        } else {
            return Err(Invalid::Algorithm);
        };

        if h.len() != i.as_ref().len() * 2 {
            return Err(Invalid::Length);
        }

        let mut chars = h.as_bytes().iter();
        for b in i.as_mut().iter_mut() {
            let l = *chars.next().unwrap();
            let r = *chars.next().unwrap();
            *b = dehex(l)? << 4 | dehex(r)?;
        }

        Ok(i)
    }
}

struct Visitor;
impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Digest;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string in the format `ALGO:HASH`")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        let i = Inner::from_str(v).map_err(|e| E::custom(format!("{}", e)))?;
        Ok(Digest(i))
    }

    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        self.visit_str(&v)
    }
}

/// A cryptographic digest
///
/// This digest is most often represented in the form 'ALGORITHM:HEX_BYTES'.
///
/// A digest instance implements `std::io::Write` so you can write directly
/// into it. You can also `validate()` the data written to it to confirm
/// integrity.
#[derive(Clone, Debug)]
pub struct Digest(Inner);

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(Visitor)
    }
}

impl FromStr for Digest {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl Digest {
    pub fn algorithm(&self) -> &str {
        match self.0 {
            Inner::Sha224(..) => "sha224",
            Inner::Sha256(..) => "sha256",
            Inner::Sha384(..) => "sha384",
            Inner::Sha512(..) => "sha512",
        }
    }
}

impl Write for Digest {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.0 {
            Inner::Sha224(w, ..) => w.write(buf),
            Inner::Sha256(w, ..) => w.write(buf),
            Inner::Sha384(w, ..) => w.write(buf),
            Inner::Sha512(w, ..) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.0 {
            Inner::Sha224(w, ..) => w.flush(),
            Inner::Sha256(w, ..) => w.flush(),
            Inner::Sha384(w, ..) => w.flush(),
            Inner::Sha512(w, ..) => w.flush(),
        }
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.algorithm())?;

        for byte in self.0.as_ref().iter().cloned() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl Validatable for Digest {
    fn validate(&self) -> bool {
        match &self.0 {
            Inner::Sha224(w, h) => &w.clone().finalize() == h,
            Inner::Sha256(w, h) => &w.clone().finalize() == h,
            Inner::Sha384(w, h) => &w.clone().finalize() == h,
            Inner::Sha512(w, h) => &w.clone().finalize() == h,
        }
    }
}
