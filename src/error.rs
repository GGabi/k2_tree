/*!
These are all the custom errors that this library could return.

This library uses a nesting system to convey the most useful information 
while minimising the number of unique enumerations required:
- CorruptedK2Tree:
  - The K2Tree is *likely* to have been corrupted by the operation that resulted in this error.
- Read:
  - The error occured during a Read operation, meaning that the state of the K2Tree could
    not have been changed by the operation that resulted in this error.
- Write:
  - The error occured during a Write operation, meaning that the state of the K2Tree *may*
    have been changed by the operation that resulted in this error.
*/

/// Errors produced as a result of interactions with the K2Tree object.
#[derive(Clone, Debug)]
pub enum K2TreeError {
  /// Produced when a user attempts to create a K2Tree with a k values below 2.
  SmallKValue {
    ///
    k: u8,
  },
  /// Produced when a problem occurs attempting to traverse a K2Tree.
  /// 
  /// This mostly appears because the internal state of K2Tree is corrupted,
  /// or the user found a way to search for an invalid coordinate within the bounds
  /// of the matrix the K2Tree represents.  
  TraverseError {
    ///
    x: usize,
    ///
    y: usize
  },
  /// Produced when a user attempts to access a coordinate outside the bounds of
  /// the matrix a K2Tree represents.
  OutOfBounds{
    ///
    x_y: [usize; 2],
    ///
    min_x_y: [usize; 2],
    ///
    max_x_y: [usize; 2],
  },
  /// Produced when a stem could not be inserted into a K2Tree's stems.
  StemInsertionError {
    /// The index the stem-insertion was attempted at.
    pos: usize,
    /// The length of the stem.
    len: usize
  },
  /// Produced when a stem coud not be removed from a K2Tree's stems.
  StemRemovalError {
    /// The index the stem-removal was attempted at.
    pos: usize,
    /// The length of the stem.
    len: usize
  },
  /// Produced when a leaf could not be inserted into a K2Tree's leaves.
  LeafInsertionError {
    /// The index the leaf-insertion was attempted at.
    pos: usize,
    /// The length of the leaf.
    len: usize
  },
  /// Produced when a leaf could not be removed from a K2Tree's leaves.
  LeafRemovalError {
    /// The index the leaf-removal was attempted at.
    pos: usize,
    /// the length of the leaf.
    len: usize
  },
  /// Produced when a problem occurs attempting to shrink the matrix a K2Tree represents.
  CouldNotShrink {
    ///
    reason: String
  },
  /// Indicates that the source error resulted in the corruption of a K2Tree.
  CorruptedK2Tree {
    ///
    source: Box<K2TreeError>
  },
  /// Indicates that the source error was produced during a read operation on a K2Tree.
  /// 
  /// Almost certainly guarantees that the error did not cause any corruption. 
  Read {
    ///
    source: Box<K2TreeError>
  },
  /// Indicates that the source error was produced during a write operation on a K2Tree.
  Write {
    ///
    source: Box<K2TreeError>
  },
  /// Propogation of a BitMatrixError.
  BitMatrixError {
    ///
    source: Box<BitMatrixError>,
  },
  /// Propogation of a SubRangesError.
  SubRangesError {
    ///
    source: Box<SubRangesError>,
  }
}
impl std::error::Error for K2TreeError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    use K2TreeError::*;
    match self {
      CorruptedK2Tree{source} => Some(source),
      Read{source} => Some(source),
      Write{source} => Some(source),
      BitMatrixError{source} => Some(source),
      SubRangesError{source} => Some(source),
      _ => None,
    }
  }
}
impl std::fmt::Display for K2TreeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use K2TreeError::*;
    match self {
      SmallKValue{k} => write!(f, "Attempt to create a K2Tree with a k value of {}, which less than the minimum of 2.", k),
      TraverseError{x, y} => write!(f, "Error encountered while traversing K2Tree for value at coordinates ({}, {})", x, y),
      OutOfBounds {
        x_y: [x, y],
        min_x_y: [min_x, min_y],
        max_x_y: [max_x, max_y]
      } => write!(f, "Attempt to access a bit at coordiantes ({}, {}) which are not in the range of the matrix represented by the K2Tree: ({}, {}) -> ({}, {})", x, y, min_x, min_y, max_x, max_y),
      StemInsertionError{pos, len} => write!(f, "Could not insert stem of length {} at index {}", len, pos),
      StemRemovalError{pos, len} => write!(f, "Could not remove stem of length {} at index {}", len, pos),
      LeafInsertionError{pos, len} => write!(f, "Could not insert leaf of length {} at index {}", len, pos),
      LeafRemovalError{pos, len} => write!(f, "Could not remove leaf of length {} at index {}", len, pos),
      CouldNotShrink{reason} => write!(f, "Could not shrink the matrix a K2Tree represents: {}", reason),
      CorruptedK2Tree{source} => write!(f, "The K2Tree's contents are corrupted as a result of the following error: {}", source),
      Read{source} => write!(f, "Error during read: {}", source),
      Write{source} => write!(f, "Error during write: {}", source),
      BitMatrixError{source} => write!(f, "{}", source),
      SubRangesError{source} => write!(f, "{}", source),
    }
  }
}
impl From<BitMatrixError> for K2TreeError {
  fn from(error: BitMatrixError) -> Self {
    K2TreeError::BitMatrixError {
      source: Box::new(error)
    }
  }
}
impl From<SubRangesError> for K2TreeError {
  fn from(error: SubRangesError) -> Self {
    K2TreeError::SubRangesError {
      source: Box::new(error),
    }
  }
}

/// Errors produced as a result of interactions with the BitMatrix object.
#[derive(Clone, Debug)]
pub enum BitMatrixError {
  /// Produced when a user attempts to read or write to a bit outside of the
  /// valid range.
  OutOfBounds {
    ///
    x_y: [usize; 2],
    ///
    max_x_y: [usize; 2],
  }
}
impl std::error::Error for BitMatrixError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    None
  }
}
impl std::fmt::Display for BitMatrixError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use BitMatrixError::*;
    match self {
      OutOfBounds {
        x_y: [x, y],
        max_x_y: [max_x, max_y],
      } => write!(f, "Attempts to access a bit at coordinates({}, {}) which are not in the range of the matrix: (0, 0) -> ({}, {})", x, y, max_x, max_y),
    }
  }
}

/// Errors produced as a result of interactions with the SubRanges object.
#[derive(Clone, Debug)]
pub enum SubRangesError {
  /// Produced when a user attempts a Range2D cannot be evenly subdivided
  /// by the requested number of subranges.
  CannotSubdivideRange {
    /// The range which could not be subdivided.
    range: [[usize; 2]; 2],
    /// The attempted number of horizontal subdivisions.
    horizontal_subdivisions: usize,
    /// The attempted number of vertical subdivisions.
    vertical_subdivisions: usize,
  }
}
impl std::error::Error for SubRangesError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    None
  }
}
impl std::fmt::Display for SubRangesError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use SubRangesError::*;
    match self {
      CannotSubdivideRange {
        range: [[min_x, max_x], [min_y, max_y]],
        horizontal_subdivisions: hs,
        vertical_subdivisions: vs, 
      } => write!(f, "The 2D range ({}, {}) -> ({}, {}) could not be evenly subdivided {} times horizontally or {} times vertically.", min_x, min_y, max_x, max_y, hs, vs),
    }
  }
}