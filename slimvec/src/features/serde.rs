// Copyright © ickk, 2026

use {
  crate::SlimVec,
  ::core::{fmt, marker::PhantomData},
  ::serde_core::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, Serializer},
  },
};

impl<T> Serialize for SlimVec<T>
where
  T: Serialize,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_seq(self.as_slice())
  }
}

impl<'de, T> Deserialize<'de> for SlimVec<T>
where
  T: Deserialize<'de>,
{
  fn deserialize<D>(deserializer: D) -> Result<SlimVec<T>, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(SlimVecVisitor(PhantomData))
  }

  fn deserialize_in_place<D>(
    deserializer: D,
    slimvec: &mut SlimVec<T>,
  ) -> Result<(), D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(SlimVecInPlaceVisitor(slimvec))
  }
}

struct SlimVecVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for SlimVecVisitor<T>
where
  T: Deserialize<'de>,
{
  type Value = SlimVec<T>;

  fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("a sequence")
  }

  fn visit_seq<Seq>(self, mut seq: Seq) -> Result<SlimVec<T>, Seq::Error>
  where
    Seq: SeqAccess<'de>,
  {
    let mut slimvec = SlimVec::new();
    let capacity = seq.size_hint().unwrap_or(0);
    slimvec.reserve_hint(capacity);
    while let Some(element) = seq.next_element()? {
      slimvec.push(element);
    }
    Ok(slimvec)
  }
}

struct SlimVecInPlaceVisitor<'v, T>(&'v mut SlimVec<T>);

impl<'v, 'de, T> Visitor<'de> for SlimVecInPlaceVisitor<'v, T>
where
  T: Deserialize<'de>,
{
  type Value = ();

  fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("a sequence")
  }

  fn visit_seq<Seq>(self, mut seq: Seq) -> Result<(), Seq::Error>
  where
    Seq: SeqAccess<'de>,
  {
    self.0.clear();
    let capacity = seq.size_hint().unwrap_or(0);
    self.0.reserve_hint(capacity);
    while let Some(element) = seq.next_element()? {
      self.0.push(element);
    }
    Ok(())
  }
}
