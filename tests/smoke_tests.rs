// © ickk 2026.

use ::slimvec::{SliceExt, SlimVec, slimvec};

#[test]
fn static_asserts() {
  // monomorphising causes static asserts to be checked in the `EMPTY` &
  // `SENTINEL_UNALLOCATED` constants.
  // using a type with a large alignment to verify alignment assertions.
  SlimVec::<::core::arch::x86_64::__m512>::new();
}

#[test]
fn slimvec_macro() {
  let slim: SlimVec<usize> = slimvec![];
  assert_eq!(slim.len(), 0);
  assert_eq!(slim.capacity(), 0);

  let slim = slimvec![1; 0];
  assert_eq!(slim.len(), 0);
  assert_eq!(slim.capacity(), 0);

  let slim = slimvec![1; 10];
  assert_eq!(slim.len(), 10);
  assert_eq!(slim.capacity(), 10);
  assert_eq!(slim.as_slice(), &[1; 10]);

  let slim = slimvec![1, 2, 3, 4, 5];
  assert_eq!(slim.len(), 5);
  assert_eq!(slim.capacity(), 5);
  assert_eq!(slim.as_slice(), &[1, 2, 3, 4, 5]);
}

#[test]
fn push_pop() {
  let mut slim = SlimVec::<u8>::new();
  slim.push(6u8);
  slim.push(7u8);
  let v = slim.pop();
  assert_eq!(v, Some(7u8));
  let v = slim.pop();
  assert_eq!(v, Some(6u8));
}

#[test]
fn drop_zst() {
  use ::core::sync::atomic::{AtomicUsize, Ordering};
  static COUNT: AtomicUsize = AtomicUsize::new(0);
  struct DropCounter;
  impl Drop for DropCounter {
    fn drop(&mut self) {
      COUNT.fetch_add(1, Ordering::Release);
    }
  }
  let mut slim = SlimVec::new();
  slim.push(DropCounter);
  assert_eq!(COUNT.load(Ordering::Acquire), 0);
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 1);
  slim.push(DropCounter);
  slim.push(DropCounter);
  slim.push(DropCounter);
  assert_eq!(COUNT.load(Ordering::Acquire), 1);
  _ = slim.pop();
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 3);
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 4);
  _ = slim.pop();
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 4);
  slim.push(DropCounter);
  slim.push(DropCounter);
  slim.push(DropCounter);
  assert_eq!(COUNT.load(Ordering::Acquire), 4);
  slim.clear();
  assert_eq!(COUNT.load(Ordering::Acquire), 7);
}

#[test]
fn drop() {
  use ::core::sync::atomic::{AtomicUsize, Ordering};
  static COUNT: AtomicUsize = AtomicUsize::new(0);
  #[derive(Default)]
  struct DropCounter {
    _s: String,
  }
  impl Drop for DropCounter {
    fn drop(&mut self) {
      COUNT.fetch_add(1, Ordering::Release);
    }
  }
  let mut slim = SlimVec::new();
  slim.push(DropCounter::default());
  assert_eq!(COUNT.load(Ordering::Acquire), 0);
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 1);
  slim.push(DropCounter::default());
  slim.push(DropCounter::default());
  slim.push(DropCounter::default());
  assert_eq!(COUNT.load(Ordering::Acquire), 1);
  _ = slim.pop();
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 3);
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 4);
  _ = slim.pop();
  _ = slim.pop();
  assert_eq!(COUNT.load(Ordering::Acquire), 4);
  slim.push(DropCounter::default());
  slim.push(DropCounter::default());
  slim.push(DropCounter::default());
  assert_eq!(COUNT.load(Ordering::Acquire), 4);
  slim.clear();
  assert_eq!(COUNT.load(Ordering::Acquire), 7);
}

#[test]
fn clone() {
  let mut slim = SlimVec::new();
  slim.push(['a', 'b', 'c']);
  slim.push(['d', 'e', 'f']);
  slim.push(['g', 'h', 'i']);
  slim.push(['j', 'k', 'l']);

  let mut clone = slim.clone();

  let last = slim.pop();
  assert_eq!(last, Some(['j', 'k', 'l']));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, Some(['g', 'h', 'i']));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, Some(['d', 'e', 'f']));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, Some(['a', 'b', 'c']));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, None);
  assert_eq!(last, clone.pop());
}

