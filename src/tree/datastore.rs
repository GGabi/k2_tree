use {
  bitvec::{prelude::{bitvec, bitbox, BitVec}},
  crate::error::K2TreeError as Error,
  crate::tree::*,
  crate::matrix::BitMatrix,
};

type Result<T> = std::result::Result<T, Error>;

/*
  Funcs which use slayer_starts:
  - set()
  - grow()
  - shrink()
  - Trait Display
  - stem_to_leaf_start()

  Instances used in hot code:
  - None?
*/

/// A collection designed to efficiently compress sparsely-populated bit-matrices.
///
/// The `K2Tree` represents a matrix of bits and behaves ***as if*** it is a bit-matrix.
/// The `k` value of this structure is currently fixed at 2, but future updates may allow customisation.
/// The matrix represented by the `K2Tree` must always be square, with a width/height equal to a power of k: 8, 16, 32 etc.
/// This isn't much of an issue because almost all empty cells in the matrix are compressed-away, so don't stress about wasted columns/rows.
/// 
/// ```
/// fn main() -> Result<(), k2_tree::error::K2TreeError> {
///   use k2_tree::K2Tree;
///   //matrix_width = 8, k = 2
///   let mut tree = K2Tree::with_k(2, 2)?;
///   tree.set(0, 4, true);
///   tree.set(6, 5, true);
///   tree.set(0, 4, false);
///   assert_eq!(false, tree.get(0, 4)?);
///   assert_eq!(true, tree.get(6, 5)?);
///   assert_eq!(false, tree.get(0, 0)?);
///   Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct K2Tree {
  /// The k value of the K2Tree's stems.
  pub stem_k: usize,
  /// The k value of the K2Tree's leaves.
  pub leaf_k: usize,
  /// The maximum number of stem-layers possible given the matrix_width.
  pub max_slayers: usize, //Could I encode this in the length of slayer_starts? Without making code much slower?
  /// The index of the first bit in each stem-layer in stems.
  pub slayer_starts: Vec<usize>,
  /// The bits that comprise the stems of the tree. 
  pub stems: BitVec,
  /// The bits that comprise the leaves of the tree.
  pub leaves: BitVec,
}

