// © ickk 2026, All Rights Reserved.

//! Assert that all the same traits and constraints exist for [`slimvec`] as do
//! for [`std::vec`].
//!
//! The following traits, considered to be implementation details or advisory
//! only, are excluded:
//! - `Drop`
//! - `UnwindSafe`
//! - `RefUnwindowSafe`

#![expect(non_snake_case, reason = "test names correspond to trait names")]
extern crate alloc;

mod SlimVec {
  use {
    crate::helpers::*,
    ::core::{
      borrow::{Borrow, BorrowMut},
      fmt::Debug,
      hash::Hash,
      ops::{Deref, DerefMut, Index, IndexMut, Range},
      slice,
    },
    ::impls::impls,
    ::slimvec::{IntoIter, SlimVec},
    ::std::io::Write,
  };

  #[test]
  fn AsMut() {
    assert!(impls!(SlimVec<E>: AsMut<[E]>));
    assert!(impls!(SlimVec<E>: AsMut<SlimVec<E>>));
  }

  #[test]
  fn AsRef() {
    assert!(impls!(SlimVec<E>: AsRef<[E]>));
    assert!(impls!(SlimVec<E>: AsRef<SlimVec<E>>));
  }

  #[test]
  fn Borrow() {
    assert!(impls!(SlimVec<E>: Borrow<[E]>));
  }

  #[test]
  fn BorrowMut() {
    assert!(impls!(SlimVec<E>: BorrowMut<[E]>));
  }

  #[test]
  fn Clone() {
    assert!(impls!(SlimVec<HasClone>: Clone));

    assert!(impls!(SlimVec<E>: !Clone));
  }

  #[test]
  fn Debug() {
    assert!(impls!(SlimVec<HasDebug>: Debug));

    assert!(impls!(SlimVec<E>: !Debug));
  }

  #[test]
  fn Default() {
    assert!(impls!(SlimVec<E>: Default));
  }

  #[test]
  fn Deref() {
    assert!(impls!(SlimVec<E>: Deref<Target = [E]>));
  }

  #[test]
  fn DerefMut() {
    assert!(impls!(SlimVec<E>: DerefMut));
  }

  #[test]
  fn Eq() {
    assert!(impls!(SlimVec<HasEq>: Eq));

    assert!(impls!(SlimVec<E>: !Eq));
  }

