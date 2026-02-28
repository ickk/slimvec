// Copyright © ickk, 2026

use {
  crate::{SlimVec, utils::TypeMeta},
  ::borsh::{
    BorshDeserialize, BorshSerialize,
    io::{Error, ErrorKind, Read, Result, Write},
  },
};

impl<T> BorshSerialize for SlimVec<T>
where
  T: BorshSerialize,
{
  #[inline]
  fn serialize<W>(&self, writer: &mut W) -> Result<()>
  where
    W: Write,
  {
    if T::IS_ZST {
      return Err(Error::zst_not_allowed());
    }

    self.as_slice().serialize(writer)
  }
}

impl<T> BorshDeserialize for SlimVec<T>
where
  T: BorshDeserialize,
{
  #[inline]
  fn deserialize_reader<R>(reader: &mut R) -> Result<Self>
  where
    R: Read,
  {
    if T::IS_ZST {
      return Err(Error::zst_not_allowed());
    }

    let mut slimvec = SlimVec::new();
    let length = {
      let v = u32::deserialize_reader(reader)?;
      usize::try_from(v).map_err(|_| Error::out_of_memory())?
    };
    slimvec.reserve_hint(length);
    for _ in 0..length {
      slimvec.push(T::deserialize_reader(reader)?);
    }
    Ok(slimvec)
  }
}

trait ErrorExt {
  #[cold]
  fn zst_not_allowed() -> Error {
    Error::new(
      ErrorKind::InvalidData,
      "Collections of zero-sized types are not allowed.",
    )
  }
  #[cold]
  fn out_of_memory() -> Error {
    Error::from(ErrorKind::OutOfMemory)
  }
}
impl ErrorExt for Error {}
