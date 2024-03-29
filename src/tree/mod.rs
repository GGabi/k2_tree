
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

/* Private Yet Common to Everything Module */
use bitvec::prelude::BitVec;

impl K2Tree {
  // Internal: Never call with layer >= self.max_slayers
  fn layer_len(&self, l: usize) -> usize {
    if l == self.max_slayers-1 {
      return self.stems.len() - self.layer_start(l)
    }
    let layer_starts = self.layer_starts();
    layer_starts[l+1] - layer_starts[l]
  }
  fn get_coords(&self, leaf_bit_pos: usize) -> [usize; 2] {
    /* Start at the leaf_bit and traverse our way up to the top of the tree,
    keeping track of the path we took on our way up in terms of
    bit-positions (offsets) in the stems. Then, traverse back down the same
    path to find the coords of the leaf_bit. */
    let parent_bit = self.leaf_parent(leaf_bit_pos);
    let mut stem_start = self.stem_start(parent_bit);
    let mut offset = parent_bit - stem_start;
    let mut offsets = vec![offset];
    for _ in 1..self.max_slayers {
      let parent = self.parent(stem_start).unwrap();
      stem_start = parent[0];
      offset = parent[1];
      offsets.push(offset);
    }
    /* Reverse the offsets ready to traverse them back down the tree */
    offsets.reverse();
    let range_max = self.matrix_width()-1;
    let mut range = Range2D::new(0, range_max, 0, range_max);
    for child_offset in offsets.into_iter().take(self.max_slayers) {
      range = self.to_subranges(range).unwrap()[child_offset];
    }
    let leaf_offset = leaf_bit_pos - self.leaf_start(leaf_bit_pos);
    let x = leaf_offset % self.leaf_k;
    let y = leaf_offset / self.leaf_k;
    [range.min_x + x, range.min_y + y]
  }
  fn leaf_parent(&self, bit_pos: usize) -> usize {
    let nth_leaf = bit_pos / self.leaf_len();
    let final_stem_layer_start = self.layer_start(self.max_slayers-1);
    let stem_ones_positions = one_positions_range(
      &self.stems,
      final_stem_layer_start,
      self.stems.len()
    );
    final_stem_layer_start + stem_ones_positions[nth_leaf] //TODO: check
  }
  fn parent(&self, stem_start: usize) -> std::result::Result<[usize; 2], ()> {
    /* Returns [stem_start, bit_offset] */
    let stem_len = self.stem_len();
    if stem_start < stem_len {
      return Err(()) //First stem cannot have parent
    }
    /* Find which layer stem_start is in */
    let layer_starts = self.layer_starts();
    let stem_layer = {
      let mut layer = self.max_slayers-1; //If no match, must be in highest layer
      for (i, &layer_start) in layer_starts.iter().enumerate() {
        if stem_start < layer_start { layer = i-1; break }
      }
      layer
    };
    /* Find the nth stem it is in the layer */
    let stem_num = (stem_start - layer_starts[stem_layer]) / stem_len;
    /* Find the nth 1 in the parent layer */
    let parent_bit_relative_pos = one_positions_range(
      &self.stems,
      layer_starts[stem_layer-1],
      layer_starts[stem_layer]
    )[stem_num];
    let parent_bit_absolute_pos = parent_bit_relative_pos + layer_starts[stem_layer-1];
    Ok([self.stem_start(parent_bit_absolute_pos), parent_bit_absolute_pos % stem_len])
  }
  fn layer_start(&self, l: usize) -> usize {
    //Private method, let it crash seeing as is just unwrapped otherwise
    let mut curr_layer = 1;
    let mut layer_starts = vec![0, self.stem_len()];
    while curr_layer < l {
      let stems_in_curr_layer = ones_in_range(
        &self.stems,
        layer_starts[curr_layer-1],
        layer_starts[curr_layer]
      );
      let curr_layer_len = stems_in_curr_layer * self.stem_len();
      layer_starts.push(layer_starts[curr_layer] + curr_layer_len);
      curr_layer += 1;
    }
    layer_starts[l]
  }
  fn layer_starts(&self) -> Vec<usize> {
    //Private method, let it crash seeing as is just unwrapped otherwise
    let mut curr_layer = 1;
    let mut layer_starts = vec![0, self.stem_len()];
    while curr_layer < self.max_slayers-1 {
      let stems_in_curr_layer = ones_in_range(
        &self.stems,
        layer_starts[curr_layer-1],
        layer_starts[curr_layer]
      );
      let curr_layer_len = stems_in_curr_layer * self.stem_len();
      layer_starts.push(layer_starts[curr_layer] + curr_layer_len);
      curr_layer += 1;
    }
    layer_starts
  }
}

