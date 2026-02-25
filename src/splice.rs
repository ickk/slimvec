// © ickk 2026.

use {
  crate::{Drain, SlimVec},
  ::core::{mem, mem::ManuallyDrop, ops::RangeBounds},
};

/// An iterator that moves out of a [`SlimVec`], replacing the removed elements
/// with another iterator
///
/// This may be created using [`SlimVec::splice`].
#[derive(Debug)]
pub struct Splice<'v, I>
where
  I: Iterator + 'v,
{
  // All the splicing magic happens in Drop, overriding Drain's Drop.
  drain: ManuallyDrop<Drain<'v, I::Item>>,
  replacements: I,
}

impl<I> Iterator for Splice<'_, I>
where
  I: Iterator,
{
  type Item = I::Item;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.drain.next()
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.drain.size_hint()
  }
}

impl<I> DoubleEndedIterator for Splice<'_, I>
where
  I: Iterator,
{
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    self.drain.next_back()
  }
}

impl<I> ExactSizeIterator for Splice<'_, I> where I: Iterator {}

/// Internal methods
impl<I, T> Splice<'_, I>
where
  I: Iterator<Item = T>,
{
  #[inline]
  pub(crate) fn new(
    slimvec: &mut SlimVec<T>,
    range: impl RangeBounds<usize>,
    replace_with: impl IntoIterator<IntoIter = I>,
  ) -> Splice<'_, I> {
    Splice {
      drain: ManuallyDrop::new(Drain::new(slimvec, range)),
      replacements: replace_with.into_iter(),
    }
  }
}

impl<I> Drop for Splice<'_, I>
where
  I: Iterator,
{
  fn drop(&mut self) {
    // Take the `yield_range`, for the sake of unwind-safety; prevent running
    // destructors twice if a destructor panics.
    let yield_range = mem::take(&mut self.drain.yield_range);
    unsafe { self.drain.slimvec.raw.drop_in_place(yield_range) };

    // If there is no tail, this reduces to `extend`.
    if self.drain.tail.is_empty() {
      self.drain.slimvec.extend(&mut self.replacements);
      return;
    }

    // If the size-hint appears to be exact, then use the size-hint to resize
    // the void.
    if let (size_hint, Some(upper)) = self.replacements.size_hint()
      && size_hint == upper
      && size_hint != self.drain.void().len()
    {
      let len = self.drain.slimvec.len();
      let tail_len = self.drain.tail.len();
      self.drain.slimvec.reserve(size_hint + tail_len);
      unsafe { self.drain.shift_tail(len + size_hint) }
    }

    // Fill the void. If there were too few replacements to fill the void
    // completely, then shift the tail back & return.
    self.drain.fill_void(&mut self.replacements);
    if !self.drain.void().is_empty() {
      let len = self.drain.slimvec.len();
      let tail_len = self.drain.tail.len();
      unsafe {
        self.drain.shift_tail(len);
        self.drain.slimvec.raw.set_length(len + tail_len)
      };
      return;
    }

    // Fallback; collect all remaining replacements into a temporary buffer,
    // reserve capacity, move the tail, and fill the void.
    // Does nothing if there are no more replacements.
    let mut replacements = self.replacements.by_ref().collect::<SlimVec<_>>();
    let len = self.drain.slimvec.len();
    let tail_len = self.drain.tail.len();
    let rep_len = replacements.len();
    if !replacements.is_empty() {
      self.drain.slimvec.reserve(rep_len + tail_len);
      unsafe { self.drain.shift_tail(len + rep_len) };
      self.drain.slimvec.append(&mut replacements);
    }
    unsafe { self.drain.slimvec.raw.set_length(len + rep_len + tail_len) };
  }
}