/* Public */
impl K2Tree {
  /// Returns a `K2Tree` representing an 8x8 bit-matrix of all 0s. K = 2.
  /// ```
  /// use k2_tree::K2Tree;
  /// let tree = K2Tree::new();
  /// assert!(tree.is_empty());
  /// assert_eq!(8, tree.matrix_width());
  /// assert_eq!(2, tree.stem_k);
  /// assert_eq!(2, tree.leaf_k);
  /// ```
  pub fn new() -> Self {
    K2Tree {
      stem_k: 2,
      leaf_k: 2,
      max_slayers: 2,
      slayer_starts: vec![0],
      stems: bitvec![0; 4],
      leaves: BitVec::new(),
    }
  }
  /// Returns a `K2Tree` with a specified k-value, which represents an empty bit-matrix
  /// of width `k.pow(3)`.
  /// 
  /// Returns a SmallKValue error if k < 2.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let tree = K2Tree::with_k(4, 4)?;
  ///   assert!(tree.is_empty());
  ///   assert_eq!(4usize.pow(3), tree.matrix_width());
  ///   assert_eq!(64, tree.matrix_width());
  ///   assert_eq!(4, tree.stem_k);
  ///   assert_eq!(4, tree.leaf_k);
  ///   Ok(())
  /// }
  /// ``` 
  pub fn with_k(stem_k: usize, leaf_k: usize) -> Result<Self> {
    if stem_k < 2 {
      return Err(Error::SmallStemKValue { stem_k: stem_k as u8 })
    }
    else if leaf_k < 2 {
      return Err(Error::SmallLeafKValue { leaf_k: leaf_k as u8 })
    }
    Ok(K2Tree {
      stem_k,
      leaf_k,
      max_slayers: 2,
      slayer_starts: vec![0],
      stems: bitvec![0; stem_k*stem_k],
      leaves: BitVec::new(),
    })
  }
  /// Changes the stem_k value of a `K2Tree`. This can be a time and space expensive operation
  /// for large, non-sparse datasets.
  /// Returns a SmallKValue error if stem_k < 2.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::new();
  ///   tree.set_stem_k(4);
  ///   assert!(tree.set_stem_k(1).is_err());
  ///   Ok(())
  /// }
  /// ``` 
  pub fn set_stem_k(&mut self, stem_k: usize) -> Result<()> {
    if self.stem_k == stem_k { return Ok(()) }
    if stem_k < 2 {
      return Err(Error::SmallStemKValue{stem_k: stem_k as u8})
    }
    *self = K2Tree::from_matrix(self.to_matrix()?, stem_k, self.leaf_k)?;
    Ok(())
  }
  /// Changes the leaf_k value of a `K2Tree`. This can be a time and space expensive operation
  /// for large, non-sparse datasets.
  /// Returns a SmallKValue error if stem_k < 2.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::new();
  ///   tree.set_leaf_k(4);
  ///   assert!(tree.set_leaf_k(1).is_err());
  ///   Ok(())
  /// }
  /// ``` 
  pub fn set_leaf_k(&mut self, leaf_k: usize) -> Result<()> {
    if self.leaf_k == leaf_k { return Ok(()) }
    if leaf_k < 2 {
      return Err(Error::SmallLeafKValue{leaf_k: leaf_k as u8})
    }
    *self = K2Tree::from_matrix(self.to_matrix()?, self.stem_k, leaf_k)?;
    Ok(())
  }
  ///Returns true if a `K2Tree` contains no 1s.
  pub fn is_empty(&self) -> bool {
    ones_in_range(&self.leaves, 0, self.leaves.len()) == 0
  }
  /// Returns that state of a bit at a specified coordinate in the bit-matrix the
  /// `K2Tree` represents.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.set(0, 1, true)?;
  ///   assert_eq!(true, tree.get(0, 1)?);
  ///   assert_eq!(false, tree.get(0, 0)?);
  ///   Ok(())
  /// }
  /// ```
  pub fn get(&self, x: usize, y: usize) -> Result<bool> {
    let matrix_width = self.matrix_width();
    if x >= matrix_width || y >= matrix_width {
      return Err(Error::Read {
        source: Box::new(Error::OutOfBounds {
          x_y: [x, y],
          min_x_y: [0, 0],
          max_x_y: [matrix_width-1; 2]
        })
      })
    }
    let descend_result = match self.matrix_bit(x, y, matrix_width) {
      Ok(dr) => dr,
      Err(e) => return Err(Error::Read {
        source: Box::new(e)
      }),
    };
    match descend_result {
      DescendResult::Leaf(leaf_start, leaf_range) => {
        if leaf_range.width() != self.leaf_k
        || leaf_range.height() != self.leaf_k {
          return Err(Error::Read {
            source: Box::new(Error::TraverseError{x, y})
          })
        }
        //Calculation removes extra branches, makes it faster
        // range = [[5, 6], [7, 8]]
        // (5, 7) = 0; (6, 7) = 1; (5, 8) = 2; (6, 8) = 3
        let offset = (self.leaf_k * (y - leaf_range.min_y)) + (x - leaf_range.min_x);
        Ok(self.leaves[leaf_start+offset])
      },
      DescendResult::Stem(_, _) => Ok(false),
    }
  }
  /// Returns a BitVec containing the bits in a specified row, in order.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use bitvec::prelude::bitvec;
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.set(1, 0, true)?;
  ///   tree.set(3, 0, true)?;
  ///   tree.set(6, 0, true)?;
  ///   assert_eq!(
  ///     vec![false,true,false,true,false,false,true,false],
  ///     tree.get_row(0)?
  ///   );
  ///   Ok(())
  /// }
  /// ```
  pub fn get_row(&self, y: usize) -> Result<Vec<bool>> {
    let matrix_width = self.matrix_width();
    if y >= matrix_width {
      return Err(Error::Read {
        source: Box::new(Error::OutOfBounds {
          x_y: [0, y],
          min_x_y: [0, 0],
          max_x_y: [matrix_width-1; 2]
        })
      })
    }
    let mut ret_v = Vec::new();
    for x in (0..matrix_width).step_by(self.leaf_k) {
      let descend_result = match self.matrix_bit(x, y, matrix_width) {
        Ok(dr) => dr,
        Err(e) => return Err(Error::Read {
          source: Box::new(e)
        }),
      };
      match descend_result {
        DescendResult::Leaf(leaf_start, leaf_range) => {
          if leaf_range.width() != self.leaf_k
          || leaf_range.height() != self.leaf_k {
            return Err(Error::Read {
              source: Box::new(Error::TraverseError{x, y})
            })
          }
          //Calculation instead of if-else block makes hot-code much faster
          let offset = (self.leaf_k * (y - leaf_range.min_y)) + (x - leaf_range.min_x);
          for i in 0..self.leaf_k { ret_v.push(self.leaves[leaf_start+offset+i]); }
        },
        DescendResult::Stem(_, _) => {
          for _ in 0..self.leaf_k { ret_v.push(false); }
        },
      }
    };
    Ok(ret_v)
  }
  /// Returns a BitVec containing the bits in a specified column, in order.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use bitvec::prelude::bitvec;
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.set(1, 1, true)?;
  ///   tree.set(1, 3, true)?;
  ///   tree.set(1, 6, true)?;
  ///   assert_eq!(
  ///     vec![false,true,false,true,false,false,true,false],
  ///     tree.get_column(1)?
  ///   );
  ///   Ok(())
  /// }
  /// ```
  pub fn get_column(&self, x: usize) -> Result<Vec<bool>> {
    let matrix_width = self.matrix_width();
    if x >= matrix_width {
      return Err(Error::Read {
        source: Box::new(Error::OutOfBounds {
          x_y: [x, 0],
          min_x_y: [0, 0],
          max_x_y: [matrix_width-1; 2]
        })
      })
    }
    let mut ret_v = Vec::new();
    for y in (0..matrix_width).step_by(self.leaf_k) {
      let descend_result = match self.matrix_bit(x, y, matrix_width) {
        Ok(dr) => dr,
        Err(e) => return Err(Error::Read {
          source: Box::new(e)
        }),
      };
      match descend_result{
        DescendResult::Leaf(leaf_start, leaf_range) => {
          if leaf_range.width() != self.leaf_k
          || leaf_range.height() != self.leaf_k {
            return Err(Error::Read {
              source: Box::new(Error::TraverseError{x, y})
            })
          }
          let offset = (self.leaf_k * (y - leaf_range.min_y)) + (x - leaf_range.min_x);
          for i in 0..self.leaf_k { ret_v.push(self.leaves[leaf_start+offset+(i*self.leaf_k)]); }
        },
        DescendResult::Stem(_, _) => {
          for _ in 0..self.leaf_k { ret_v.push(false); }
        },
      }
    };
    Ok(ret_v)
  }
  /// Sets the state of a bit at the coordinates (x, y) in the bit-matrix the
  /// K2Tree represents.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   assert_eq!(false, tree.get(0, 0)?);
  ///   tree.set(0, 0, true)?;
  ///   assert_eq!(true, tree.get(0, 0)?);
  ///   Ok(())
  /// }
  /// ```
  pub fn set(&mut self, x: usize, y: usize, state: bool) -> Result<()> {
    let matrix_width = self.matrix_width();
    if x >= matrix_width || y >= matrix_width {
      return Err(Error::Write {
        source: Box::new(Error::OutOfBounds {
          x_y: [x, y],
          min_x_y: [0, 0],
          max_x_y: [matrix_width-1; 2]
        })
      })
    }
    let stem_len = self.stem_len();
    let leaf_len = self.leaf_len();
    let descend_result = match self.matrix_bit(x, y, matrix_width) {
      Ok(dr) => dr,
      Err(e) => {
        return Err(Error::Write {
          source: Box::new(e)
        })
      },
    };
    match descend_result {
      DescendResult::Leaf(leaf_start, leaf_range) => {
        if leaf_range.width() != self.leaf_k
        || leaf_range.height() != self.leaf_k {
          /* Final submatrix isn't a k by k so can't be a leaf */
          return Err(Error::Write {
            source: Box::new(Error::TraverseError{x, y})
          })
        }
        /* Set the bit in the leaf to the new state */
        let offset = (self.leaf_k * (y - leaf_range.min_y)) + (x - leaf_range.min_x);
        self.leaves.set(leaf_start+offset, state);
        /* If leaf is now all 0's, remove leaf and alter rest of struct to reflect changes.
        Loop up the stems changing the parent bits to 0's and removing stems that become all 0's */
        if !state && all_zeroes(&self.leaves, leaf_start, leaf_start+leaf_len) {
          /* - Remove the leaf
              - Use find the dead leaf's parent bit
              - Set parent bit to 0, check if stem now all 0's
              - If all 0's:
              - - Remove stem
              - - Alter layer_starts if needed
              - - Find parent bit and set to 0
              - - Repeat until reach stem that isn't all 0's or reach stem layer 0 */
          if let Err(()) = remove_block(&mut self.leaves, leaf_start, leaf_len) {
            return Err(Error::CorruptedK2Tree {
              source: Box::new(Error::Write {
                source: Box::new(Error::LeafRemovalError {
                  pos: leaf_start,
                  len: leaf_len
                })
              })
            })
          }
          let stem_bit_pos = self.leaf_parent(leaf_start); //TODO: check
          if self.leaves.is_empty() {
            /* If no more leaves, then remove all stems immediately
            and don't bother with complex stuff below */
            self.stems = bitvec![0; stem_len];
            self.slayer_starts = vec![0];
            return Ok(())
          }
          self.stems.set(stem_bit_pos, false); //Dead leaf parent bit = 0
          let mut curr_layer = self.max_slayers-1;
          let mut stem_start = self.stem_start(stem_bit_pos);
          while curr_layer > 0
          && all_zeroes(&self.stems, stem_start, stem_start+stem_len) {
            for layer_start in &mut self.slayer_starts[curr_layer+1..] {
              // NOTE: this was 1 but it looks like that was an uncaught error, changed to stem_len
              //       if any errors, look here.
              *layer_start -= stem_len; //Adjust lower layer start positions to reflect removal of stem
            }
            let [parent_stem_start, bit_offset] = self.parent(stem_start).unwrap();
            if let Err(()) = remove_block(&mut self.stems, stem_start, stem_len) {
              return  Err(Error::CorruptedK2Tree {
                source: Box::new(Error::Write {
                  source: Box::new(Error::StemRemovalError {
                    pos: stem_start,
                    len: stem_len
                  })
                })
              })
            }
            self.stems.set(parent_stem_start + bit_offset, false);
            stem_start = parent_stem_start;
            curr_layer -= 1;
          }
        }
      },
      DescendResult::Stem(mut stem_start, mut stem_range) if state => {
        /* Descend returning Stem means no Leaf containing bit at (x, y),
        must be located in a submatrix of all 0's.
        If state = false: do nothing 
        If state = true:
          - Construct needed stems until reach final layer
          - Construct leaf corresponding to range containing (x, y)
          - Set bit at (x, y) to 1 */
        //Either 0 or == max_slayers?
        let mut layer_starts_len = self.slayer_starts.len(); //cannot replace with max_slayers bc might not be max?
        let mut layer = self.layer_from_range(stem_range);
        let mut subranges: SubRanges;
        /* Create correct stems in layers on the way down to the final layer,
        which points to the leaves */
        while layer < self.max_slayers-1 {
          subranges = match self.to_subranges(stem_range) {
            Ok(subranges) => subranges,
            Err(error) => return Err(Error::CorruptedK2Tree {
              source: Box::new(Error::Write {
                source: Box::new(Error::SubRangesError {
                  source: Box::new(error),
                }),
              }),
            })
          };
          let (child_pos, &subrange) =
            match subranges.iter().enumerate().find(
              |(_, subrange)| subrange.contains(x, y)
            ) {
              Some(val) => val,
              None => return Err(Error::CorruptedK2Tree {
                source: Box::new(Error::Write {
                  source: Box::new(Error::TraverseError{x, y})
                })
              })
          };
          /* Change bit containing (x, y) to 1 */
          self.stems.set(stem_start + child_pos, true);
          /* If we're not at max possible layer, but at the lowest
          but at the lowest existing layer: Create new layer before
          adding new stem to it.
          Otherwise: Find the correct position to add the new stem
          in the child layer. */
          if layer == layer_starts_len-1 {
            stem_start = self.stems.len();
            self.slayer_starts.push(stem_start);
            layer_starts_len += 1;
          }
          else {
            stem_start = match self.child_stem(layer, stem_start, child_pos) {
              Ok(ss) => ss,
              Err(()) => return Err(Error::CorruptedK2Tree {
                source: Box::new(Error::Write {
                  source: Box::new(Error::TraverseError{x, y})
                })
              }),
            };
          }
          /* We're now working on the child layer */
          layer += 1;
          stem_range = subrange;
          if let Err(()) = insert_block(&mut self.stems, stem_start, stem_len) {
            return Err(Error::CorruptedK2Tree {
              source: Box::new(Error::Write {
                source: Box::new(Error::StemInsertionError {
                  pos: stem_start,
                  len: stem_len
                })
              })
            })
          }
          /* If there are layers after the one we just insert a stem
          into: Increase the layer_starts to account for
          the extra stem */
          for layer_start in &mut self.slayer_starts[layer+1..] {
            *layer_start += stem_len;
          }
        }
        /* We're at the final stem layer */
        subranges = match self.to_subranges(stem_range) { 
          Ok(subranges) => subranges,
          Err(error) => return Err(Error::CorruptedK2Tree {
            source: Box::new(Error::Write {
              source: Box::new(Error::SubRangesError {
                source: Box::new(error),
              }),
            }),
          })
        };
        let (child_pos, &subrange) =
          match subranges.iter().enumerate().find(
            |(_, subrange)| subrange.contains(x, y)
          ) {
            Some(val) => val,
            None => return Err(Error::CorruptedK2Tree {
              source: Box::new(Error::Write {
                source: Box::new(Error::TraverseError{x, y})
              })
            })
        };
        /* Set the correct stem bit to 1 */
        self.stems.set(stem_start + child_pos, true);
        /* Find the index to insert the new leaf */
        let nth_leaf = ones_in_range(
          &self.stems,
          self.layer_start(self.max_slayers-1),
          stem_start + child_pos
        );
        let leaf_start = nth_leaf * leaf_len; //TODO: Check
        /* Create new leaf of all 0's */
        if let Err(()) = insert_block(&mut self.leaves, leaf_start, leaf_len) {
          return Err(Error::CorruptedK2Tree {
            source: Box::new(Error::Write {
              source: Box::new(Error::LeafInsertionError {
                pos: leaf_start,
                len: leaf_len
              })
            })
          })
        }
        /* Change bit at (x, y) to 1 */
        let leaf_range = subrange;
        let offset = (self.leaf_k * (y - leaf_range.min_y)) + (x - leaf_range.min_x);
        self.leaves.set(leaf_start+offset, true);
      }
      _ => {},
    };
    Ok(())
  }
  /// Returns the width of the bit-matrix that a K2Tree represents.
  /// 
  /// The matrix is always square, so this is also the height.
  /// 
  /// This can only have certain values, depending on the values of leaf_k and stem_k,
  /// so it is common for a K2Tree's matrix_width to be greater than the matrix it
  /// was built from. Thankfully, trailing rows/columns have no affect on the size
  /// of the K2Tree.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::{K2Tree, matrix::BitMatrix};
  ///   let matrix = BitMatrix::with_dimensions(8, 8);
  ///   let tree = K2Tree::from_matrix(matrix, 2, 2)?;
  ///   assert_eq!(8, tree.matrix_width());
  ///   Ok(())
  /// }
  /// ```
  pub fn matrix_width(&self) -> usize {
    self.leaf_k * (self.stem_k.pow(self.max_slayers as u32))
  }
  /// Returns an iterator over the K2Tree's stems which produces instances of StemBit.
  /// 
  /// StemBit contains extra information on the layer, block and offset of the specific
  /// bit in the stems.
  pub fn stems(&self) -> iterators::Stems<'_> {
    iterators::Stems::new(self)
  }
  /// Consumes the K2Tree to return an iterator over its stems, which produces instances of StemBit.
  /// 
  /// StemBit contains extra information on the layer, block and offset of the specific
  /// bit in the stems.
  pub fn into_stems(self) -> iterators::IntoStems {
    iterators::IntoStems::new(self)
  }
  /// Returns an iterator over the K2Tree's stems which produces only the raw boolean values.
  pub fn stems_raw(&self) -> iterators::StemsRaw<'_> {
    iterators::StemsRaw::new(self)
  }
  /// Returns an iterator over the K2Tree's leaves which produces instances of LeafBit.
  /// 
  /// LeafBit contains extra information on the exact coordinates of each bit in the leaves.
  pub fn leaves(&self) -> iterators::Leaves<'_> {
    iterators::Leaves::new(self)
  }
  /// Consumes the K2Tree to return an iterator over its leaves, which produces instances of LeafBit.
  /// 
  /// LeafBit contains extra information on the exact coordinates of each bit in the leaves.
  pub fn into_leaves(self) -> iterators::IntoLeaves {
    iterators::IntoLeaves::new(self)
  }
  /// Returns an iterator over the K2Tree's leaves which produces only the raw boolean values.
  pub fn leaves_raw(&self) -> iterators::LeavesRaw<'_> {
    iterators::LeavesRaw::new(self)
  }
  /// Increases the height and width of the matrix the K2Tree represents by a factor of k.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   assert_eq!(2, tree.stem_k);
  ///   assert_eq!(2, tree.leaf_k);
  ///   assert_eq!(8, tree.matrix_width());
  ///   tree.grow();
  ///   assert_eq!(16, tree.matrix_width());
  ///   tree.grow();
  ///   assert_eq!(32, tree.matrix_width());
  ///   Ok(())
  /// }
  /// ```
  pub fn grow(&mut self) {
    let stem_len = self.stem_len();
    self.max_slayers += 1;
    if self.leaves.len() > 0  {
      /* Only insert the extra layers etc. if the
      tree isn't all 0s */
      for slayer_start in &mut self.slayer_starts {
        *slayer_start += stem_len;
      }
      self.slayer_starts.insert(0, 0);
      /* Insert 10...00 to beginning of stems */
      for _ in 0..stem_len-1 { self.stems.insert(0, false); }
      self.stems.insert(0, true);
    }
  }
  /// Only shrinks the height and width of the matrix the K2Tree represents by a factor of k
  /// if it is possible.
  /// 
  /// Does not Err if the matrix cannot be shrunk i.e. it is already at the minimum size.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.grow();
  ///   assert_eq!(16, tree.matrix_width());
  ///   tree.shrink_if_possible();
  ///   assert_eq!(8, tree.matrix_width());
  ///   tree.shrink_if_possible();
  ///   assert_eq!(8, tree.matrix_width());
  ///   Ok(())
  /// }
  /// ```
  pub fn shrink_if_possible(&mut self) {
    match self.shrink() {
      _ => ()
    }
  }
  /// Attempts to reduce the height and width of the matrix the K2Tree represents by a factor of k.
  /// 
  /// Returns an Err if the matrix cannot be shrunk i.e. it is already at the minimum size.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.grow();
  ///   assert_eq!(16, tree.matrix_width());
  ///   assert!(tree.shrink().is_ok());
  ///   assert_eq!(8, tree.matrix_width());
  ///   assert!(tree.shrink().is_err());
  ///   Ok(())
  /// }
  /// ```
  pub fn shrink(&mut self) -> Result<()> {
    let stem_len = self.stem_len();
    if self.matrix_width() <= self.leaf_k * self.stem_k.pow(2) {
      return Err(Error::CouldNotShrink {
        reason: format!("Already at minimum size: {}", self.matrix_width())
      })
    }
    else if self.stems[1..stem_len] != bitbox![0; stem_len-1] {
      return Err(Error::CouldNotShrink {
        reason: "Shrinking would lose information about the matrix".into()
      })
    }
    self.max_slayers -= 1;
    self.slayer_starts.remove(0);
    for slayer_start in &mut self.slayer_starts {
      *slayer_start -= stem_len;
    }
    /* Remove top layer stem */
    for _ in 0..stem_len { self.stems.remove(0); }
    Ok(())
  }
  /// Reduces the height and width of the matrix the K2Tree represents by a factor of k without
  /// doing any bounds checking before or integrity checking afterwards.
  /// 
  /// # Safety
  /// Do not attempt to shrink matrix_width smaller than k^3.
  /// 
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.grow();
  ///   assert_eq!(16, tree.matrix_width());
  ///   unsafe { tree.shrink_unchecked(); }
  ///   assert_eq!(8, tree.matrix_width());
  ///   Ok(())
  /// }
  /// ```
  pub unsafe fn shrink_unchecked(&mut self) {
    let stem_len = self.stem_len();
    self.max_slayers -= 1;
    self.slayer_starts.remove(0);
    for slayer_start in &mut self.slayer_starts {
      *slayer_start -= stem_len;
    }
    /* Remove top layer stem */
    for _ in 0..stem_len { self.stems.remove(0); }
  }
  /// Comsumes the K2Tree to produce the bit-matrix it represented.
  /// 
  /// The matrix is presented as a list of columns of bits, Vec<Vec<bool>>.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.set(0, 0, true)?;
  ///   tree.set(5, 6, true)?;
  ///   tree.set(7, 7, true)?;
  ///   let matrix = tree.into_matrix()?;
  ///   assert_eq!(true, matrix.get(0, 0).unwrap());
  ///   assert_eq!(true, matrix.get(5, 6).unwrap());
  ///   assert_eq!(true, matrix.get(7, 7).unwrap());
  ///   assert_eq!(false, matrix.get(4, 3).unwrap());
  ///   Ok(())
  /// }
  /// ```
  pub fn into_matrix(self) -> Result<BitMatrix> {
    let matrix_width = self.matrix_width();
    let mut m = BitMatrix::with_dimensions(matrix_width, matrix_width);
    for (pos, &state) in self.leaves.iter().enumerate() {
      if state {
        let [x, y] = self.get_coords(pos);
        if let Err(e) = m.set(x, y, true) {
          return Err(Error::BitMatrixError {
            source: Box::new(e),
          })
        }
      }
    }
    Ok(m)
  }
  /// Produces the bit-matrix a K2Tree represents.
  /// 
  /// The matrix is presented as a list of columns of bits, Vec<Vec<bool>>.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::with_k(2, 2)?;
  ///   tree.set(0, 0, true)?;
  ///   tree.set(5, 6, true)?;
  ///   tree.set(7, 7, true)?;
  ///   let matrix = tree.to_matrix()?;
  ///   assert_eq!(true, matrix.get(0, 0).unwrap());
  ///   assert_eq!(true, matrix.get(5, 6).unwrap());
  ///   assert_eq!(true, matrix.get(7, 7).unwrap());
  ///   assert_eq!(false, matrix.get(4, 3).unwrap());
  ///   Ok(())
  /// }
  /// ```
  pub fn to_matrix(&self) -> Result<BitMatrix> {
    let matrix_width = self.matrix_width();
    let mut m = BitMatrix::with_dimensions(matrix_width, matrix_width);
    for (pos, &state) in self.leaves.iter().enumerate() {
      if state {
        let [x, y] = self.get_coords(pos);
        if let Err(e) = m.set(x, y, true) {
          return Err(Error::BitMatrixError {
            source: Box::new(e),
          })
        }
      }
    }
    Ok(m)
  }
  /// Constructs a K2Tree which represents the state of the input matrix.
  /// 
  /// All types that can produce rows of bits are valid inputs.
  /// ```
  /// use k2_tree::{K2Tree, matrix::BitMatrix};
  /// let mut m = BitMatrix::with_dimensions(8, 8);
  /// m.set(0, 5, true);
  /// assert!(K2Tree::from_matrix(m, 2, 2).is_ok());
  /// ```
  pub fn from_matrix(matrix: BitMatrix, stem_k: usize, leaf_k: usize) -> Result<Self> {
    let mut tree = K2Tree::with_k(stem_k, leaf_k)?;
    while matrix.width > tree.matrix_width()
    || matrix.height > tree.matrix_width() {
      tree.grow();
    }
    let rows = matrix.into_rows();
    for (y, row) in rows.into_iter().enumerate() {
      let xs = one_positions(row.into_iter());
      for x in xs.into_iter() {
        tree.set(x, y, true)?;
      }
    }
    Ok(tree)
  }
}