/* Block Utils */
impl K2Tree {
  fn stem_len(&self) -> usize {
    self.stem_k.pow(2)
  }
  fn leaf_len(&self) -> usize {
    self.leaf_k.pow(2)
  }
  fn stem_start(&self, bit_pos: usize) -> usize {
    (bit_pos / self.stem_len()) * self.stem_len()
  }
  fn leaf_start(&self, bit_pos: usize) -> usize {
    (bit_pos / self.leaf_len()) * self.leaf_len()
  }
  fn to_subranges(&self, r: Range2D) -> std::result::Result<SubRanges, crate::error::SubRangesError> {
    SubRanges::from_range(r, self.stem_k, self.stem_k)
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
fn one_positions_bv(bits: &BitVec) -> Vec<usize> {
  bits.iter()
  .enumerate()
  .filter_map(
    |(pos, &bit)|
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
  #[allow(dead_code)]
  pub width: usize,
  /// Number of vertical subranges.
  #[allow(dead_code)]
  pub height: usize,
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

/* Only used in tests */
#[cfg(test)]
use {
  bitvec::prelude::bitvec,
  crate::matrix::BitMatrix
};

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

/* Private funcs used in testing */
#[cfg(test)]
impl K2Tree {
  fn test_tree(k: usize) -> Self {
    match k {
      2 => K2Tree {
        stem_k: 2,
        leaf_k: 2,
        max_slayers: 2,
        stems:  bitvec![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0],
        leaves: bitvec![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0],
      },
      3 => K2Tree {
        stem_k: 3,
        leaf_k: 3,
        max_slayers: 2,
        stems:  bitvec![
          0,1,0,1,1,0,1,1,0, 1,1,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0,
          1,0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0
        ],
        leaves: bitvec![
          0,1,0,1,0,0,0,0,1, 1,0,0,1,0,0,1,0,0, 1,0,0,0,0,0,0,0,0,
          0,1,0,1,0,0,0,0,0, 1,0,0,0,0,0,0,0,0, 0,1,0,1,0,0,0,0,0,
        ]
      },
      4 => K2Tree {
        stem_k: 4,
        leaf_k: 4,
        max_slayers: 2,
        stems: bitvec![
          1,0,0,1,0,0,0,1,1,0,0,0,1,1,0,1, 1,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,
          0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,
          0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0, 0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,
          1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,
        ],
        leaves: bitvec![
          1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,
          0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0, 0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,
          0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0, 0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,
          0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
        ],
      },
      _ => K2Tree::with_k(2, 2).unwrap(),
    }
  }
  fn test_matrix(k: usize) -> BitMatrix {
    match k {
      2 => {
        let bits = bitvec![
          0,0,0,0, 0,1,0,1,
          0,0,0,0, 1,0,0,1,
          0,0,0,0, 0,0,1,1,
          0,0,0,0, 0,0,0,0,

          1,0,0,0, 0,1,0,0,
          0,0,0,0, 1,0,0,0,
          0,0,0,0, 0,0,0,0,
          0,0,0,0, 0,0,0,0
        ];
        BitMatrix::from_bits(8, 8, bits)
      },
      3 => {
        let bits = bitvec![
          0,0,0, 0,0,0, 0,0,0,  0,1,0, 1,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  1,0,0, 1,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,1, 1,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,

          1,0,0, 0,0,0, 0,0,0,  0,1,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  1,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          
          1,0,0, 0,0,0, 0,0,0,  0,1,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  1,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
          0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,  0,0,0, 0,0,0, 0,0,0,
        ];
        BitMatrix::from_bits(27, 27, bits)
      },
      _ => BitMatrix::new(),
    }
  }
}

#[cfg(test)]
mod K2Tree_common_util {
  use super::*;
  #[test]
  fn layer_len() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(4, tree.layer_len(0));
    assert_eq!(12, tree.layer_len(1));
  }
  #[test]
  fn get_coords() {
    let expected_coords = [
      [4, 0], [5, 0], [4, 1], [5, 1], // Leaf 1
      [6, 0], [7, 0], [6, 1], [7, 1], // Leaf 2
      [6, 2], [7, 2], [6, 3], [7, 3], // Leaf 3
      [0, 4], [1, 4], [0, 5], [1, 5], // Leaf 4
      [4, 4], [5, 4], [4, 5], [5, 5], // Leaf 5
    ];
    let tree = K2Tree::test_tree(2);
    for (leaf_offset, _) in tree.leaves.iter().enumerate() {
      assert_eq!(
        expected_coords[leaf_offset],
        tree.get_coords(leaf_offset)
      );
    }
  }
}
