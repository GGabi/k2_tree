use bitvec::vec::BitVec;
use crate::tree::datastore::K2Tree;

/// A struct representing the value of a bit in a K2Tree's stems.
/// 
/// This type is not intended to live for very long and
/// is not linked to the live-state of the source K2Tree,
/// so if the state of K2Tree changes then this could be invalid.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StemBit {
  /// The value of the bit.
  pub value: bool,
  /// The stem-layer of the bit. 
  pub layer: usize,
  /// The stem-number of the bit.
  pub stem: usize,
  /// The index of the bit within its stem.
  pub bit: usize,
}

/// A struct representing the value of a bit in a K2Tree's leaves.
/// 
/// This type is not intended to live for very long and
/// is not linked to the live-state of the source K2Tree,
/// so if the state of K2Tree changes then this could be invalid.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LeafBit {
  /// The value of the bit.
  pub value: bool,
  /// The x coordinate of the bit in the matrix the K2Tree represents.
  pub x: usize,
  /// The y coordinate of the bit in the matrix the K2Tree represents.
  pub y: usize,
  /// The leaf number of the bit.
  pub leaf: usize,
  /// The index of the bit within its leaf.
  pub bit: usize,
}

/// An iterator over a K2Tree's stems which produces instances of StemBit.
pub struct Stems<'a> {
  tree: &'a K2Tree,
  pos: usize,
  layer: usize,
  stem: usize,
  bit: usize,
}
impl<'a> Iterator for Stems<'a> {
  type Item = StemBit;
  fn next(&mut self) -> Option<Self::Item> {
    let stem_len = self.tree.stem_len();
    if self.pos >= self.tree.stems.len() {
      return None
    }
    /* Grab the return value */
    let ret_v = Some(StemBit {
      value: self.tree.stems[self.pos],
      layer: self.layer,
      stem: self.stem,
      bit: self.bit,
    });
    /* Increment the iterator's state for next value */
    self.pos += 1;
    if self.bit == stem_len-1 {
      self.bit = 0;
      if self.stem == (self.tree.layer_len(self.layer) / stem_len) - 1 {
        self.stem = 0;
        self.layer += 1;
      }
      else {
        self.stem += 1;
      }
    }
    else {
      self.bit += 1;
    }
    ret_v
  }
}
impl<'a> Stems<'a> {
  /// Produces a Stems iterator from a reference to a K2Tree.
  pub fn new(tree: &'a K2Tree) -> Self {
    Self {
      tree,
      pos: 0,
      layer: 0,
      stem: 0,
      bit: 0,
    }
  }
}

/// A consuming iterator over a K2Tree's stems which produces instances of StemBit.
pub struct IntoStems {
  tree: K2Tree,
  pos: usize,
  layer: usize,
  stem: usize,
  bit: usize,
}
impl Iterator for IntoStems {
  type Item = StemBit;
  fn next(&mut self) -> Option<Self::Item> {
    let stem_len = self.tree.stem_len();
    if self.pos >= self.tree.stems.len() {
      return None
    }
    /* Grab the return value */
    let ret_v = Some(StemBit {
      value: self.tree.stems[self.pos],
      layer: self.layer,
      stem: self.stem,
      bit: self.bit,
    });
    /* Increment the iterator's state for next value */
    self.pos += 1;
    if self.bit == stem_len-1 {
      self.bit = 0;
      if self.stem == (self.tree.layer_len(self.layer) / stem_len) - 1 {
        self.stem = 0;
        self.layer += 1;
      }
      else {
        self.stem += 1;
      }
    }
    else {
      self.bit += 1;
    }
    ret_v
  }
}
impl IntoStems {
  /// Produces a Stems iterator from a reference to a K2Tree.
  pub fn new(tree: K2Tree) -> Self {
    Self {
      tree,
      pos: 0,
      layer: 0,
      stem: 0,
      bit: 0,
    }
  }
}

/// An iterator over a K2Tree's leaves which produces instances of LeafBit.
pub struct Leaves<'a> {
  tree: &'a K2Tree,
  pos: usize,
}
impl<'a> Iterator for Leaves<'a> {
  type Item = LeafBit;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos == self.tree.leaves.len() { return None }
    let [x, y] = self.tree.get_coords(self.pos);
    let value = self.tree.leaves[self.pos];
    let leaf = self.pos / self.tree.leaf_len();
    let bit = self.pos % self.tree.leaf_len();
    self.pos += 1;
    Some(LeafBit {
      value,
      x,
      y,
      leaf,
      bit
    })
  }
}
impl<'a> Leaves<'a> {
  /// Produces a Leaves iterator from a reference to a K2Tree.
  pub fn new(tree: &'a K2Tree) -> Self {
    Self {
      tree,
      pos: 0,
    }
  }
}

/// A consuming iterator over a K2Tree's leaves which produces instances of LeafBit.
pub struct IntoLeaves {
  tree: K2Tree,
  pos: usize,
}
impl Iterator for IntoLeaves {
  type Item = LeafBit;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos == self.tree.leaves.len() { return None }
    let leaf_len = self.tree.leaf_len();
    let [x, y] = self.tree.get_coords(self.pos);
    let value = self.tree.leaves[self.pos];
    let leaf = self.pos / leaf_len;
    let bit = self.pos % leaf_len;
    self.pos += 1;
    Some(LeafBit {
      value,
      x,
      y,
      leaf,
      bit
    })
  }
}
impl IntoLeaves {
  /// Produces an IntoLeaves iterator from a K2Tree.
  pub fn new(tree: K2Tree) -> Self {
    Self {
      tree,
      pos: 0,
    }
  }
}

/// An iterator over a K2Tree's stems which produces the raw boolean-values of each bit.
pub struct StemsRaw<'a> {
  stems: &'a BitVec,
  pos: usize,
}
impl<'a> Iterator for StemsRaw<'a> {
  type Item = bool;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.stems.len() {
      return None
    }
    let ret_v = Some(self.stems[self.pos]);
    self.pos += 1;
    ret_v
  }
}
impl<'a> StemsRaw<'a> {
  /// Produces a StemsRaw iterator from a reference to a K2Tree.
  pub fn new(tree: &'a K2Tree) -> Self {
    Self {
      stems: &tree.stems,
      pos: 0,
    }
  }
}

/// An iterator over a K2Tree's leaves which prduces the raw boolean-values of each bit.
pub struct LeavesRaw<'a> {
  leaves: &'a BitVec,
  pos: usize,
}
impl<'a> Iterator for LeavesRaw<'a> {
  type Item = bool;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.leaves.len() {
      return None
    }
    let ret_v = Some(self.leaves[self.pos]);
    self.pos += 1;
    ret_v
  }
}
impl<'a> LeavesRaw<'a> {
  /// Produces a LeavesRaw iterator from a reference to a K2Tree.
  pub fn new(tree: &'a K2Tree) -> Self {
    Self {
      leaves: &tree.leaves,
      pos: 0,
    }
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   #[test]
//   fn IntoStems() {
//     let tree = K2Tree::
//   }
// }