  #[test]
  fn Extend() {
    assert!(impls!(SlimVec<HasCopy>: Extend<&'static HasCopy>));
    assert!(impls!(SlimVec<E>: Extend<E>));
  }

  // #[test]
  // fn From() {
  //   todo!()
  // }

  #[test]
  fn FromIterator() {
    assert!(impls!(SlimVec<E>: FromIterator<E>));
  }

  #[test]
  fn Hash() {
    assert!(impls!(SlimVec<HasHash>: Hash));

    assert!(impls!(SlimVec<E>: !Hash));
  }

  #[test]
  fn Index() {
    assert!(impls!(SlimVec<E>: Index<usize, Output = E>));
    assert!(impls!(SlimVec<E>: Index<Range<usize>, Output = [E]>));
  }

  #[test]
  fn IndexMut() {
    assert!(impls!(SlimVec<E>: IndexMut<usize>));
    assert!(impls!(SlimVec<E>: IndexMut<Range<usize>>));
  }

  #[test]
  fn IntoIterator() {
    assert!(
      impls!(&SlimVec<E>: IntoIterator<Item = &'static E, IntoIter = slice::Iter<'static, E>>)
    );
    assert!(
      impls!(&mut SlimVec<E>: IntoIterator<Item = &'static mut E, IntoIter = slice::IterMut<'static, E>>)
    );
    assert!(
      impls!(SlimVec<E>: IntoIterator<Item = E, IntoIter = IntoIter<E>>)
    );
  }

  #[test]
  fn Ord() {
    assert!(impls!(SlimVec<HasOrd>: Ord));

    assert!(impls!(SlimVec<E>: !Ord));
  }

  #[test]
  fn PartialEq() {
    assert!(impls!(SlimVec<HasPartialEq>: PartialEq<SlimVec<HasPartialEq>>));
    assert!(impls!(SlimVec<HasPartialEq>: PartialEq<[HasPartialEq]>));
    assert!(impls!([HasPartialEq]: PartialEq<SlimVec<HasPartialEq>>));
    assert!(impls!(SlimVec<HasPartialEq>: PartialEq<&'static [HasPartialEq]>));
    assert!(impls!(&'static [HasPartialEq]: PartialEq<SlimVec<HasPartialEq>>));
    assert!(
      impls!(SlimVec<HasPartialEq>: PartialEq<&'static mut [HasPartialEq]>)
    );
    assert!(
      impls!(&'static mut [HasPartialEq]: PartialEq<SlimVec<HasPartialEq>>)
    );
    assert!(impls!(SlimVec<HasPartialEq>: PartialEq<[HasPartialEq; 64]>));
    assert!(
      impls!(SlimVec<HasPartialEq>: PartialEq<&'static [HasPartialEq; 64]>)
    );
    assert!(impls!(SlimVec<HasPartialEq>: PartialEq<Vec<HasPartialEq>>));
    assert!(impls!(Vec<HasPartialEq>: PartialEq<SlimVec<HasPartialEq>>));

    // TODO
    // assert!(
    //   impls!(Cow<'static, [HasPartialEq]>: PartialEq<SlimVec<HasPartialEq>>)
    // );
    // assert!(impls!(VecDeque<HasPartialEq>: PartialEq<SlimVec<HasPartialEq>>));
  }

  #[test]
  fn PartialOrd() {
    assert!(
      impls!(SlimVec<HasPartialOrd>: PartialOrd<SlimVec<HasPartialOrd>>)
    );

    assert!(impls!(SlimVec<E>: !PartialOrd<E>));
  }

  // #[test]
  // fn TryFrom() {
  //   todo!()
  // }

  #[test]
  fn Write() {
    assert!(impls!(SlimVec<u8>: Write))
  }

  #[test]
  fn Send() {
    assert!(impls!(SlimVec<HasSend>: Send));

    assert!(impls!(SlimVec<E>: !Send));
  }

  #[test]
  fn Sync() {
    assert!(impls!(SlimVec<HasSync>: Sync));

    assert!(impls!(SlimVec<E>: !Sync));
  }

  #[test]
  fn Unpin() {
    assert!(impls!(SlimVec<HasUnpin>: Unpin));

    assert!(impls!(SlimVec<E>: !Unpin));
  }
}

mod IntoIter {
  use {
    crate::helpers::*,
    ::core::{fmt::Debug, iter::FusedIterator},
    ::impls::impls,
    ::slimvec::IntoIter,
  };

  #[test]
  fn AsRef() {
    assert!(impls!(IntoIter<E>: AsRef<[E]>));
  }

  #[test]
  fn Clone() {
    assert!(impls!(IntoIter<HasClone>: Clone));

    assert!(impls!(IntoIter<E>: !Clone));
  }

  #[test]
  fn Debug() {
    assert!(impls!(IntoIter<HasDebug>: Debug));

    assert!(impls!(IntoIter<E>: !Debug));
  }

  #[test]
  fn Default() {
    assert!(impls!(IntoIter<E>: Default));
  }

  #[test]
  fn DoubleEndedIterator() {
    assert!(impls!(IntoIter<E>: DoubleEndedIterator));
  }

  #[test]
  fn ExactSizeIterator() {
    assert!(impls!(IntoIter<E>: ExactSizeIterator));
  }

  #[test]
  fn FusedIterator() {
    assert!(impls!(IntoIter<E>: FusedIterator));
  }

  #[test]
  fn Iterator() {
    assert!(impls!(IntoIter<E>: Iterator<Item = E>));
  }

  #[test]
  fn Send() {
    assert!(impls!(IntoIter<HasSend>: Send));

    assert!(impls!(IntoIter<E>: !Send));
  }

  #[test]
  fn Sync() {
    assert!(impls!(IntoIter<HasSync>: Sync));

    assert!(impls!(IntoIter<E>: !Sync));
  }

  #[test]
  fn Unpin() {
    assert!(impls!(IntoIter<HasUnpin>: Unpin));

    assert!(impls!(IntoIter<E>: !Unpin));
  }
}

mod Drain {
  use {
    crate::helpers::*,
    ::core::{fmt::Debug, iter::FusedIterator},
    ::impls::impls,
    ::slimvec::Drain,
  };

  #[test]
  fn AsRef() {
    assert!(impls!(Drain<E>: AsRef<[E]>));
  }

  #[test]
  fn Debug() {
    assert!(impls!(Drain<HasDebug>: Debug));

    assert!(impls!(Drain<E>: !Debug));
  }

  #[test]
  fn DoubleEndedIterator() {
    assert!(impls!(Drain<E>: DoubleEndedIterator));
  }

  #[test]
  fn ExactSizeIterator() {
    assert!(impls!(Drain<E>: ExactSizeIterator));
  }

  #[test]
  fn FusedIterator() {
    assert!(impls!(Drain<E>: FusedIterator));
  }

  #[test]
  fn Iterator() {
    assert!(impls!(Drain<E>: Iterator<Item = E>));
  }

  #[test]
  fn Send() {
    assert!(impls!(Drain<HasSend>: Send));

    assert!(impls!(Drain<E>: !Send));
  }

  #[test]
  fn Sync() {
    assert!(impls!(Drain<HasSync>: Sync));

    assert!(impls!(Drain<E>: !Sync));
  }

  #[test]
  fn Unpin() {
    assert!(impls!(Drain<E>: Unpin));
  }
}

mod Splice {
  use {
    crate::helpers::*, ::core::fmt::Debug, ::impls::impls, ::slimvec::Splice,
  };

  #[test]
  fn Debug() {
    assert!(impls!(Splice<ItDebug<HasDebug>>: Debug));

    assert!(impls!(Splice<It<E>>: !Debug));
    assert!(impls!(Splice<ItDebug<E>>: !Debug));
    assert!(impls!(Splice<It<HasDebug>>: !Debug));
  }

  #[test]
  fn DoubleEndedIterator() {
    assert!(impls!(Splice<It<E>>: DoubleEndedIterator));
  }

  #[test]
  fn ExactSizeIterator() {
    assert!(impls!(Splice<It<E>>: ExactSizeIterator));
  }

  #[test]
  fn Iterator() {
    assert!(impls!(Splice<It<E>>: Iterator<Item = E>));
  }

  #[test]
  fn Send() {
    assert!(impls!(Splice<ItSend<HasSend>>: Send));

    assert!(impls!(Splice<It<E>>: !Send));
    assert!(impls!(Splice<ItSend<E>>: !Send));
    assert!(impls!(Splice<It<HasSend>>: !Send));
  }

  #[test]
  fn Sync() {
    assert!(impls!(Splice<ItSync<HasSync>>: Sync));

    assert!(impls!(Splice<It<E>>: !Sync));
    assert!(impls!(Splice<ItSync<E>>: !Sync));
    assert!(impls!(Splice<It<HasSync>>: !Sync));
  }

  #[test]
  fn Unpin() {
    assert!(impls!(Splice<ItUnpin<E>>: Unpin));

    assert!(impls!(Splice<It<E>>: !Unpin));
  }
}

mod ExtractIf {
  use {
    crate::helpers::*, ::core::fmt::Debug, ::impls::impls,
    ::slimvec::ExtractIf,
  };

  #[test]
  fn Debug() {
    assert!(impls!(ExtractIf<HasDebug, E>: Debug));

    assert!(impls!(ExtractIf<E, E>: !Debug));
  }

  // TODO: express a type corresponding to FnMut
  // #[test]
  // fn Iterator() {
  //   assert!(impls!(ExtractIf<E, F>: Iterator<Item = E>));
  // }

  #[test]
  fn Send() {
    assert!(impls!(ExtractIf<HasSend, HasSend>: Send));

    assert!(impls!(ExtractIf<E, E>: !Send));
    assert!(impls!(ExtractIf<E, HasSend>: !Send));
    assert!(impls!(ExtractIf<HasSend, E>: !Send));
  }

  #[test]
  fn Sync() {
    assert!(impls!(ExtractIf<HasSync, HasSync>: Sync));

    assert!(impls!(ExtractIf<E, E>: !Sync));
    assert!(impls!(ExtractIf<E, HasSync>: !Sync));
    assert!(impls!(ExtractIf<HasSync, E>: !Sync));
  }

  #[test]
  fn Unpin() {
    assert!(impls!(ExtractIf<E, HasUnpin>: Unpin));

    assert!(impls!(ExtractIf<E, E>: !Unpin));
    assert!(impls!(ExtractIf<HasUnpin, E>: !Unpin));
  }
}

// Helper types
// ------------

mod helpers {
  use {
    ::core::{
      cell::UnsafeCell, fmt, fmt::Debug, hash::Hash, marker::PhantomPinned,
    },
    ::impls::impls,
  };

  #[expect(unused, reason = "Fields are included to disable auto traits")]
  pub struct E(*mut (), PhantomPinned);
  const _: () = {
    // Sized is always required for element types
    assert!(impls!(E: Sized));

    assert!(impls!(E: !Copy));
    assert!(impls!(E: !Send));
    assert!(impls!(E: !Sync));
    assert!(impls!(E: !Unpin));
    assert!(impls!(E: !Clone));
    assert!(impls!(E: !Debug));
    assert!(impls!(E: !Default));
    assert!(impls!(E: !Eq));
    assert!(impls!(E: !Ord));
    assert!(impls!(E: !PartialEq));
    assert!(impls!(E: !PartialOrd));
  };

  #[expect(unused)]
  pub struct HasSend(*mut ());
  unsafe impl Send for HasSend {}
  const _: () = assert!(impls!(HasSend: Send & !Sync));

  #[expect(unused)]
  pub struct HasSync(*mut ());
  unsafe impl Sync for HasSync {}
  const _: () = assert!(impls!(HasSync: Sync & !Send));

  #[derive(Copy, Clone)]
  pub struct HasCopy;
  const _: () = assert!(impls!(HasCopy: Copy));

  pub struct HasUnpin(PhantomPinned);
  impl Unpin for HasUnpin {}
  const _: () = assert!(impls!(HasUnpin: Unpin));

  #[derive(Clone)]
  pub struct HasClone;
  const _: () = assert!(impls!(HasClone: Clone & !Copy));

  #[derive(Debug)]
  pub struct HasDebug;
  const _: () = assert!(impls!(HasDebug: Debug));

  #[derive(Hash)]
  pub struct HasHash;
  const _: () = assert!(impls!(HasHash: Hash));

  #[derive(Eq, PartialEq)]
  pub struct HasEq;
  const _: () = assert!(impls!(HasEq: Eq));

  #[derive(Ord, PartialOrd, Eq, PartialEq)]
  pub struct HasOrd;
  const _: () = assert!(impls!(HasOrd: Ord));

  #[derive(PartialEq)]
  pub struct HasPartialEq;
  const _: () = {
    assert!(impls!(HasPartialEq: PartialEq<HasPartialEq> & !Eq));
    assert!(impls!(HasPartialEq: !PartialEq<E>));
  };

  #[derive(PartialOrd, PartialEq)]
  pub struct HasPartialOrd;
  const _: () = assert!(impls!(HasPartialOrd: PartialOrd & !Ord));

  #[expect(unused)]
  pub struct It<T>(T, &'static mut (), *mut (), UnsafeCell<()>, PhantomPinned);
  impl<T> Iterator for It<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
      todo!()
    }
  }
  const _: () = {
    assert!(impls!(It<E>: Iterator & !Send & !Sync & !Unpin & !Debug));
    assert!(impls!(It<HasSend>: Iterator & !Send & !Sync & !Unpin & !Debug));
    assert!(impls!(It<HasSync>: Iterator & !Send & !Sync & !Unpin & !Debug));
  };

  pub struct ItDebug<T>(T);
  impl<T> Iterator for ItDebug<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
      todo!()
    }
  }
  impl<T> Debug for ItDebug<T> {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
      todo!()
    }
  }
  const _: () = assert!(impls!(ItDebug<E>: Iterator & Debug));

  #[expect(unused)]
  pub struct ItSend<T>(T, *mut (), PhantomPinned);
  impl<T> Iterator for ItSend<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
      todo!()
    }
  }
  unsafe impl<T> Send for ItSend<T> {}
  const _: () = assert!(impls!(ItSend<E>: Iterator & Send & !Sync & !Unpin));

  #[expect(unused)]
  pub struct ItSync<T>(T, *mut (), PhantomPinned);
  impl<T> Iterator for ItSync<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
      todo!()
    }
  }
  unsafe impl<T> Sync for ItSync<T> {}
  const _: () = assert!(impls!(ItSync<E>: Iterator & Sync & !Send & !Unpin));

  #[expect(unused)]
  pub struct ItUnpin<T>(T, *mut (), PhantomPinned);
  impl<T> Iterator for ItUnpin<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
      todo!()
    }
  }
  impl<T> Unpin for ItUnpin<T> {}
  const _: () = assert!(impls!(ItUnpin<E>: Iterator & Unpin & !Send & !Sync));
}
