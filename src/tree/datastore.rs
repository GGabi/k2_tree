use {
  bitvec::{prelude::{bitvec, bitbox, BitVec}},
  crate::error::K2TreeError as Error,
  crate::tree::*,
  crate::matrix::BitMatrix,
};

type Result<T> = std::result::Result<T, Error>;

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
///   let mut tree = K2Tree::new();
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
  /// The width of the matrix this K2Tree represents. The matrix is always square,
  /// so this is also the height.
  pub matrix_width: usize,
  /// The k value of the K2Tree, currently fixed at 2.
  pub k: usize,
  /// The maximum number of stem-layers possible given the matrix_width.
  pub max_slayers: usize,
  /// The index of the first bit in each stem-layer in stems.
  pub slayer_starts: Vec<usize>,
  /// The bits that comprise the stems of the tree. 
  pub stems: BitVec,
  /// Layer that links the positive bits in the final stem-layer.
  /// 
  /// The value of each element is the offset of a positive stem-bit
  /// relative to the the start of the final stem-layer. The index
  /// of each element corresponds to the position of the leaf-block
  /// it links to.
  pub stem_to_leaf: Vec<usize>,
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
  /// assert_eq!(8, tree.matrix_width);
  /// assert_eq!(2, tree.k);
  /// ```
  pub fn new() -> Self {
    /* For now fix k as 2, further work to make it user-defined */
    let k: usize = 2;
    let mw = k.pow(3);
    K2Tree {
      matrix_width: mw,
      k,
      max_slayers: (mw as f64).log(k as f64) as usize - 1,
      slayer_starts: vec![0],
      stems: bitvec![0; k*k],
      stem_to_leaf: Vec::new(),
      leaves: BitVec::new(),
    }
  }
  ///Returns true if a `K2Tree` contains no 1s
  pub fn is_empty(&self) -> bool {
    ones_in_range(&self.leaves, 0, self.leaves.len()) == 0
  }
  /// Returns that state of a bit at a specified coordinate in the bit-matrix the
  /// `K2Tree` represents.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::new();
  ///   tree.set(0, 1, true)?;
  ///   assert_eq!(true, tree.get(0, 1)?);
  ///   assert_eq!(false, tree.get(0, 0)?);
  ///   Ok(())
  /// }
  /// ```
  pub fn get(&self, x: usize, y: usize) -> Result<bool> {
    if x >= self.matrix_width || y >= self.matrix_width {
      return Err(Error::Read {
        source: Box::new(Error::OutOfBounds {
          x_y: [x, y],
          min_x_y: [0, 0],
          max_x_y: [self.matrix_width-1; 2]
        })
      })
    }
    /* Assuming k=2 */
    let descend_result = match self.matrix_bit(x, y, self.matrix_width) {
      Ok(dr) => dr,
      Err(e) => return Err(Error::Read {
        source: Box::new(e)
      }),
    };
    match descend_result {
      DescendResult::Leaf(leaf_start, leaf_range) => {
        if leaf_range[0][1] - leaf_range[0][0] != 1
        || leaf_range[1][1] - leaf_range[1][0] != 1 {
          return Err(Error::Read {
            source: Box::new(Error::TraverseError{x, y})
          })
        }
        //Calculation removes extra branches, makes it faster
        // range = [[5, 6], [7, 8]]
        // (5, 7) = 0; (6, 7) = 1; (5, 8) = 2; (6, 8) = 3
        let offset = (2 * (y - leaf_range[1][0])) + (x - leaf_range[0][0]);
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
  ///   let mut tree = K2Tree::new();
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
    if y >= self.matrix_width {
      return Err(Error::Read {
        source: Box::new(Error::OutOfBounds {
          x_y: [0, y],
          min_x_y: [0, 0],
          max_x_y: [self.matrix_width-1; 2]
        })
      })
    }
    let mut ret_v = Vec::new();
    for x in (0..self.matrix_width).step_by(self.k) {
      match self.matrix_bit(x, y, self.matrix_width)? {
        DescendResult::Leaf(leaf_start, leaf_range) => {
          if leaf_range[0][1] - leaf_range[0][0] != 1
          || leaf_range[1][1] - leaf_range[1][0] != 1 {
            return Err(Error::Read {
              source: Box::new(Error::TraverseError{x, y})
            })
          }
          //Calculation instead of if-else block makes hot-code much faster
          let offset = (2 * (y - leaf_range[1][0])) + (x - leaf_range[0][0]);
          for i in 0..self.k { ret_v.push(self.leaves[leaf_start+offset+i]); }
        },
        DescendResult::Stem(_, _) => {
          for _ in 0..self.k { ret_v.push(false); }
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
  ///   let mut tree = K2Tree::new();
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
    if x >= self.matrix_width {
      return Err(Error::Read {
        source: Box::new(Error::OutOfBounds {
          x_y: [x, 0],
          min_x_y: [0, 0],
          max_x_y: [self.matrix_width-1; 2]
        })
      })
    }
    let mut ret_v = Vec::new();
    for y in (0..self.matrix_width).step_by(self.k) {
      match self.matrix_bit(x, y, self.matrix_width)? {
        DescendResult::Leaf(leaf_start, leaf_range) => {
          if leaf_range[0][1] - leaf_range[0][0] != 1
          || leaf_range[1][1] - leaf_range[1][0] != 1 {
            return Err(Error::Read {
              source: Box::new(Error::TraverseError{x, y})
            })
          }
          let offset = (2 * (y - leaf_range[1][0])) + (x - leaf_range[0][0]);
          for i in 0..self.k { ret_v.push(self.leaves[leaf_start+offset+(i*self.k)]); }
        },
        DescendResult::Stem(_, _) => {
          for _ in 0..self.k { ret_v.push(false); }
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
  ///   let mut tree = K2Tree::new();
  ///   assert_eq!(false, tree.get(0, 0)?);
  ///   tree.set(0, 0, true)?;
  ///   assert_eq!(true, tree.get(0, 0)?);
  ///   Ok(())
  /// }
  /// ```
  pub fn set(&mut self, x: usize, y: usize, state: bool) -> Result<()> {
    /* Assuming k=2 */
    let descend_result = match self.matrix_bit(x, y, self.matrix_width) {
      Ok(dr) => dr,
      Err(e) => return Err(Error::Write {
        source: Box::new(e)
      }),
    };
    match descend_result {
      DescendResult::Leaf(leaf_start, leaf_range) => {
        if leaf_range[0][1] - leaf_range[0][0] != 1
        || leaf_range[1][1] - leaf_range[1][0] != 1 {
          /* Final submatrix isn't a 2 by 2 so can't be a leaf */
          return Err(Error::Write {
            source: Box::new(Error::TraverseError{x, y})
          })
        }
        /* Set the bit in the leaf to the new state */
        let offset = (2 * (y - leaf_range[1][0])) + (x - leaf_range[0][0]);
        self.leaves.set(leaf_start+offset, state);
        /* If leaf is now all 0's, remove leaf and alter rest of struct to reflect changes.
        Loop up the stems changing the parent bits to 0's and removing stems that become all 0's */
        if !state && all_zeroes(&self.leaves, leaf_start, leaf_start+4) {
          /* - Remove the leaf
              - Use stem_to_leaf to find the dead leaf's parent bit
              - Remove the elem from stem_to_leaf that mapped to dead leaf
              - Set parent bit to 0, check if stem now all 0's
              - If all 0's:
              - - Remove stem
              - - Alter layer_starts if needed
              - - Find parent bit and set to 0
              - - Repeat until reach stem that isn't all 0's or reach stem layer 0 */
          if let Err(()) = remove_block(&mut self.leaves, leaf_start) {
            return Err(Error::CorruptedK2Tree {
              source: Box::new(Error::Write {
                source: Box::new(Error::LeafRemovalError {
                  pos: leaf_start,
                  len: 4
                })
              })
            })
          }
          let stem_bit_pos = self.stem_to_leaf[leaf_start/4];
          self.stem_to_leaf.remove(leaf_start/4);
          if self.stem_to_leaf.is_empty() {
            /* If no more leaves, then remove all stems immediately
            and don't bother with complex stuff below */
            self.stems = bitvec![0,0,0,0];
            self.slayer_starts = vec![0];
            return Ok(())
          }
          let layer_start = self.slayer_starts[self.max_slayers-1];
          self.stems.set(layer_start + stem_bit_pos, false); //Dead leaf parent bit = 0
          let mut curr_layer = self.max_slayers-1;
          let mut stem_start = layer_start + block_start(stem_bit_pos);
          while curr_layer > 0
          && all_zeroes(&self.stems, stem_start, stem_start+4) {
            if curr_layer == self.max_slayers-1 {
              for stem_to_leaf_bit in &mut self.stem_to_leaf[leaf_start/4..] {
                *stem_to_leaf_bit -= 4;
              }
            }
            for layer_start in &mut self.slayer_starts[curr_layer+1..] {
              // NOTE: this was 1 but it looks like that was an uncaught error, changed to 4
              //       which is stem-length but if any errors, look here.
              *layer_start -= 4; //Adjust lower layer start positions to reflect removal of stem
            }
            let (parent_stem_start, bit_offset) = self.parent(stem_start);
            if let Err(()) = remove_block(&mut self.stems, stem_start) {
              return  Err(Error::CorruptedK2Tree {
                source: Box::new(Error::Write {
                  source: Box::new(Error::StemRemovalError {
                    pos: stem_start,
                    len: 4
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
        let mut layer_starts_len = self.slayer_starts.len();
        let mut layer = self.layer_from_range(stem_range);
        let mut subranges: [Range; 4];
        /* Keep track of whether lowest stem is freshly created (all 0000s) */
        let mut fresh_stem = false;
        /* Create correct stems in layers on the way down to the final layer,
        which points to the leaves */
        while layer < self.max_slayers-1 {
          fresh_stem = true;
          subranges = to_4_subranges(stem_range);
          let (child_pos, &subrange) =
            match subranges.iter().enumerate().find(
              |(_, subrange)| within_range(subrange, x, y)
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
          if let Err(()) = insert_block(&mut self.stems, stem_start) {
            return Err(Error::CorruptedK2Tree {
              source: Box::new(Error::Write {
                source: Box::new(Error::StemInsertionError {
                  pos: stem_start,
                  len: 4
                })
              })
            })
          }
          /* If there are layers after the one we just insert a stem
          into: Increase the layer_starts for them by 4 to account for
          the extra stem */
          for layer_start in &mut self.slayer_starts[layer+1..] {
            *layer_start += 4;
          }
        }
        /* We're at the final stem layer */
        subranges = to_4_subranges(stem_range);
        let (child_pos, &subrange) =
          match subranges.iter().enumerate().find(
            |(_, subrange)| within_range(subrange, x, y)
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
        /* Get the bit position within the final stem layer,
        find the position in stem_to_leaf to insert the linking elem,
        insert linking elem */
        let layer_bit_pos = (stem_start + child_pos) - self.slayer_starts[layer_starts_len-1];
        /* If stem is fresh, increase bit positions in stem_to_leaf
        after the new elem by 4 to account for the new stem before them */
        if fresh_stem {
          /* Warning:
            Before inserting this block, a subtle and infrequent problem was occuring:
              If final stem layer == [0010] then stem_to_leaf == [2]
              If a fresh stem inserted making it [0001 0010] then:
                New offset inserted to stem_to_leaf was 3, which was greater than 2.
                The code thought that the bit corresponding to the 2 was before the
                new stem, so wouldn't update offset correctly.
              Now we update stem_to_leaf offsets BEFORE inserting new value
              AND update offsets greater than the block_start of the new stem.
          */
          let block_start = (layer_bit_pos / 4) * 4;
          self.stem_to_leaf = self.stem_to_leaf.iter().map(
            |&n|
              if n >= block_start { n+4 }
              else { n }
          ).collect();
        }
        let mut stem_to_leaf_pos: usize = 0;
        while stem_to_leaf_pos < self.stem_to_leaf.len()
        && self.stem_to_leaf[stem_to_leaf_pos] < layer_bit_pos {
          stem_to_leaf_pos += 1;
        }
        self.stem_to_leaf.insert(stem_to_leaf_pos, layer_bit_pos);
        /* Create new leaf of all 0's */
        let leaf_start = stem_to_leaf_pos * 4;
        if let Err(()) = insert_block(&mut self.leaves, leaf_start) {
          return Err(Error::CorruptedK2Tree {
            source: Box::new(Error::Write {
              source: Box::new(Error::LeafInsertionError {
                pos: leaf_start,
                len: 4
              })
            })
          })
        }
        /* Change bit at (x, y) to 1 */
        let leaf_range = subrange;
        let offset = (2 * (y - leaf_range[1][0])) + (x - leaf_range[0][0]);
        self.leaves.set(leaf_start+offset, true);
        return Ok(())
      }
      _ => {},
    };
    Ok(())
  }
  /// Returns an iterator over the K2Tree's stems which produces instances of StemBit.
  /// 
  /// StemBit contains extra information on the layer, block and offset of the specific
  /// bit in the stems.
  pub fn stems(&self) -> iterators::Stems<'_> {
    iterators::Stems::new(self)
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
  /// use k2_tree::K2Tree;
  /// let mut tree = K2Tree::new();
  /// assert_eq!(2, tree.k);
  /// assert_eq!(8, tree.matrix_width);
  /// tree.grow();
  /// assert_eq!(16, tree.matrix_width);
  /// tree.grow();
  /// assert_eq!(32, tree.matrix_width);
  /// ```
  pub fn grow(&mut self) {
    self.matrix_width *= self.k;
    self.max_slayers += 1;
    if self.leaves.len() > 0  {
      /* Only insert the extra layers etc. if the
      tree isn't all 0s */
      for slayer_start in &mut self.slayer_starts {
        *slayer_start += 4;
      }
      self.slayer_starts.insert(0, 0);
      /* Insert 1000 to beginning of stems */
      for _ in 0..3 { self.stems.insert(0, false); }
      self.stems.insert(0, true);
    }
  }
  /// Only shrinks the height and width of the matrix the K2Tree represents by a factor of k
  /// if it is possible.
  /// 
  /// Does not Err if the matrix cannot be shrunk i.e. it is already at the minimum size.
  /// ```
  /// use k2_tree::K2Tree;
  /// let mut tree = K2Tree::new();
  /// tree.grow();
  /// assert_eq!(16, tree.matrix_width);
  /// tree.shrink_if_possible();
  /// assert_eq!(8, tree.matrix_width);
  /// tree.shrink_if_possible();
  /// assert_eq!(8, tree.matrix_width);
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
  /// use k2_tree::K2Tree;
  /// let mut tree = K2Tree::new();
  /// tree.grow();
  /// assert_eq!(16, tree.matrix_width);
  /// assert!(tree.shrink().is_ok());
  /// assert_eq!(8, tree.matrix_width);
  /// assert!(tree.shrink().is_err());
  /// ```
  pub fn shrink(&mut self) -> Result<()> {
    if self.matrix_width <= self.k.pow(3) {
      return Err(Error::CouldNotShrink {
        reason: format!("Already at minimum size: {}", self.matrix_width)
      })
    }
    else if self.stems[1..4] != bitbox![0,0,0] {
      return Err(Error::CouldNotShrink {
        reason: "Shrinking would lose information about the matrix".into()
      })
    }
    self.matrix_width /= self.k;
    self.max_slayers -= 1;
    self.slayer_starts.remove(0);
    for slayer_start in &mut self.slayer_starts {
      *slayer_start -= 4;
    }
    /* Remove top layer stem */
    for _ in 0..4 { self.stems.remove(0); }
    Ok(())
  }
  /// Reduces the height and width of the matrix the K2Tree represents by a factor of k without
  /// doing any bounds checking before or integrity checking afterwards.
  /// 
  /// # Safety
  /// Do not attempt to shrink matrix_width smaller than k^3.
  /// 
  /// ```
  /// use k2_tree::K2Tree;
  /// let mut tree = K2Tree::new();
  /// tree.grow();
  /// assert_eq!(16, tree.matrix_width);
  /// unsafe { tree.shrink_unchecked(); }
  /// assert_eq!(8, tree.matrix_width);
  /// ```
  pub unsafe fn shrink_unchecked(&mut self) {
    self.matrix_width /= self.k;
    self.max_slayers -= 1;
    self.slayer_starts.remove(0);
    for slayer_start in &mut self.slayer_starts {
      *slayer_start -= 4;
    }
    /* Remove top layer stem */
    for _ in 0..4 { self.stems.remove(0); }
  }
  /// Comsumes the K2Tree to produce the bit-matrix it represented.
  /// 
  /// The matrix is presented as a list of columns of bits, Vec<Vec<bool>>.
  /// ```
  /// fn main() -> Result<(), k2_tree::error::K2TreeError> {
  ///   use k2_tree::K2Tree;
  ///   let mut tree = K2Tree::new();
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
    let mut m = BitMatrix::with_dimensions(self.matrix_width, self.matrix_width);
    for y in 0..self.matrix_width {
      for x in 0..self.matrix_width {
        if let Err(e) = m.set(x, y, self.get(x, y)?) {
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
  ///   let mut tree = K2Tree::new();
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
    let mut m = BitMatrix::with_dimensions(self.matrix_width, self.matrix_width);
    for y in 0..self.matrix_width {
      for x in 0..self.matrix_width {
        if let Err(e) = m.set(x, y, self.get(x, y)?) {
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
  /// let t = K2Tree::new();
  /// let mut m = BitMatrix::with_dimensions(8, 8);
  /// m.set(0, 5, true);
  /// assert!(K2Tree::from_matrix(m).is_ok());
  /// ```
  pub fn from_matrix(matrix: BitMatrix) -> Result<Self> {
    let mut tree = K2Tree::new();
    while matrix.width > tree.matrix_width
    || matrix.height > tree.matrix_width {
      tree.grow();
    }
    let rows = matrix.into_rows();
    for (y, row) in rows.into_iter().enumerate() {
      for (x, state) in row.into_iter().enumerate() {
        tree.set(x, y, state)?;
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
    for layer_num in 0..self.slayer_starts.len() {
      for bit_pos in self.layer_start(layer_num)..self.layer_start(layer_num+1) {
        if self.stems[bit_pos] { s.push('1'); }
        else { s.push('0'); }
        if i == self.k*self.k
        && (bit_pos - self.layer_start(layer_num)) < self.layer_len(layer_num)-1 {
          s.push(',');
          i = 1;
        } 
        else { i += 1; }
      }
      i = 1;
      s.push_str("::");
    }
    i = 1;
    for bit_pos in 0..self.leaves.len() {
      if self.leaves[bit_pos] { s.push('1'); }
      else { s.push('0'); }
      if i == self.k*self.k
      && bit_pos < self.leaves.len()-1 {
        s.push(',');
        i = 1;
      } 
      else { i += 1; }
    }
    write!(f, "[{}]", s)
  }
}
impl PartialEq for K2Tree {
  fn eq(&self, other: &Self) -> bool {
    self.k == other.k
    && self.matrix_width == other.matrix_width
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
    self.k.hash(state);
    self.matrix_width.hash(state);
    self.stems.hash(state);
    self.leaves.hash(state);
  }
}
impl std::convert::TryFrom<BitMatrix> for K2Tree {
  type Error = Error;
  fn try_from(matrix: BitMatrix) -> Result<Self> {
    //error checking needed
    Self::from_matrix(matrix)
  }
}

/* Private */
enum DescendResult {
  Leaf(usize, Range), //leaf_start, leaf_range
  Stem(usize, Range), //stem_start, stem_range
}
struct DescendEnv {
  /* Allows for descend to be recursive without parameter hell */
  x: usize,
  y: usize,
  slayer_max: usize,
}
impl K2Tree {
  fn layer_from_range(&self, r: Range) -> usize {
    let r_width = r[0][1]-r[0][0]+1;
    ((self.matrix_width as f64).log(self.k as f64) as usize)
    - ((r_width as f64).log(self.k as f64) as usize)
  }
  fn matrix_bit(&self, x: usize, y: usize, m_width: usize) -> Result<DescendResult> {
    let env = DescendEnv {
      x,
      y,
      slayer_max: self.max_slayers-1,
    };
    self.descend(&env, 0, 0, [[0, m_width-1], [0, m_width-1]])
  }
  fn descend(&self, env: &DescendEnv, layer: usize, stem_pos: usize, range: Range) -> Result<DescendResult> {
    // TODO: Completely remove all uses of old Range type, convert to Range2D
    //       Make generic over any K value, not just 2
    let range2d = Range2D::from_range(range);
    let subranges2d = self.to_subranges(range2d).unwrap();
    for (child_pos, child) in self.stems[stem_pos..stem_pos+self.block_len()].iter().enumerate() {
      if subranges2d.subranges[child_pos].contains(env.x, env.y) {
      // if within_range(&subranges[child_pos], env.x, env.y) {
        if !child { return Ok(DescendResult::Stem(stem_pos, range)) } //The bit exists within a range that has all zeros
        else if layer == env.slayer_max {
          let leaf_start = match self.leaf_start(stem_pos + child_pos) {
            Ok(ls) => ls,
            Err(_) => return Err(Error::TraverseError {
              x: env.x,
              y: env.y
            }),
          };
          return Ok(DescendResult::Leaf(leaf_start, subranges2d.subranges[child_pos].to_range()))
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
                              subranges2d.subranges[child_pos].to_range())
        }
      }
    }
    unreachable!()
  }
  fn num_stems_before_child(&self, bit_pos: usize, layer: usize) -> usize {
    ones_in_range(&self.stems, self.layer_start(layer), bit_pos)
  }
  fn leaf_start(&self, stem_bitpos: usize) -> std::result::Result<usize, ()> {
    if !self.stems[stem_bitpos] { return Err(()) }
    if let Some(leaf_num) = self.stem_to_leaf.iter().position(|&n|
      n == (stem_bitpos - self.slayer_starts[self.max_slayers-1])
    ) {
      return Ok(leaf_num * 4)
    }
    Err(())
  }
  fn child_stem(&self, layer: usize, stem_start: usize, nth_child: usize) -> std::result::Result<usize, ()> {
    if !self.stems[stem_start+nth_child]
    || layer == self.max_slayers-1 {
      /* If stem_bit is 0 or final stem layer, cannot have children */
      return Err(())
    }
    Ok(self.layer_start(layer+1)
    + (self.num_stems_before_child(stem_start+nth_child, layer) * 4))
  }
}

/* Private funcs used in testing */
#[cfg(test)]
impl K2Tree {
  fn test_tree() -> Self {
    K2Tree {
      matrix_width: 8,
      k: 2,
      max_slayers: 2,
      slayer_starts: vec![0, 4],
      stems:  bitvec![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0],
      stem_to_leaf: vec![0, 1, 3, 4, 8],
      leaves: bitvec![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0],
    }
  }
  fn parent_stem(&self, stem_start: usize) -> usize {
    self.parent(stem_start).0
  }
  fn parent_bit(&self, stem_start: usize) -> usize {
    let (stem_start, bit_offset) = self.parent(stem_start);
    stem_start + bit_offset
  }
  #[allow(dead_code)]
  fn footprint(&self) -> usize {
    let mut size: usize = std::mem::size_of_val(self);
    size += std::mem::size_of::<usize>() * self.slayer_starts.len();
    size += self.stems.len() / 8;
    size += std::mem::size_of::<usize>() * self.stem_to_leaf.len();
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
      matrix_width: 8,
      k: 2,
      max_slayers: 2,
      slayer_starts: vec![0],
      stems: bitvec![0,0,0,0],
      stem_to_leaf: vec![],
      leaves: bitvec![],
    };
    assert_eq!(K2Tree::new(), expected);
  }  
  #[test]
  fn is_empty_0() {
    let tree = K2Tree::new();
    assert!(tree.is_empty());
  }
  #[test]
  fn is_empty_1() -> Result<()> {
    let mut tree = K2Tree::new();
    tree.set(0, 0, true)?;
    assert!(!tree.is_empty());
    tree.set(0, 0, false)?;
    assert!(tree.is_empty());
    Ok(())
  }
  #[test]
  fn get() -> Result<()> {
    let tree = K2Tree::test_tree();
    let quad_1 = bitvec![0; 16];
    let quad_2 = bitvec![0,1,0,1, 1,0,0,1, 0,0,1,1, 0,0,0,0];
    let quad_3 = {
      let mut q3 = bitvec![0; 15];
      q3.insert(0, true);
      q3
    };
    let quad_4 = {
      let mut q4 = bitvec![0; 14];
      q4.insert(1, true);
      q4.insert(4, true);
      q4
    };
    for i in 0..16 {
      let [x, y] = [i%4, i/4];
      assert_eq!(quad_1[i], tree.get(x, y)?);
    };
    for i in 0..16 {
      let [x, y] = [4+i%4, i/4];
      assert_eq!(quad_2[i], tree.get(x, y)?);
    };
    for i in 0..16 {
      let [x, y] = [i%4, 4+i/4];
      assert_eq!(quad_3[i], tree.get(x, y)?);
    };
    for i in 0..16 {
      let [x, y] = [4+i%4, 4+i/4];
      assert_eq!(quad_4[i], tree.get(x, y)?);
    };
    Ok(())
  }
  #[test]
  fn get_row() -> Result<()> {
    let tree = K2Tree::test_tree();
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
    let tree = K2Tree::test_tree();
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
    let mut tree = K2Tree::new();
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
    let mut tree = K2Tree::new();
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
  fn matrix_width_and_grow() {
    assert_eq!(8, K2Tree::new().matrix_width);
    assert_eq!(8, K2Tree::test_tree().matrix_width);
    let mut grown_tree = K2Tree::new(); grown_tree.grow();
    assert_eq!(16, grown_tree.matrix_width);
    grown_tree.grow();
    assert_eq!(32, grown_tree.matrix_width);
    for _ in 0..3 { grown_tree.grow(); }
    assert_eq!(256, grown_tree.matrix_width);
  }
  #[test]
  fn k() {
    assert_eq!(2, K2Tree::new().k);
    assert_eq!(2, K2Tree::test_tree().k);
  }
  #[test]
  fn stems() {
    let tree = K2Tree::test_tree();
    let values = bitvec![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0];
    let stems  = [0, 0, 1, 2];
    let bits   = [0, 1, 2, 3];
    for (i, stem) in tree.stems().enumerate() {
      assert_eq!(
        iterators::StemBit{
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
  fn stems_raw() {
    let tree = K2Tree::test_tree();
    let values = bitbox![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0];
    for (i, stem) in tree.stems_raw().enumerate() {
      assert_eq!(values[i], stem);
    }
  }
  #[test]
  fn leaves() {
    let tree = K2Tree::test_tree();
    let values = bitbox![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0];
    let xs     =        [4,5,4,5, 6,7,6,7, 6,7,6,7, 0,1,0,1, 4,5,4,5];
    let ys     =        [0,0,1,1, 0,0,1,1, 2,2,3,3, 4,4,5,5, 4,4,5,5];
    for (i, leaf) in tree.leaves().enumerate() {
      assert_eq!(
        iterators::LeafBit {
          value: values[i],
          x: xs[i],
          y: ys[i],
        },
        leaf
      );
    }
  }
  #[test]
  fn leaves_raw() {
    let tree = K2Tree::test_tree();
    let values = bitbox![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0];
    for (i, leaf) in tree.leaves_raw().enumerate() {
      assert_eq!(values[i], leaf);
    }
  }
  #[test]
  fn shrink_if_possible() {
    let mut tree = K2Tree::new();
    tree.grow();
    assert_eq!(16, tree.matrix_width);
    tree.shrink_if_possible();
    assert_eq!(8, tree.matrix_width);
    tree.shrink_if_possible();
    assert_eq!(8, tree.matrix_width);
  }
  #[test]
  fn shrink() {
    let mut tree = K2Tree::new();
    tree.grow();
    assert_eq!(16, tree.matrix_width);
    assert!(tree.shrink().is_ok());
    assert_eq!(8, tree.matrix_width);
    assert!(tree.shrink().is_err());
    assert_eq!(8, tree.matrix_width);
  }
  #[test]
  fn shrink_unchecked() {
    let mut tree = K2Tree::new();
    tree.grow();
    assert_eq!(16, tree.matrix_width);
    unsafe { tree.shrink_unchecked(); }
    assert_eq!(8, tree.matrix_width);
  }
  #[test]
  fn from_matrix() -> Result<()> {
    let bits = bitvec![
      0,0,0,0, 0,1,0,1,
      0,0,0,0, 1,0,0,1,
      0,0,0,0, 0,0,1,1,
      0,0,0,0, 0,0,0,0,

      1,0,0,0, 0,1,0,0,
      0,0,0,0, 1,0,0,0,
      0,0,0,0, 0,0,0,0,
      0,0,0,0, 0,0,0,0,
    ];
    let m = BitMatrix::from_bits(8, 8, bits);
    let tree = K2Tree {
      matrix_width: 8,
      k: 2,
      max_slayers: 2,
      slayer_starts: vec![0, 4],
      stems:  bitvec![0,1,1,1, 1,1,0,1, 1,0,0,0, 1,0,0,0],
      stem_to_leaf: vec![0, 1, 3, 4, 8],
      leaves: bitvec![0,1,1,0, 0,1,0,1, 1,1,0,0, 1,0,0,0, 0,1,1,0]
    };
    assert_eq!(tree, K2Tree::from_matrix(m)?);
    Ok(())
  }
  #[test]
  fn to_matrix() -> Result<()> {
    let bits = bitvec![
      0,0,0,0, 0,1,0,1,
      0,0,0,0, 1,0,0,1,
      0,0,0,0, 0,0,1,1,
      0,0,0,0, 0,0,0,0,

      1,0,0,0, 0,1,0,0,
      0,0,0,0, 1,0,0,0,
      0,0,0,0, 0,0,0,0,
      0,0,0,0, 0,0,0,0,
    ];
    let m = BitMatrix::from_bits(8, 8, bits);
    let new_m = K2Tree::from_matrix(m.clone())?.to_matrix()?;
    assert_eq!(m, new_m);
    Ok(())
  }
  #[test]
  fn into_matrix() -> Result<()> {
    let bits = bitvec![
      0,0,0,0, 0,1,0,1,
      0,0,0,0, 1,0,0,1,
      0,0,0,0, 0,0,1,1,
      0,0,0,0, 0,0,0,0,

      1,0,0,0, 0,1,0,0,
      0,0,0,0, 1,0,0,0,
      0,0,0,0, 0,0,0,0,
      0,0,0,0, 0,0,0,0,
    ];
    let m = BitMatrix::from_bits(8, 8, bits);
    let new_m = K2Tree::from_matrix(m.clone())?.into_matrix()?;
    assert_eq!(m, new_m);
    Ok(())
  }
}

/* Util Tests */
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
    assert_eq!(vec![1,3,5,9], one_positions(&bv));
  }
  #[test]
  fn to_4_subranges_0() {
    let ranges = [[[0, 7], [0, 7]], [[4, 7], [0, 3]], [[8, 15], [8, 15]]];
    let subranges = [
      [[[0, 3], [0, 3]], [[4, 7], [0, 3]], [[0, 3], [4, 7]], [[4, 7], [4, 7]]],
      [[[4, 5], [0, 1]], [[6, 7], [0, 1]], [[4, 5], [2, 3]], [[6, 7], [2, 3]]],
      [[[8, 11], [8, 11]], [[12, 15], [8, 11]], [[8, 11], [12, 15]], [[12, 15], [12, 15]]]
    ];
    for i in 0..ranges.len() {
      assert_eq!(to_4_subranges(ranges[i]), subranges[i]);
    }
  }
  #[test]
  fn within_range_0() {
    let coords = [[0, 0], [5, 6], [87, 2],[5, 5]];
    let ranges = [[[0, 3], [0, 3]], [[0, 7], [0, 7]], [[50, 99], [0, 49]], [[5, 9], [5, 9]]];
    for i in 0..coords.len() {
      assert!(within_range(&ranges[i], coords[i][0], coords[i][1]));
    }
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
    let tree = K2Tree::test_tree();
    assert_eq!(tree.layer_start(0), 0);
    assert_eq!(tree.layer_start(1), 4);
  }
  #[test]
  fn stem_layer_len_0() {
    let tree = K2Tree::test_tree();
    assert_eq!(tree.layer_len(0), 4);
    assert_eq!(tree.layer_len(1), 12);
  }
  #[test]
  fn leaf_start_0() {
    let tree = K2Tree::test_tree();
    assert_eq!(tree.leaf_start(4), Ok(0));
    assert_eq!(tree.leaf_start(5), Ok(4));
    assert_eq!(tree.leaf_start(7), Ok(8));
    assert_eq!(tree.leaf_start(8), Ok(12));
    assert_eq!(tree.leaf_start(12), Ok(16));
    assert_eq!(tree.leaf_start(9), Err(()));
  }
  #[test]
  fn child_stem_0() {
    let tree = K2Tree::test_tree();
    assert_eq!(tree.child_stem(0, 0, 0), Err(()));
    assert_eq!(tree.child_stem(0, 0, 1), Ok(4));
    assert_eq!(tree.child_stem(0, 0, 2), Ok(8));
    assert_eq!(tree.child_stem(0, 0, 3), Ok(12));
    assert_eq!(tree.child_stem(1, 4, 0), Err(()));
  }
  #[test]
  fn parent_stem_0() {
    let tree = K2Tree::test_tree();
    assert_eq!(tree.parent_stem(4), 0);
    assert_eq!(tree.parent_stem(8), 0);
    assert_eq!(tree.parent_stem(12), 0);
  }
  #[test]
  fn parent_bit_0() {
    let tree = K2Tree::test_tree();
    assert_eq!(tree.parent_bit(4), 1);
    assert_eq!(tree.parent_bit(8), 2);
    assert_eq!(tree.parent_bit(12), 3);
  }
  #[test]
  fn get_coords_0() {
    let tree = K2Tree::test_tree();
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
    let mut tree = K2Tree::new();
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
}