#[test]
fn clone_zst() {
  #[derive(Clone, PartialEq, Debug)]
  struct ZST();

  let mut slim = SlimVec::new();
  slim.push(ZST());
  slim.push(ZST());
  slim.push(ZST());
  slim.push(ZST());

  let mut clone = slim.clone();

  let last = slim.pop();
  assert_eq!(last, Some(ZST()));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, Some(ZST()));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, Some(ZST()));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, Some(ZST()));
  assert_eq!(last, clone.pop());

  let last = slim.pop();
  assert_eq!(last, None);
  assert_eq!(last, clone.pop());
}

#[test]
fn truncate() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  assert_eq!(slim.len(), 4);
  slim.truncate(2);
  assert_eq!(slim.len(), 2);
  assert_eq!(slim.as_slice(), &[['a', 'b', 'c'], ['d', 'e', 'f']]);
}

#[test]
fn truncate_zst() {
  #[derive(PartialEq, Debug)]
  struct Zst();
  let mut slim = SlimVec::from([Zst(), Zst(), Zst(), Zst()]);
  assert_eq!(slim.len(), 4);
  slim.truncate(2);
  assert_eq!(slim.len(), 2);
  assert_eq!(slim.as_slice(), &[Zst(), Zst()]);
}

#[test]
fn get() {
  let slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  assert_eq!(slim.get(0), Some(&['a', 'b', 'c']));
  assert_eq!(slim.get(3), Some(&['j', 'k', 'l']));
  assert_eq!(slim.get(4), None);
}

#[test]
fn get_zst() {
  #[derive(PartialEq, Debug)]
  struct Zst();
  let slim = SlimVec::from([Zst(), Zst(), Zst(), Zst()]);
  assert_eq!(slim.get(0), Some(&Zst()));
  assert_eq!(slim.get(3), Some(&Zst()));
  assert_eq!(slim.get(4), None);
}

#[test]
fn get_mut() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  assert_eq!(slim.get(0), Some(&['a', 'b', 'c']));
  *slim.get_mut(0).unwrap() = ['z', 'z', 'z'];
  assert_eq!(slim.get(0), Some(&['z', 'z', 'z']));

  assert_eq!(slim.get(3), Some(&['j', 'k', 'l']));
  *slim.get_mut(3).unwrap() = ['1', '2', '3'];
  assert_eq!(slim.get(3), Some(&['1', '2', '3']));

  assert_eq!(slim.get(4), None);
}

#[test]
fn get_mut_zst() {
  #[derive(PartialEq, Debug)]
  struct Zst();
  let mut slim = SlimVec::from([Zst(), Zst(), Zst(), Zst()]);
  assert_eq!(slim.get_mut(0), Some(&mut Zst()));
  assert_eq!(slim.get_mut(3), Some(&mut Zst()));
  assert_eq!(slim.get_mut(4), None);
}

#[test]
fn insert() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  assert_eq!(slim.len(), 4);
  assert_eq!(slim.get(2), Some(&['g', 'h', 'i']));
  slim.insert(2, ['z', 'z', 'z']);
  assert_eq!(slim.len(), 5);
  assert_eq!(slim.get(2), Some(&['z', 'z', 'z']));
  assert_eq!(slim.get(3), Some(&['g', 'h', 'i']));
}

#[test]
fn insert_zst() {
  #[derive(PartialEq, Debug)]
  struct Zst();
  let mut slim = SlimVec::from([Zst(), Zst(), Zst(), Zst()]);
  assert_eq!(slim.len(), 4);
  slim.insert(2, Zst());
  assert_eq!(slim.len(), 5);
}

#[test]
fn remove() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  assert_eq!(slim.len(), 4);
  assert_eq!(slim.get(2), Some(&['g', 'h', 'i']));
  let removed = slim.remove(2);
  assert_eq!(slim.len(), 3);
  assert_eq!(removed, ['g', 'h', 'i']);
  assert_eq!(slim.get(2), Some(&['j', 'k', 'l']));
}

#[test]
fn remove_zst() {
  #[derive(PartialEq, Debug)]
  struct Zst();
  let mut slim = SlimVec::from([Zst(), Zst(), Zst(), Zst()]);
  assert_eq!(slim.len(), 4);
  let removed = slim.remove(2);
  assert_eq!(slim.len(), 3);
  assert_eq!(removed, Zst());
}

