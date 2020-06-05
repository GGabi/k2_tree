
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
type Range = [[usize; 2]; 2];

impl K2Tree {
  fn layer_len(&self, l: usize) -> usize {
    if l == self.slayer_starts.len()-1 {
      return self.stems.len() - self.slayer_starts[l]
    }
    self.slayer_starts[l+1] - self.slayer_starts[l]
  }
  fn get_coords(&self, leaf_bit_pos: usize) -> [usize; 2] {
    /* Start at the leaf_bit and traverse our way up to the top of the tree,
    keeping track of the path we took on our way up in terms of
    bit-positions (offsets) in the stems. Then, traverse back down the same
    path to find the coords of the leaf_bit. */
    let parent_bit = self.leaf_parent(leaf_bit_pos);
    let mut stem_start = block_start(parent_bit);
    let mut offset = parent_bit - stem_start;
    let mut offsets = Vec::new();
    offsets.push(offset);
    for _ in 1..self.max_slayers {
      let parent = self.parent(stem_start);
      stem_start = parent.0;
      offset = parent.1;
      offsets.push(offset);
    }
    /* Reverse the offsets ready to traverse them back down the tree */
    offsets.reverse();
    let mut range = [[0, self.matrix_width-1], [0, self.matrix_width-1]];
    for child_offset in offsets.into_iter().take(self.max_slayers) {
      range = to_4_subranges(range)[child_offset];
    }
    let leaf_offset = leaf_bit_pos - (leaf_bit_pos/4*4);
    match leaf_offset {
      0 => [range[0][0], range[1][0]],
      1 => [range[0][1], range[1][0]],
      2 => [range[0][0], range[1][1]],
      3 => [range[0][1], range[1][1]],
      _ => [std::usize::MAX, std::usize::MAX],
    }
  }
  fn leaf_parent(&self, bit_pos: usize) -> usize {
    self.layer_start(self.max_slayers-1) + self.stem_to_leaf[bit_pos / 4]
  }
  fn parent(&self, stem_start: usize) -> (usize, usize) {
    /* Returns (stem_start, bit_offset) */
    if stem_start < self.slayer_starts[1] {
      return (std::usize::MAX, std::usize::MAX)
    }
    /* Find which layer stem_start is in */
    let stem_layer = {
      let mut layer = self.max_slayers-1; //If no match, must be in highest layer
      for (i, &layer_start) in self.slayer_starts.iter().enumerate() {
        if stem_start <= layer_start { layer = i; break }
      }
      layer
    };
    /* Find the nth stem it is in the layer */
    let stem_num = (stem_start - stem_layer)/4;
    /* Find the nth 1 in the parent layer */
    let parent_bit = one_positions_range(
      &self.stems,
      self.slayer_starts[stem_layer-1],
      self.slayer_starts[stem_layer]
    )[stem_num];
    (block_start(parent_bit), parent_bit % 4)
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

const fn block_start(bit_pos: usize) -> usize {
  (bit_pos / 4) * 4
}
fn remove_block(bit_vec: &mut BitVec, block_start: usize) -> std::result::Result<(), ()> {
  if block_start > bit_vec.len()-4
  || block_start % 4 != 0 {
    Err(())
  }
  else {
    for _ in 0..4 { bit_vec.remove(block_start); }
    Ok(())
  }
}
fn insert_block(bit_vec: &mut BitVec, block_start: usize) -> std::result::Result<(), ()> {
  if block_start > bit_vec.len()
  || block_start % 4 != 0 {
    Err(())
  }
  else {
    for _ in 0..4 { bit_vec.insert(block_start, false); }
    Ok(())
  }
}
const fn to_4_subranges(r: Range) -> [Range; 4] {
  [
    [[r[0][0], r[0][0]+((r[0][1]-r[0][0])/2)],   [r[1][0], r[1][0]+((r[1][1]-r[1][0])/2)]], //Top left quadrant
    [[r[0][0]+((r[0][1]-r[0][0])/2)+1, r[0][1]], [r[1][0], r[1][0]+((r[1][1]-r[1][0])/2)]], //Top right quadrant
    [[r[0][0], r[0][0]+((r[0][1]-r[0][0])/2)],   [r[1][0]+((r[1][1]-r[1][0])/2)+1, r[1][1]]], //Bottom left quadrant
    [[r[0][0]+((r[0][1]-r[0][0])/2)+1, r[0][1]], [r[1][0]+((r[1][1]-r[1][0])/2)+1, r[1][1]]]  //Bottom right quadrant
  ]
}
fn within_range(r: &Range, x: usize, y: usize) -> bool {
  x >= r[0][0] && x <= r[0][1] && y >= r[1][0] && y <= r[1][1]
}
fn ones_in_range(bits: &BitVec, begin: usize, end: usize) -> usize {
  bits[begin..end].into_iter().fold(0, |total, bit| total + *bit as usize)
}
fn all_zeroes(bits: &BitVec, begin: usize, end: usize) -> bool {
  bits[begin..end].into_iter().fold(true, |total, bit| total & !bit)
}
#[allow(dead_code)] //very may well need this later
fn one_positions(bits: &BitVec) -> Vec<usize> {
  bits
  .iter()
  .enumerate()
  .filter_map(
    |(pos, bit)|
    if *bit { Some(pos) }
    else   { None })
  .collect()
}
fn one_positions_range(bits: &BitVec, begin: usize, end: usize) -> Vec<usize> {
  bits[begin..end].into_iter().enumerate().filter_map(|(pos, bit)|
    if *bit { Some(pos) } else { None }
  ).collect()
}