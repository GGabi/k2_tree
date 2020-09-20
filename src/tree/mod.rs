
mod datastore;
// mod iterators;

// #[cfg(test)]
// mod api_tests;

pub use datastore::*;
pub use datastore::K2Tree;
// pub use iterators::{
//   StemBit,
//   LeafBit,
//   Stems,
//   StemsRaw,
//   Leaves,
//   IntoLeaves,
//   LeavesRaw,
// };

/* Common */
use bitvec::vec::BitVec;





fn remove_block(bit_vec: &mut BitVec, block_start: usize, block_len: usize) -> std::result::Result<(), ()> {
  if block_start > bit_vec.len()-block_len
  || block_start % block_len != 0 {
    Err(())
  }
  else {
    for _ in 0..block_len { bit_vec.remove(block_start); }
    Ok(())
  }
}
fn insert_block(bit_vec: &mut BitVec, block_start: usize, block_len: usize) -> std::result::Result<(), ()> {
  if block_start > bit_vec.len()
  || block_start % block_len != 0 {
    Err(())
  }
  else {
    for _ in 0..block_len { bit_vec.insert(block_start, false); }
    Ok(())
  }
}
fn ones_in_range(bits: &BitVec, begin: usize, end: usize) -> usize {
  bits[begin..end].into_iter().fold(0, |total, bit| total + *bit as usize)
}
fn all_zeroes(bits: &BitVec, begin: usize, end: usize) -> bool {
  bits[begin..end].into_iter().fold(true, |total, bit| total & !bit)
}
fn one_positions(bits: impl Iterator<Item=bool>) -> Vec<usize> {
  bits
  .enumerate()
  .filter_map(
    |(pos, bit)|
    if bit { Some(pos) }
    else   { None })
  .collect()
}
fn one_positions_range(bits: &BitVec, begin: usize, end: usize) -> Vec<usize> {
  bits[begin..end].into_iter().enumerate()
  .filter_map(|(pos, bit)|
    if *bit { Some(pos) }
    else    { None }
  ).collect()
}

/* Ranges */
#[derive(Debug, Clone)]
struct SubRanges {
  /// Number of horizontal subranges.
  width: usize,
  /// Number of vertical subranges.
  height: usize,
  /// Subranges
  subranges: Vec<Range2D>,
}
impl SubRanges {
  fn from_range(r: Range2D, w: usize, h: usize) -> std::result::Result<Self, crate::error::SubRangesError> {
    // If the range cannot be evenly divided up by w and h, break
    if r.width() / w * w != r.width()
    || r.height() / h * h != r.height() {
      return Err(crate::error::SubRangesError::CannotSubdivideRange {
        range: [[r.min_x, r.min_y], [r.max_x, r.max_y]],
        horizontal_subdivisions: w,
        vertical_subdivisions: h,
      })
    }
    let mut subranges: Vec<Range2D> = Vec::new();
    let sub_width = r.width() / w;
    let sub_height = r.height() / h;
    // Process subranges by rows then columns
    for y in 0..h {
      for x in 0..w {
        let min_x = r.min_x + (x * sub_width);
        let max_x = min_x + sub_width-1;
        let min_y = r.min_y + (y * sub_height);
        let max_y = min_y + sub_height-1;
        subranges.push(Range2D::new(min_x, max_x, min_y, max_y));
      }
    }
    Ok(SubRanges {
      width: w,
      height: h,
      subranges
    })
  }
  fn iter(&self) -> impl Iterator<Item=&Range2D> {
    self.subranges.iter()
  }
}
impl std::ops::Index<usize> for SubRanges {
  type Output = Range2D;
  fn index(&self, i: usize) -> &Self::Output {
    &self.subranges[i]
  }
}
impl std::ops::IndexMut<usize> for SubRanges {
  fn index_mut(&mut self, i: usize) -> &mut Self::Output {
    &mut self.subranges[i]
  }
}

#[derive(Debug, Clone, Copy)]
struct Range2D {
  pub min_x: usize,
  pub max_x: usize,
  pub min_y: usize,
  pub max_y: usize
}
impl Range2D {
  fn new(min_x: usize, max_x: usize, min_y: usize, max_y: usize) -> Self {
    Range2D {
      min_x,
      max_x,
      min_y,
      max_y
    }
  }
  fn width(&self) -> usize {
    self.max_x - self.min_x + 1 // +1 because range is inclusive
  }
  fn height(&self) -> usize {
    self.max_y - self.min_y + 1 // +1 because range is inclusive
  }
  fn contains(&self, x: usize, y: usize) -> bool {
    x >= self.min_x && x <= self.max_x
    && y >= self.min_y && y <= self.max_y
  }
}
impl PartialEq for Range2D {
  fn eq(&self, other: &Self) -> bool {
    self.min_x == other.min_x
    && self.max_x == other.max_x
    && self.min_y == other.min_y
    && self.max_y == other.max_y
  }
}
impl Eq for Range2D {}

/* Tests */
#[cfg(test)]
mod range_tests {
  use super::*;
  type Result<T> = std::result::Result<T, crate::error::SubRangesError>;
  #[test]
  fn subranges_from_range2d_0() -> Result<()> {
    let original = Range2D::new(0, 7, 0, 7);
    let expected_subs = [
      Range2D::new(0, 3, 0, 3),
      Range2D::new(4, 7, 0, 3),
      Range2D::new(0, 3, 4, 7),
      Range2D::new(4, 7, 4, 7),
    ];
    let subs = SubRanges::from_range(original, 2, 2)?;
    for i in 0..4 { assert_eq!(expected_subs[i], subs[i]); }
    Ok(())
  }
  #[test]
  fn subranges_from_range2d_1() -> Result<()> {
    let original = Range2D::new(0, 8, 0, 8);
    let expected_subs = [
      Range2D::new(0, 2, 0, 2),
      Range2D::new(3, 5, 0, 2),
      Range2D::new(6, 8, 0, 2),
      Range2D::new(0, 2, 3, 5),
      Range2D::new(3, 5, 3, 5),
      Range2D::new(6, 8, 3, 5),
      Range2D::new(0, 2, 6, 8),
      Range2D::new(3, 5, 6, 8),
      Range2D::new(6, 8, 6, 8),
    ];
    let subs = SubRanges::from_range(original, 3, 3)?;
    for i in 0..9 { assert_eq!(expected_subs[i], subs[i]); }
    Ok(())
  }
  #[test]
  fn subranges_from_range2d_2() -> Result<()> {
    let original = Range2D::new(0, 8, 0, 7);
    let expected_subs = [
      Range2D::new(0, 2, 0, 3),
      Range2D::new(3, 5, 0, 3),
      Range2D::new(6, 8, 0, 3),
      Range2D::new(0, 2, 4, 7),
      Range2D::new(3, 5, 4, 7),
      Range2D::new(6, 8, 4, 7),
    ];
    let subs = SubRanges::from_range(original, 3, 2)?;
    for i in 0..6 { assert_eq!(expected_subs[i], subs[i]); }
    Ok(())
  }
}