/* Traits */
impl core::fmt::Display for K2Tree {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    if self.leaves.len() == 0 { return write!(f, "[0000]") }
    let mut s = String::new();
    let mut i: usize = 1;
    let layer_starts = self.layer_starts();
    for layer_num in 0..self.max_slayers {
      for bit_pos in layer_starts[layer_num]..layer_starts[layer_num+1] {
        if self.stems[bit_pos] { s.push('1'); }
        else { s.push('0'); }
        if i == self.stem_k*self.stem_k
        && (bit_pos - layer_starts[layer_num]) < self.layer_len(layer_num)-1 {
          s.push_str(", ");
          i = 1;
        } 
        else { i += 1; }
      }
      i = 1;
      s.push_str("; ");
    }
    i = 1;
    for bit_pos in 0..self.leaves.len() {
      if self.leaves[bit_pos] { s.push('1'); }
      else { s.push('0'); }
      if i == self.leaf_k*self.leaf_k
      && bit_pos < self.leaves.len()-1 {
        s.push_str(", ");
        i = 1;
      } 
      else { i += 1; }
    }
    write!(f, "[{}]", s)
  }
}
impl PartialEq for K2Tree {
  fn eq(&self, other: &Self) -> bool {
    self.stem_k == other.stem_k
    && self.leaf_k == other.leaf_k
    && self.max_slayers == other.max_slayers
    && self.stems == other.stems
    && self.leaves == other.leaves
  }
}
impl Eq for K2Tree {}
impl Default for K2Tree {
  fn default() -> Self {
    Self::new()
  }
}
impl std::hash::Hash for K2Tree {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.stem_k.hash(state);
    self.leaf_k.hash(state);
    self.max_slayers.hash(state);
    self.stems.hash(state);
    self.leaves.hash(state);
  }
}

