
mod datastore;
mod iterators;

pub use datastore::*;
pub use datastore::K2Tree;
pub use iterators::{
  StemBit,
  LeafBit,
  Stems,
  StemsRaw,
  Leaves,
  IntoLeaves,
  LeavesRaw,
};

/* Common */
use bitvec::vec::BitVec;

impl K2Tree {
  fn layer_len(&self, l: usize) -> usize {
    if l == self.slayer_starts.len()-1 {
      return self.stems.len() - self.slayer_starts[l]
    }
    self.slayer_starts[l+1] - self.slayer_starts[l]
  }
  fn get_coords(&self, leaf_bit_pos: usize) -> [usize; 2] { //TODO: Verify
    /* Start at the leaf_bit and traverse our way up to the top of the tree,
    keeping track of the path we took on our way up in terms of
    bit-positions (offsets) in the stems. Then, traverse back down the same
    path to find the coords of the leaf_bit. */
    let parent_bit = self.leaf_parent(leaf_bit_pos);
    let mut stem_start = self.block_start(parent_bit);
    let mut offset = parent_bit - stem_start;
    let mut offsets = Vec::new();
    offsets.push(offset);
    for _ in 1..self.max_slayers {
      let parent = self.parent(stem_start).unwrap();
      stem_start = parent[0];
      offset = parent[1];
      offsets.push(offset);
    }
    /* Reverse the offsets ready to traverse them back down the tree */
    offsets.reverse();
    let mut range = Range2D::new(0, self.matrix_width-1, 0, self.matrix_width-1);
    // let mut range = [[0, self.matrix_width-1], [0, self.matrix_width-1]];
    for child_offset in offsets.into_iter().take(self.max_slayers) {
      range = self.to_subranges(range).unwrap()[child_offset];
      // range = to_4_subranges(range)[child_offset];
    }
    let leaf_offset = leaf_bit_pos - self.block_start(leaf_bit_pos);
    let x = leaf_offset % self.k;
    let y = leaf_offset / self.k;
    [range.min_x + x, range.min_y + y] //TODO: Verify
    // match leaf_offset {
    //   0 => [range[0][0], range[1][0]],
    //   1 => [range[0][1], range[1][0]],
    //   2 => [range[0][0], range[1][1]],
    //   3 => [range[0][1], range[1][1]],
    //   _ => unreachable!(),
    // }
  }
  fn leaf_parent(&self, bit_pos: usize) -> usize {
    self.layer_start(self.max_slayers-1) + self.stem_to_leaf[bit_pos / self.block_len()]
  }
  fn parent(&self, stem_start: usize) -> std::result::Result<[usize; 2], ()> { //TODO
    /* Returns (stem_start, bit_offset) */
    if stem_start < self.slayer_starts[1] {
      return Err(())
    }
    let block_len = self.block_len();
    /* Find which layer stem_start is in */
    let stem_layer = {
      let mut layer = self.max_slayers-1; //If no match, must be in highest layer
      for (i, &layer_start) in self.slayer_starts.iter().enumerate() {
        if stem_start <= layer_start { layer = i; break }
      }
      layer
    };
    /* Find the nth stem it is in the layer */
    let stem_num = (stem_start - stem_layer) / block_len;
    /* Find the nth 1 in the parent layer */
    let parent_bit = one_positions_range(
      &self.stems,
      self.slayer_starts[stem_layer-1],
      self.slayer_starts[stem_layer]
    )[stem_num];
    Ok([self.block_start(parent_bit), parent_bit % block_len])
  }
  fn layer_start(&self, l: usize) -> usize {
    if l == self.slayer_starts.len() {
      self.stems.len()
    }
    else {
      self.slayer_starts[l]
    }
  }
}

/* Block Utils */
impl K2Tree {
  fn block_len(&self) -> usize {
    self.k.pow(2)
  }
  fn block_start(&self, bit_pos: usize) -> usize {
    (bit_pos / self.block_len()) * self.block_len()
  }
  fn to_subranges(&self, r: Range2D) -> std::result::Result<SubRanges, crate::error::SubRangesError> {
    SubRanges::from_range(r, self.k, self.k)
  }
}

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

/* Tests */
#[cfg(tests)]
mod range_tests {
  use super::*;
  #[test]
  fn range2d_from_range() {
    let range = [[0, 7], [0, 7]];
    let expected = Range2D {
      min_x: 0,
      max_x: 7,
      min_y: 0,
      max_y: 7,
    };
    assert_eq!(expected, Range2D::from_range(range));
  }
  #[test]
  fn subranges_from_range2d_0() {
    let original = Range2D::new(0, 7, 0, 7);
    let expected_subs = [
      Range2D::new(0, 3, 0, 3),
      Range2D::new(4, 7, 0, 3),
      Range2D::new(0, 3, 4, 7),
      Range2D::new(4, 7, 4, 7),
    ];
    let subs = SubRange::from_range(original, 2, 2);
    for i in 0..4 { assert_eq!(expected_subs[i], subs[i]); }
  }
  #[test]
  fn subranges_from_range2d_1() {
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
    let subs = SubRange::from_range(original, 3, 3);
    for i in 0..9 { assert_eq!(expected_subs[i], subs[i]); }
  }
  #[test]
  fn subranges_from_range2d_2() {
    let original = Range2D::new(0, 8, 0, 7);
    let expected_subs = [
      Range2D::new(0, 2, 0, 3),
      Range2D::new(3, 5, 0, 3),
      Range2D::new(6, 8, 0, 3),
      Range2D::new(0, 2, 4, 7),
      Range2D::new(3, 5, 4, 7),
      Range2D::new(6, 8, 4, 7),
    ];
    let subs = SubRange::from_range(original, 3, 2);
    for i in 0..6 { assert_eq!(expected_subs[i], subs[i]); }
  }
}