#[test]
fn append() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  let mut other = SlimVec::from([['z', 'z', 'z'], ['y', 'y', 'y']]);
  assert_eq!(slim.len(), 4);
  assert_eq!(other.len(), 2);
  slim.append(&mut other);
  assert_eq!(slim.len(), 6);
  assert_eq!(other.len(), 0);
  assert_eq!(
    slim.as_slice(),
    &[
      ['a', 'b', 'c'],
      ['d', 'e', 'f'],
      ['g', 'h', 'i'],
      ['j', 'k', 'l'],
      ['z', 'z', 'z'],
      ['y', 'y', 'y']
    ]
  )
}

#[test]
fn retain() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);

  assert_eq!(slim.len(), 4);
  slim.retain(|c| c[0] == 'a' || c[0] == 'g');
  assert_eq!(slim.len(), 2);
  assert_eq!(slim.as_slice(), &[['a', 'b', 'c'], ['g', 'h', 'i']])
}

#[test]
fn retain_mut() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);

  assert_eq!(slim.len(), 4);
  slim.retain_mut(|c| {
    if c[0] == 'a' || c[0] == 'd' {
      c[1] = 'z';
    }
    c[0] == 'a' || c[0] == 'd' || c[0] == 'j'
  });
  assert_eq!(slim.len(), 3);
  assert_eq!(
    slim.as_slice(),
    &[['a', 'z', 'c'], ['d', 'z', 'f'], ['j', 'k', 'l']]
  )
}

#[test]
fn dedup() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['g', 'h', 'i'],
    ['a', 'b', 'c'],
    ['g', 'h', 'i'],
    ['g', 'h', 'i'],
    ['d', 'e', 'f'],
    ['j', 'k', 'l'],
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
  ]);

  assert_eq!(slim.len(), 16);
  slim.dedup();
  assert_eq!(slim.len(), 8);
  assert_eq!(
    slim.as_slice(),
    &[
      ['a', 'b', 'c'],
      ['d', 'e', 'f'],
      ['g', 'h', 'i'],
      ['a', 'b', 'c'],
      ['g', 'h', 'i'],
      ['d', 'e', 'f'],
      ['j', 'k', 'l'],
      ['a', 'b', 'c'],
    ]
  )
}

#[test]
fn dedup_by() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
    ['a', 'b', 'c'],
    ['d', 'e', 'd'],
    ['d', 'e', 'c'],
    ['g', 'h', 'c'],
    ['g', 'h', 'c'],
    ['a', 'b', 'c'],
    ['g', 'h', 'i'],
    ['g', 'h', 'i'],
    ['d', 'e', 'f'],
    ['j', 'k', 'f'],
    ['a', 'b', 'f'],
    ['a', 'b', 'd'],
    ['a', 'b', 'd'],
  ]);

  assert_eq!(slim.len(), 16);
  slim.dedup_by(|a, b| a[2] == b[2]);
  assert_eq!(slim.len(), 6);
  assert_eq!(
    slim.as_slice(),
    &[
      ['a', 'b', 'c'],
      ['d', 'e', 'd'],
      ['d', 'e', 'c'],
      ['g', 'h', 'i'],
      ['d', 'e', 'f'],
      ['a', 'b', 'd'],
    ]
  )
}

#[test]
fn into_flattened() {
  let slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  let flattened: SlimVec<char> = slim.into_flattened();
  assert_eq!(
    flattened.as_slice(),
    &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',],
  );
}

#[test]
fn into_flattened_zst() {
  #[derive(PartialEq, Debug, Copy, Clone)]
  struct Zst();
  let slim = SlimVec::from([[Zst(); 3], [Zst(); 3], [Zst(); 3], [Zst(); 3]]);
  let flattened: SlimVec<Zst> = slim.into_flattened();
  assert_eq!(flattened.as_slice(), &[Zst(); 12]);
}

#[test]
fn from_vec() {
  let vec = vec![
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ];
  let slim = SlimVec::from(vec.clone());
  assert_eq!(slim.as_slice(), vec.as_slice());
}

#[test]
fn from_boxed_slice() {
  let boxed_slice = vec![
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]
  .into_boxed_slice();
  let slim = SlimVec::from(boxed_slice);
  assert_eq!(
    slim.as_slice(),
    &[
      ['a', 'b', 'c'],
      ['d', 'e', 'f'],
      ['g', 'h', 'i'],
      ['j', 'k', 'l'],
    ]
  );
}