/* Private */
enum DescendResult {
  Leaf(usize, Range2D), //leaf_start, leaf_range
  Stem(usize, Range2D), //stem_start, stem_range
}
struct DescendEnv {
  /* Allows for descend to be recursive without parameter hell */
  x: usize,
  y: usize,
  slayer_max: usize,
}
impl K2Tree {
  fn layer_from_range(&self, r: Range2D) -> usize {
    (self.max_slayers+1) -
    (
      ((r.width()/self.leaf_k) as f64).log(self.stem_k as f64) as usize
      +1
    )
  }
  fn matrix_bit(&self, x: usize, y: usize, m_width: usize) -> Result<DescendResult> {
    let env = DescendEnv {
      x,
      y,
      slayer_max: self.max_slayers-1,
    };
    self.descend(&env, 0, 0, Range2D::new(0, m_width-1, 0, m_width-1))
  }
  fn descend(&self, env: &DescendEnv, layer: usize, stem_pos: usize, range: Range2D) -> Result<DescendResult> {
    let subranges = self.to_subranges(range)?;
    for (child_pos, child) in self.stems[stem_pos..stem_pos+self.stem_len()].iter().enumerate() {
      if subranges[child_pos].contains(env.x, env.y) {
        if !child { return Ok(DescendResult::Stem(stem_pos, range)) } //The bit exists within a range that has all zeros
        else if layer == env.slayer_max {
          let leaf_start = match self.stem_to_leaf_start(stem_pos + child_pos) {
            Ok(ls) => ls,
            Err(_) => return Err(Error::TraverseError {
              x: env.x,
              y: env.y
            }),
          };
          return Ok(DescendResult::Leaf(leaf_start, subranges[child_pos]))
        }
        else {
          let child_stem = match self.child_stem(layer, stem_pos, child_pos) {
            Ok(cs) => cs,
            Err(_) => return Err(Error::TraverseError {
              x: env.x,
              y: env.y
            }),
          };
          return self.descend(env,
                              layer+1,
                              child_stem,
                              subranges[child_pos])
        }
      }
    }
    unreachable!()
  }
  fn num_stems_before_child(&self, bit_pos: usize, layer: usize) -> usize {
    ones_in_range(&self.stems, self.layer_start(layer), bit_pos)
  }
  fn stem_to_leaf_start(&self, stem_bitpos: usize) -> std::result::Result<usize, ()> {
    if !self.stems[stem_bitpos] { Err(()) }
    else {
      let nth_leaf = ones_in_range(
        &self.stems,
        self.slayer_starts[self.max_slayers-1],
        stem_bitpos
      );
      Ok(nth_leaf * self.leaf_len())
    }
  }
  fn child_stem(&self, layer: usize, stem_start: usize, nth_child: usize) -> std::result::Result<usize, ()> {
    if !self.stems[stem_start+nth_child]
    || layer == self.max_slayers-1 {
      /* If stem_bit is 0 or final stem layer, cannot have children */
      return Err(())
    }
    Ok(self.layer_start(layer+1)
    + (self.num_stems_before_child(stem_start+nth_child, layer) * self.stem_len()))
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
        slayer_starts: vec![0, 4],
        stems:  bitvec![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0],
        leaves: bitvec![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0],
      },
      3 => K2Tree {
        stem_k: 3,
        leaf_k: 3,
        max_slayers: 2,
        slayer_starts: vec![0, 9],
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
        slayer_starts: vec![0, 16],
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
  fn parent_stem(&self, stem_start: usize) -> usize {
    self.parent(stem_start).unwrap()[0]
  }
  fn parent_bit(&self, stem_start: usize) -> usize {
    let [stem_start, bit_offset] = self.parent(stem_start).unwrap();
    stem_start + bit_offset
  }
  #[allow(dead_code)]
  fn footprint(&self) -> usize {
    let mut size: usize = std::mem::size_of_val(self);
    size += std::mem::size_of::<usize>() * self.slayer_starts.len();
    size += self.stems.len() / 8;
    size += self.leaves.len() / 8;
    size
  }
  #[allow(dead_code)]
  fn theoretical_size(&self) -> usize {
    (self.stems.len() + self.leaves.len()) / 8
  }
}

/* Public Interface Tests */
#[cfg(test)]
mod api {
  use super::*;
  use bitvec::bitbox;
  #[test]
  fn new() {
    let expected = K2Tree {
      stem_k: 2,
      leaf_k: 2,
      max_slayers: 2,
      slayer_starts: vec![0],
      stems: bitvec![0,0,0,0],
      leaves: bitvec![],
    };
    assert_eq!(K2Tree::new(), expected);
  }
  #[test]
  fn with_k_0() -> Result<()> {
    assert_eq!(K2Tree::with_k(2, 2)?, K2Tree::new());
    Ok(())
  }
  #[test]
  fn with_k_1() -> Result<()> {
    for stem_k in 2..9usize {
      for leaf_k in 2..9usize {
        let expected = K2Tree {
          stem_k,
          leaf_k,
          max_slayers: 2,
          slayer_starts: vec![0],
          stems: bitvec![0; stem_k.pow(2)],
          leaves: BitVec::new(),
        };
        assert_eq!(K2Tree::with_k(stem_k, leaf_k)?, expected);
      }
    }
    Ok(())
  }
  #[test]
  fn set_stem_k_0() {
    let mut tree = K2Tree::new();
    for valid_k in 2..7 {
      assert!(tree.set_stem_k(valid_k).is_ok());
    }
    for invalid_k in 0..2 {
      assert!(tree.set_stem_k(invalid_k).is_err());
    }
  }
  #[test]
  fn set_stem_k_1() {
    let mut tree = K2Tree::test_tree(2);
    assert!(tree.set_stem_k(3).is_ok());
    let expected = K2Tree {
      stem_k: 3,
      leaf_k: 2,
      max_slayers: 2,
      slayer_starts: vec![0, 9],
      stems: bitvec![
        1,1,0,0,0,0,0,0,0, 0,0,1,0,0,0,1,0,1,
        1,0,0,1,0,0,0,0,0
      ],
      leaves: bitvec![
        0,1,1,0, 1,0,0,0, 0,1,1,0,
        0,1,0,1, 1,1,0,0
      ],
    };
    assert_eq!(tree, expected);
  }
  #[test]
  fn set_stem_k_2() {
    let mut tree = K2Tree::test_tree(3);
    assert!(tree.set_stem_k(2).is_ok());
    let expected = K2Tree {
      stem_k: 2,
      leaf_k: 3,
      max_slayers: 4,
      slayer_starts: vec![0, 4, 12, 32],
      stems: bitvec![
        1,0,0,0, 1,1,1,0, 0,1,1,1, 1,0,0,0, 0,0,1,1, //final layer begins here
        0,1,0,0, 0,0,1,0, 0,0,0,1, 1,0,0,0, 1,0,0,0, 0,1,0,0
      ],
      leaves: bitvec![
        0,1,0,1,0,0,0,0,1, 1,0,0,0,0,0,0,0,0,
        0,1,0,1,0,0,0,0,0, 1,0,0,1,0,0,1,0,0,
        1,0,0,0,0,0,0,0,0, 0,1,0,1,0,0,0,0,0
      ],
    };
    assert_eq!(tree, expected);
  }
  #[test]
  fn set_leaf_k_0() {
    let mut tree = K2Tree::new();
    for valid_k in 2..7 {
      assert!(tree.set_leaf_k(valid_k).is_ok());
    }
    for invalid_k in 0..2 {
      assert!(tree.set_leaf_k(invalid_k).is_err());
    }
  }
  #[test]
  fn set_leaf_k_1() {
    let mut tree = K2Tree::test_tree(2);
    assert!(tree.set_leaf_k(3).is_ok());
    let expected = K2Tree {
      stem_k: 2,
      leaf_k: 3,
      max_slayers: 2,
      slayer_starts: vec![0, 4],
      stems: bitvec![
        1,1,0,0, 0,1,1,1, 1,0,0,0,
      ],
      leaves: bitvec![
        0,0,1,0,1,0,0,0,0, 0,0,0,1,0,0,0,0,0,
        0,0,0,0,0,1,0,1,0, 0,1,0,0,1,0,1,1,0
      ],
    };
    assert_eq!(tree, expected);
  }
  #[test]
  fn set_leaf_k_2() {
    let mut tree = K2Tree::test_tree(3);
    assert!(tree.set_leaf_k(2).is_ok());
    let expected = K2Tree {
      stem_k: 3,
      leaf_k: 2,
      max_slayers: 3,
      slayer_starts: vec![0, 9, 27],
      stems: bitvec![
        1,0,0,1,0,0,0,0,0, 0,1,1,1,1,0,0,0,0, 1,1,0,0,0,0,0,0,0, //final layer starts below
        0,1,1,0,0,1,0,0,0, 1,0,0,1,0,0,0,0,0, 0,0,0,1,0,0,0,0,0,
        0,0,0,0,0,1,0,1,0, 1,0,0,0,0,0,0,0,0, 0,1,1,0,0,0,0,0,0
      ],
      leaves: bitvec![
        0,0,0,1, 1,0,0,0, 0,1,0,0, 1,0,1,0, 1,0,0,0,
        0,0,1,0, 0,0,1,0, 0,1,0,0, 1,0,0,0, 0,0,0,1,
        1,0,0,0
      ],
    };
    assert_eq!(tree, expected);
  }
  #[test]
  fn is_empty_0() -> Result<()> {
    for stem_k in 2..10 {
      for leaf_k in 2..10 {
        let tree = K2Tree::with_k(stem_k, leaf_k)?;
        assert!(tree.is_empty());
      }
    }
    Ok(())
  }
  #[test]
  fn is_empty_1() -> Result<()> {
    for stem_k in 2..5 {
      for leaf_k in 2..5 {
        let mut tree = K2Tree::with_k(stem_k, leaf_k)?;
        tree.set(0, 0, true)?;
        assert!(!tree.is_empty());
        tree.set(0, 0, false)?;
        assert!(tree.is_empty());
      }
    }
    Ok(())
  }
  #[test]
  fn get() -> Result<()> {
    for k in 2..4 {
      let tree = K2Tree::test_tree(k);
      let matrix = K2Tree::test_matrix(k);
      for y in 0..matrix.height {
        for x in 0..matrix.width {
          assert_eq!(tree.get(x, y)?, matrix.get(x, y)?);
        }
      }
    }
    Ok(())
  }
  #[test]
  fn get_row() -> Result<()> {
    let tree = K2Tree::test_tree(2);
    let rows = [
      vec![false,false,false,false,false,true,false,true],
      vec![false,false,false,false,true,false,false,true],
      vec![false,false,false,false,false,false,true,true],
      vec![false; 8],
      vec![true,false,false,false,false,true,false,false],
      vec![false,false,false,false,true,false,false,false],
      vec![false; 8],
      vec![false; 8]
    ];
    for i in 0..8 {
      assert_eq!(rows[i], tree.get_row(i)?);
    }
    Ok(())
  }
  #[test]
  fn get_column() -> Result<()> {
    let tree = K2Tree::test_tree(2);
    let cols = [
      vec![false,false,false,false,true,false,false,false],
      vec![false; 8],
      vec![false; 8],
      vec![false; 8],
      vec![false,true,false,false,false,true,false,false],
      vec![true,false,false,false,true,false,false,false],
      vec![false,false,true,false,false,false,false,false],
      vec![true,true,true,false,false,false,false,false],
    ];
    for i in 0..8 {
      assert_eq!(cols[i], tree.get_column(i)?);
    }
    Ok(())
  }
  #[test]
  fn set_0() -> Result<()> {
    let mut tree = K2Tree::with_k(2, 2)?;
    assert_eq!(false, tree.get(0, 0).unwrap());
    tree.set(0, 0, true)?;
    assert_eq!(true, tree.get(0, 0).unwrap());
    tree.set(0, 0, false)?;
    assert_eq!(false, tree.get(0, 0).unwrap());
    assert_eq!(false, tree.get(7, 7).unwrap());
    tree.set(7, 7, true)?;
    assert_eq!(true, tree.get(7, 7).unwrap());
    tree.set(7, 7, false)?;
    assert_eq!(false, tree.get(7, 7).unwrap());
    assert_eq!(false, tree.get(2, 6).unwrap());
    tree.set(2, 6, true)?;
    assert_eq!(true, tree.get(2, 6).unwrap());
    tree.set(6, 2, true)?;
    assert_eq!(true, tree.get(2, 6).unwrap());
    assert_eq!(true, tree.get(6, 2).unwrap());
    Ok(())
  }
  #[test]
  fn set_1() -> Result<()> {
    let mut tree = K2Tree::with_k(2, 2)?;
    tree.grow();
    for i in 0..256 {
      let [x, y] = [i%16, i/16];
      assert_eq!(false, tree.get(x, y)?);
      tree.set(x, y, true)?;
      assert_eq!(true, tree.get(x, y)?);
      tree.set(x, y, false)?;
    }
    Ok(())
  }
  #[test]
  fn set_2() -> Result<()> {
    for k in 2..5 {
      let mut tree = K2Tree::with_k(k, k)?;
      for y in 0..(k.pow(3)) {
        for x in 0..(k.pow(3)) {
          assert_eq!(false, tree.get(x, y)?);
          tree.set(x, y, true)?;
          assert_eq!(true, tree.get(x, y)?);
          tree.set(x, y, false)?;
        }
      }
    }
    Ok(())
  }
  #[test]
  fn set_3() -> Result<()> {
    let mut tree = K2Tree::with_k(3, 3)?;
    tree.grow();
    for i in 0..6561 {
      let [x, y] = [i%81, i/81];
      assert_eq!(false, tree.get(x, y)?);
      tree.set(x, y, true)?;
      assert_eq!(true, tree.get(x, y)?);
      tree.set(x, y, false)?;
    }
    Ok(())
  }
  #[test]
  fn matrix_width_and_grow_0() -> Result<()> {
    for k in 2..9usize {
      let k_cubed = k.pow(3);
      let mut tree = K2Tree::with_k(k, k)?;
      assert_eq!(k_cubed, tree.matrix_width());
      tree.grow();
      assert_eq!(k_cubed*k, tree.matrix_width());
      tree.grow();
      assert_eq!(k_cubed*k*k, tree.matrix_width());
      for _ in 0..3 { tree.grow(); }
      assert_eq!(k_cubed*k.pow(5), tree.matrix_width());
    }
    Ok(())
  }
  #[test]
  fn matrix_width_and_grow_1() -> Result<()> {
    for k in 2..4usize {
      let k_cubed = k.pow(3);
      let mut tree = K2Tree::test_tree(k);
      assert_eq!(k_cubed, tree.matrix_width());
      tree.grow();
      assert_eq!(k_cubed*k, tree.matrix_width());
      for _ in 0..3 { tree.grow(); }
      assert_eq!(k_cubed*k.pow(4), tree.matrix_width());
    }
    Ok(())
  }
  #[test]
  fn stem_k() -> Result<()> {
    for k in 2..9 {
      assert_eq!(k, K2Tree::with_k(k, k)?.stem_k);
    }
    Ok(())
  }
  #[test]
  fn stems_0() {
    let tree = K2Tree::test_tree(2);
    let values = bitvec![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0];
    let stems  = [0, 0, 1, 2];
    let bits   = [0, 1, 2, 3];
    for (i, stem) in tree.stems().enumerate() {
      assert_eq!(
        iterators::StemBit {
          value: values[i],
          layer: if i < 4 { 0 } else { 1 },
          stem: stems[i/4],
          bit: bits[i%4],
        },
        stem
      );
    }
  }
  #[test]
  fn stems_1() {
    let tree = K2Tree::test_tree(3);
    let values = bitvec![
      0,1,0,1,1,0,1,1,0, 1,1,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0,
      1,0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0
    ];
    let stems = [0, 0, 1, 2, 3, 4];
    let bits = [0, 1, 2, 3, 4, 5, 6, 7, 8];
    for (i, stem) in tree.stems().enumerate() {
      assert_eq!(
        iterators::StemBit {
          value: values[i],
          layer: if i < 9 { 0 } else { 1 },
          stem: stems[i/9],
          bit: bits[i%9],
        },
        stem
      );
    }
  }
  #[test]
  fn stems_2() {
    let tree = K2Tree::test_tree(4);
    let values = bitbox![
      1,0,0,1,0,0,0,1,1,0,0,0,1,1,0,1, 1,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,
      0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,
      0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0, 0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,
      1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,
    ];
    let stems = [0,0,1,2,3,4,5,6];
    let bits = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
    for (i, stem) in tree.stems().enumerate() {
      assert_eq!(
        iterators::StemBit {
          value: values[i],
          layer: if i < 16 { 0 } else { 1 },
          stem: stems[i/16],
          bit: bits[i%16],
        },
        stem
      );
    }
  }
  #[test]
  fn stems_raw_0() {
    let tree = K2Tree::test_tree(2);
    let values = bitbox![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0];
    for (i, stem) in tree.stems_raw().enumerate() {
      assert_eq!(values[i], stem);
    }
  }
  #[test]
  fn stems_raw_1() {
    let tree = K2Tree::test_tree(3);
    let values = bitbox![
      0,1,0,1,1,0,1,1,0, 1,1,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0,
      1,0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0,0
    ];
    for (i, stem) in tree.stems_raw().enumerate() {
      assert_eq!(values[i], stem);
    }
  }
  #[test]
  fn stems_raw_2() {
    let tree = K2Tree::test_tree(4);
    let values = bitbox![
      1,0,0,1,0,0,0,1,1,0,0,0,1,1,0,1, 1,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,
      0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,
      0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0, 0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,
      1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,
    ];
    for (i, stem) in tree.stems_raw().enumerate() {
      assert_eq!(values[i], stem);
    }
  }
  #[test]
  fn leaves_0() {
    let tree = K2Tree::test_tree(2);
    let values = bitbox![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0];
    let xs     =        [4,5,4,5, 6,7,6,7, 6,7,6,7, 0,1,0,1, 4,5,4,5];
    let ys     =        [0,0,1,1, 0,0,1,1, 2,2,3,3, 4,4,5,5, 4,4,5,5];
    let leaves =        [0,0,0,0, 1,1,1,1, 2,2,2,2, 3,3,3,3, 4,4,4,4];
    let bits   =        [0,1,2,3, 0,1,2,3, 0,1,2,3, 0,1,2,3, 0,1,2,3];
    for (i, leaf) in tree.leaves().enumerate() {
      assert_eq!(
        iterators::LeafBit {
          value: values[i],
          x: xs[i],
          y: ys[i],
          leaf: leaves[i],
          bit: bits[i],
        },
        leaf
      );
    }
  }
  #[test]
  fn leaves_1() {
    let tree = K2Tree::test_tree(3);
    let values = bitbox![
      0,1,0,1,0,0,0,0,1, 1,0,0,1,0,0,1,0,0, 1,0,0,0,0,0,0,0,0,
      0,1,0,1,0,0,0,0,0, 1,0,0,0,0,0,0,0,0, 0,1,0,1,0,0,0,0,0,
    ];
    let xs = [
      9,10,11,9,10,11,9,10,11, 12,13,14,12,13,14,12,13,14,
      0,1,2,0,1,2,0,1,2, 9,10,11,9,10,11,9,10,11,
      0,1,2,0,1,2,0,1,2, 9,10,11,9,10,11,9,10,11
    ];
    let ys = [
      0,0,0,1,1,1,2,2,2, 0,0,0,1,1,1,2,2,2,
      9,9,9,10,10,10,11,11,11, 9,9,9,10,10,10,11,11,11,
      18,18,18,19,19,19,20,20,20, 18,18,18,19,19,19,20,20,20
    ];
    let leaves = [
      0,0,0,0,0,0,0,0,0, 1,1,1,1,1,1,1,1,1, 2,2,2,2,2,2,2,2,2,
      3,3,3,3,3,3,3,3,3, 4,4,4,4,4,4,4,4,4, 5,5,5,5,5,5,5,5,5,
    ];
    let bits = [
      0,1,2,3,4,5,6,7,8, 0,1,2,3,4,5,6,7,8, 0,1,2,3,4,5,6,7,8,
      0,1,2,3,4,5,6,7,8, 0,1,2,3,4,5,6,7,8, 0,1,2,3,4,5,6,7,8
    ];
    for (i, leaf) in tree.leaves().enumerate() {
      assert_eq!(
        iterators::LeafBit {
          value: values[i],
          x: xs[i],
          y: ys[i],
          leaf: leaves[i],
          bit: bits[i],
        },
        leaf
      );
    }
  }
  #[test]
  fn leaves_raw_0() {
    let tree = K2Tree::test_tree(2);
    let values = bitbox![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0];
    for (i, leaf) in tree.leaves_raw().enumerate() {
      assert_eq!(values[i], leaf);
    }
  }
  #[test]
  fn leaves_raw_1() {
    let tree = K2Tree::test_tree(3);
    let values = bitbox![
      0,1,0,1,0,0,0,0,1, 1,0,0,1,0,0,1,0,0, 1,0,0,0,0,0,0,0,0,
      0,1,0,1,0,0,0,0,0, 1,0,0,0,0,0,0,0,0, 0,1,0,1,0,0,0,0,0,
    ];
    for (i, leaf) in tree.leaves_raw().enumerate() {
      assert_eq!(values[i], leaf);
    }
  }
  #[test]
  fn shrink_if_possible() -> Result<()> {
    for k in 2..9usize {
      let mw = k.pow(3);
      let mut tree = K2Tree::with_k(k, k)?;
      tree.grow();
      assert_eq!(mw*k, tree.matrix_width());
      tree.shrink_if_possible();
      assert_eq!(mw, tree.matrix_width());
      tree.shrink_if_possible();
      assert_eq!(mw, tree.matrix_width());
    }
    Ok(())
  }
  #[test]
  fn shrink() -> Result<()> {
    for k in 2..9usize {
      let mw = k.pow(3);
      let mut tree = K2Tree::with_k(k, k)?;
      tree.grow();
      assert_eq!(mw*k, tree.matrix_width());
      assert!(tree.shrink().is_ok());
      assert_eq!(mw, tree.matrix_width());
      assert!(tree.shrink().is_err());
      assert_eq!(mw, tree.matrix_width());
    }
    Ok(())
  }
  #[test]
  fn shrink_unchecked() -> Result<()> {
    for k in 2..9usize {
      let mut tree = K2Tree::with_k(k, k)?;
      tree.grow();
      assert_eq!(k.pow(4), tree.matrix_width());
      unsafe { tree.shrink_unchecked(); }
      assert_eq!(k.pow(3), tree.matrix_width());
    }
    Ok(())
  }
  #[test]
  fn from_matrix() -> Result<()> {
    for k in 2..=3usize {
      let matrix = K2Tree::test_matrix(k);
      let tree = K2Tree::test_tree(k);
      assert_eq!(tree, K2Tree::from_matrix(matrix, k, k)?);
    }
    Ok(())
  }
  #[test]
  fn to_matrix() -> Result<()> {
    for k in 2..=3usize {
      let tree = K2Tree::test_tree(k);
      let matrix = K2Tree::test_matrix(k);
      assert_eq!(matrix, tree.to_matrix()?);
      assert_eq!(matrix, K2Tree::from_matrix(matrix.clone(), k, k)?.to_matrix()?);
    }
    Ok(())
  }
  #[test]
  fn into_matrix() -> Result<()> {
    for k in 2..=3usize {
      let tree = K2Tree::test_tree(k);
      let matrix = K2Tree::test_matrix(k);
      assert_eq!(matrix, tree.into_matrix()?);
      assert_eq!(matrix, K2Tree::from_matrix(matrix.clone(), k, k)?.into_matrix()?);
    }
    Ok(())
  }
}

#[cfg(test)]
mod util {
  use super::*;
  #[test]
  fn all_zeroes_0() {
    let zeroes = bitvec![0,0,0,0, 0,0,0,0, 0,0];
    let one    = bitvec![0,0,0,0, 0,1,0,0, 0];
    let ones   = bitvec![1,1,1,1, 1];
    let edge   = bitvec![0,0,0,1];
    assert!(all_zeroes(&zeroes, 0, 10));
    assert!(!all_zeroes(&one, 0, 9));
    assert!(!all_zeroes(&ones, 0, 5));
    assert!(!all_zeroes(&edge, 0, 4));
  }
  #[test]
  fn one_positions_0() {
    let bv = bitvec![0,1,0,1,0,1,0,0,0,1];
    assert_eq!(vec![1,3,5,9], one_positions(bv.into_iter()));
  }
  #[test]
  fn ones_in_range_0() {
    let ranges = [
      bitvec![0,1,1,1,0,0,1,0,1,1,0,0],
      bitvec![0,0,0,0,0,0,1],
      bitvec![0,1,1,1,1,1,1,0,1,0,0,1]
    ];
    let num_ones = [6, 1, 8];
    for i in 0..ranges.len() {
      assert_eq!(ones_in_range(&ranges[i], 0, ranges[i].len()), num_ones[i]);
    }
  }
  #[test]
  fn stem_layer_start_0() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(tree.layer_start(0), 0);
    assert_eq!(tree.layer_start(1), 4);
  }
  #[test]
  fn stem_layer_start_1() {
    let tree = K2Tree::test_tree(3);
    assert_eq!(tree.layer_start(0), 0);
    assert_eq!(tree.layer_start(1), 9);
  }
  #[test]
  fn stem_layer_start_2() {
    let tree = K2Tree::test_tree(4);
    assert_eq!(tree.layer_start(0), 0);
    assert_eq!(tree.layer_start(1), 16);
  }
  #[test]
  fn stem_layer_len_0() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(tree.layer_len(0), 4);
    assert_eq!(tree.layer_len(1), 12);
  }
  #[test]
  fn stem_layer_len_1() {
    let tree = K2Tree::test_tree(3);
    assert_eq!(tree.layer_len(0), 9);
    assert_eq!(tree.layer_len(1), 45);
  }
  #[test]
  fn stem_layer_len_2() {
    let tree = K2Tree::test_tree(4);
    assert_eq!(tree.layer_len(0), 16);
    assert_eq!(tree.layer_len(1), 112);
  }
  #[test]
  fn stem_to_leaf_start_0() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(tree.stem_to_leaf_start(4), Ok(0));
    assert_eq!(tree.stem_to_leaf_start(5), Ok(4));
    assert_eq!(tree.stem_to_leaf_start(7), Ok(8));
    assert_eq!(tree.stem_to_leaf_start(8), Ok(12));
    assert_eq!(tree.stem_to_leaf_start(12), Ok(16));
    assert_eq!(tree.stem_to_leaf_start(9), Err(()));
  }
  #[test]
  fn child_stem_0() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(tree.child_stem(0, 0, 0), Err(()));
    assert_eq!(tree.child_stem(0, 0, 1), Ok(4));
    assert_eq!(tree.child_stem(0, 0, 2), Ok(8));
    assert_eq!(tree.child_stem(0, 0, 3), Ok(12));
    assert_eq!(tree.child_stem(1, 4, 0), Err(()));
  }
  #[test]
  fn parent_stem_0() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(tree.parent_stem(4), 0);
    assert_eq!(tree.parent_stem(8), 0);
    assert_eq!(tree.parent_stem(12), 0);
  }
  #[test]
  fn parent_bit_0() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(tree.parent_bit(4), 1);
    assert_eq!(tree.parent_bit(8), 2);
    assert_eq!(tree.parent_bit(12), 3);
  }
  #[test]
  fn get_coords_0() {
    let tree = K2Tree::test_tree(2);
    assert_eq!(tree.get_coords(12), [0, 4]);
  }
}