#[test]
fn from_array() {
  let array: [[char; 3]; 4] = [
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ];
  let slim: SlimVec<[char; 3]> = SlimVec::from(array);
  assert_eq!(slim.as_slice(), &array);
}

#[test]
fn extend() {
  let mut slim = SlimVec::new();
  assert_eq!(slim.as_slice(), &[]);
  slim.extend(['a', 'b', 'c']);
  assert_eq!(slim.as_slice(), &['a', 'b', 'c']);
  slim.extend(['d', 'e', 'f']);
  assert_eq!(slim.as_slice(), &['a', 'b', 'c', 'd', 'e', 'f']);
}

#[test]
fn extend_from_slice() {
  let mut slim = SlimVec::from(['a', 'b', 'c']);
  let s: &[char] = &['d', 'e', 'f'];
  slim.extend_from_slice(s);
  assert_eq!(slim.as_slice(), &['a', 'b', 'c', 'd', 'e', 'f']);
}

#[test]
fn extend_from_within() {
  let mut slim = SlimVec::from(['a', 'b', 'c', 'd', 'e']);
  slim.extend_from_within(2..);
  assert_eq!(slim.as_slice(), &['a', 'b', 'c', 'd', 'e', 'c', 'd', 'e']);
}

#[test]
fn swap_remove() {
  let mut slim = SlimVec::from([
    ['a', 'b', 'c'],
    ['d', 'e', 'f'],
    ['g', 'h', 'i'],
    ['j', 'k', 'l'],
  ]);
  assert_eq!(slim[1], ['d', 'e', 'f']);
  assert_eq!(slim.swap_remove(1), ['d', 'e', 'f']);
  assert_eq!(slim[1], ['j', 'k', 'l']);
}

#[test]
fn swap_remove_zst() {
  #[derive(PartialEq, Debug)]
  struct Zst();
  let mut slim = SlimVec::from([Zst(), Zst(), Zst(), Zst()]);
  dbg!(&slim);
  assert_eq!(slim.len(), 4);
  assert_eq!(slim[1], Zst());
  assert_eq!(slim.swap_remove(1), Zst());
  assert_eq!(slim.len(), 3);
  assert_eq!(slim[1], Zst());
}

#[test]
fn into_iter() {
  let mut into_iter: ::slimvec::IntoIter<char> =
    SlimVec::from(['a', 'b', 'c', 'd', 'e']).into_iter();
  assert_eq!(into_iter.next(), Some('a'));
  assert_eq!(into_iter.next_back(), Some('e'));
  assert_eq!(into_iter.next(), Some('b'));
  assert_eq!(into_iter.next_back(), Some('d'));
  assert_eq!(into_iter.next_back(), Some('c'));
  assert_eq!(into_iter.next(), None);
  assert_eq!(into_iter.next_back(), None);
}

#[test]
fn drain() {
  let mut slimvec = SlimVec::from(['0', '1', '2', '3', '4', '5', '6', '7']);
  {
    let mut drain = slimvec.drain(2..=5);
    assert_eq!(drain.next(), Some('2'));
    assert_eq!(drain.next_back(), Some('5'));
    assert_eq!(drain.next(), Some('3'));
  }
  assert_eq!(&slimvec[..], &['0', '1', '6', '7'])
}

#[test]
fn extract_if() {
  let mut slimvec: SlimVec<usize> =
    SlimVec::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
  let muls_three = slimvec
    .extract_if(2..=7, |v| v.is_multiple_of(3))
    .collect::<Vec<_>>();
  assert_eq!(muls_three, [3, 6]);
  assert_eq!(&slimvec[..], &[0, 1, 2, 4, 5, 7, 8, 9]);
}

#[test]
fn splice() {
  let mut slimvec: SlimVec<isize> =
    SlimVec::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
  let removed = slimvec.splice(2..4, (-10..=-7).rev()).collect::<Vec<_>>();
  assert_eq!(removed, [2, 3]);
  assert_eq!(&slimvec[..], &[0, 1, -7, -8, -9, -10, 4, 5, 6, 7, 8, 9]);
}

#[rustfmt::skip]
#[test]
fn slice_repeat() {
  let s: &[usize] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
  let slimvec = s.repeat_to_slimvec(20);
  assert_eq!(
    slimvec,
    [
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,

      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,

      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,

      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    ]
  )
}