#[cfg(test)]
mod misc {
  use super::*;
  #[test]
  fn flood() -> Result<()> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut tree = K2Tree::with_k(2, 2)?;
    for _ in 0..10 { tree.grow(); }
    for _ in 0..500 {
      let x: usize = rng.gen_range(0, 512);
      let y: usize = rng.gen_range(0, 512);
      tree.set(x, y, true)?;
    }
    Ok(())
  }
  #[test]
  fn is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<K2Tree>();
  }
  #[test]
  fn is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<K2Tree>();
  }
  #[test]
  fn display() {
    println!("{}", K2Tree::test_tree(3));
  }
}

#[cfg(test)]
mod many_k {
  use super::*;
  #[test]
  fn build() -> Result<()> {
    for i in 2..=3 {
      for stem_k in 2..9 {
        for leaf_k in 2..9 {
          let m = K2Tree::test_matrix(i);
          let mut tree = K2Tree::with_k(stem_k, leaf_k)?;
          while tree.matrix_width() < m.width { tree.grow(); }
          for (y, row) in m.into_rows().into_iter().enumerate() {
            for (x, state) in row.into_iter().enumerate() {
              tree.set(x, y, state)?;
            }
          }
        }
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod layer_tests {
  use super::*;
  #[test]
  fn layer_start_0() {
    let trees = [
      K2Tree::test_tree(2),
      K2Tree::test_tree(3),
      K2Tree::test_tree(4)
    ];
    let expecteds = [
      [0, 4],
      [0, 9],
      [0, 16]
    ];
    for k in 0..trees.len() {
      for layer in 0..expecteds[k].len() {
        assert_eq!(expecteds[k][layer], trees[k].layer_start(layer));
      }
    }
  }
  #[test]
  fn layer_start_1() {
    let mut tree = K2Tree::test_tree(3);
    for _ in 0..9 { tree.grow(); }
    tree.set(67, 78, true);
    tree.set(100, 100, true);
    tree.set(33, 146, true);
    tree.set(43, 146, true);
    dbg!(&tree);
    let expected = [0, 9, 18, 27, 36, 45, 54, 63, 72, 99, 135];
    for layer in 0..tree.max_slayers {
      assert_eq!(expected[layer], tree.layer_start(layer));
    }
  }
}