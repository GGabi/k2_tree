
use bitvec::vec::BitVec;
use crate::error::BitMatrixError;

type Result<T> = std::result::Result<T, BitMatrixError>;

/// A 2-d bit-matrix.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitMatrix {
  /// Width of the matrix.
  pub width: usize,
  /// Height of the matrix.
  pub height: usize,
  bits: BitVec,
}
impl BitMatrix {
  /// Creates an empty BitMatrix with zero width or height.
  pub fn new() -> Self {
    BitMatrix {
      width: 0,
      height: 0,
      bits: BitVec::new(),
    }
  }
  /// Creates an empty BitMatrix with predefined dimensions.
  pub fn with_dimensions(width: usize, height: usize) -> Self {
    let mut bits = BitVec::with_capacity(width*height);
    bits.resize_with(width*height, Default::default);
    BitMatrix {
      width,
      height,
      bits,
    }
  }
  /// Builds a BitMatrix instance from another collection of bits.
  /// 
  /// If the data passed in contains more bits than will fit a matrix of the specified
  /// height and width, excess data is discarded. If not enough bits are passed in, 0s
  /// will be appended until the right size is reached.
  pub fn from_bits(width: usize, height: usize, data: impl IntoIterator<Item=bool>) -> Self {
    let mut bits: BitVec = data.into_iter().collect();
    bits.resize_with(width*height, Default::default);
    BitMatrix {
      width,
      height,
      bits,
    }
  }
  /// Returns the state of a bit at a specific coordinate.
  pub fn get(&self, x: usize, y: usize) -> Result<bool> {
    if x >= self.width || y >= self.height {
      return Err(BitMatrixError::OutOfBounds {
        x_y: [x, y],
        max_x_y: [self.width-1, self.height-1],
      })
    }
    let index: usize = y*self.width + x;
    Ok(*self.bits.get(index).unwrap())
  }
  /// Returns the state of all the bits at a specific x-coordinate.
  /// 
  /// Bits are ordered by row, starting at y-coordinate 0.
  pub fn get_column(&self, x: usize) -> Result<Vec<bool>> {
    if x >= self.width {
      return Err(BitMatrixError::OutOfBounds {
        x_y: [x, 0],
        max_x_y: [self.width-1, self.height-1],
      })
    }
    let mut column = Vec::new();
    for row in 0..self.height {
      column.push(self.bits[x + (row * self.width)]);
    }
    Ok(column)
  }
  /// Returns the state of all the bits at a specific y-coordinate.
  /// 
  /// Bits are ordered by column, starting at x-coordinate 0.
  pub fn get_row(&self, y: usize) -> Result<Vec<bool>> {
    if y >= self.height {
      return Err(BitMatrixError::OutOfBounds {
        x_y: [0, y],
        max_x_y: [self.width-1, self.height-1],
      })
    }
    let mut row = Vec::new();
    for column in 0..self.width {
      row.push(self.bits[(y * self.width) + column]);
    }
    Ok(row)
  }
  /// Changes the state of a bit at a specififc coordinate.
  pub fn set(&mut self, x: usize, y: usize, state: bool) -> Result<()> {
    if x >= self.width || y >= self.height {
      return Err(BitMatrixError::OutOfBounds {
        x_y: [x, y],
        max_x_y: [self.width-1, self.height-1],
      })
    }
    let index: usize = y*self.width + x;
    self.bits.set(index, state);
    Ok(())
  }
  /// Changes the width of the matrix.
  /// 
  /// If len is greater than matrix's width, each row is extended with 0s.
  /// Otherwise, each row is concatenated.
  pub fn resize_width(&mut self, len: usize) {
    //Add or remove values at the correct spaces from the end backwards,
    //  as to not change the index of insertion sites on other rows.
    //Work out whether we're growing or shrinking
    if len > self.width {
      //Growing
      let diff = len - self.width;
      for row in (1..=self.height).rev() {
        let row_end = self.width * row;
        for _ in 0..diff { self.bits.insert(row_end, false); }
      }
    }
    else if len < self.width {
      //Shrinking
      let diff = self.width - len;
      for row in (1..=self.height).rev() {
        let row_end = self.width * row;
        for _ in 0..diff { self.bits.remove(row_end-diff); }
      }
    }
    self.width = len;
  }
  /// Changes the hieght of the matrix.
  /// 
  /// If len is greater than matrix's height, it is extended with blank rows.
  /// Otherwise, the number of rows is suitably concatenated.
  pub fn resize_height(&mut self, len: usize) {
    if len > self.height {
      //Growing
      let new_rows = len - self.height;
      let new_bits = new_rows * self.width;
      let end = self.bits.len();
      for _ in 0..new_bits { self.bits.insert(end, false); }
    }
    else if len < self.height {
      //Shrinking
      let less_rows = self.height - len;
      let less_bits = less_rows * self.width;
      let new_end = self.bits.len() - less_bits;
      for _ in 0..less_bits { self.bits.remove(new_end); }
    }
    self.height = len;
  }
  /// Produces the contents of the matrix as a flat vec of bits.
  /// 
  /// Vec contains each row one after another.
  pub fn to_bits(&self) -> Vec<bool> {
    let mut bits = Vec::with_capacity(self.bits.len());
    bits.extend(&self.bits);
    bits
  }
  /// Consumes the BitMatrix to produce its contents as a flat vec of bits.
  /// 
  /// Vec contains each row one after another.
  pub fn into_bits(self) -> Vec<bool> {
    let mut bits = Vec::with_capacity(self.bits.len());
    bits.extend(&self.bits);
    bits
  }
  /// Produces the contents of the matrix as a vec of its columns.
  pub fn to_columns(&self) -> Vec<Vec<bool>> {
    let mut vecs = vec![Vec::with_capacity(self.height); self.width];
    for column in 0..self.width {
      for row in 0..self.height {
        vecs[column].push(self.bits[row*self.width + column]);
      }
    }
    vecs
  }
  /// Consumes the BitMatrix to produce its contents as a vec of its columns.
  pub fn into_columns(self) -> Vec<Vec<bool>> {
    let mut vecs = vec![Vec::with_capacity(self.height); self.width];
    for column in 0..self.width {
      for row in 0..self.height {
        vecs[column].push(self.bits[row*self.width + column]);
      }
    }
    vecs
  }
  /// Produces the contents of the matrix as a vec of its rows.
  pub fn to_rows(&self) -> Vec<Vec<bool>> {
    let mut vecs = vec![Vec::with_capacity(self.width); self.height];
    for row in 0..self.height {
      vecs[row].extend(&self.bits[row*self.width..(row+1)*self.width]);
    }
    vecs
  }
  /// Consumes the BitMatrix to produce its contents as a vec of its rows.
  pub fn into_rows(self) -> Vec<Vec<bool>> {
    let mut vecs = vec![Vec::with_capacity(self.width); self.height];
    for row in 0..self.height {
      vecs[row].extend(&self.bits[row*self.width..(row+1)*self.width]);
    }
    vecs
  }
  /// Reduces the width and height such that there are no empty columns or rows
  /// on the edges.
  pub fn shrink_to_fit(&mut self) {
    //Find the rightmost column containing a 1
    //Find the lowest row containing a 1
    //Set width and height to those positions
    for col in (0..self.width).rev() {
      let col_bits = {
        let mut bv = BitVec::new();
        bv.extend(self.get_column(col).unwrap());
        bv
      };
      if !all_zeroes(&col_bits, 0, col_bits.len()) {
        self.resize_width(col+1);
        break
      }
    }
    for row in (0..self.height).rev() {
      let row_bits = {
        let mut bv = BitVec::new();
        bv.extend(self.get_row(row).unwrap());
        bv
      };
      if !all_zeroes(&row_bits, 0, row_bits.len()) {
        self.resize_height(row+1);
        break
      }
    }
  }
}
impl Default for BitMatrix {
  fn default() -> Self {
    BitMatrix::new()
  }
}

fn all_zeroes(bits: &BitVec, begin: usize, end: usize) -> bool {
  bits[begin..end].into_iter().fold(true, |total, bit| total & !bit)
}

#[cfg(test)]
mod api {
  use super::*;
  #[test]
  fn new() {
    let m = BitMatrix::new();
    assert_eq!(0, m.width);
    assert_eq!(0, m.height);
    assert_eq!(Vec::<bool>::new(), m.into_bits());
  }
  #[test]
  fn with_dimensions() {
    let m = BitMatrix::with_dimensions(8, 8);
    assert_eq!(8, m.width);
    assert_eq!(8, m.height);
    assert_eq!(vec![false; 64], m.into_bits());
  }
  #[test]
  fn from_bits() {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let m = BitMatrix::from_bits(4, 4, bits.clone());
    assert_eq!(4, m.width);
    assert_eq!(4, m.height);
    assert_eq!(bits, m.into_bits());
  }
  #[test]
  fn get() -> Result<()> {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let m = BitMatrix::from_bits(4, 4, bits.clone());
    assert_eq!(false, m.get(0, 0)?);
    assert_eq!(false, m.get(0, 1)?);
    assert_eq!(true, m.get(3, 0)?);
    assert_eq!(true, m.get(2, 1)?);
    assert_eq!(true, m.get(1, 2)?);
    assert_eq!(true, m.get(0, 3)?);
    assert_eq!(false, m.get(3, 3)?);
    Ok(())
  }
  #[test]
  fn set() -> Result<()> {
    let mut m = BitMatrix::with_dimensions(8, 8);
    assert_eq!(false, m.get(0, 0)?);
    m.set(0, 0, true)?;
    assert_eq!(true, m.get(0, 0)?);
    m.set(0, 0, false)?;
    assert_eq!(false, m.get(0, 0)?);
    m.set(3, 3, true)?;
    assert_eq!(true, m.get(3, 3)?);
    assert_eq!(false, m.get(2, 3)?);
    assert_eq!(false, m.get(3, 2)?);
    Ok(())
  }
  #[test]
  fn resize_width_grow() -> Result<()> {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let mut m = BitMatrix::from_bits(4, 4, bits);
    assert_eq!(4, m.width);
    assert_eq!(true, m.get(3, 0)?);
    assert!(m.get(7, 0).is_err());
    m.resize_width(8);
    assert_eq!(8, m.width);
    assert_eq!(true, m.get(3, 0)?);
    assert_eq!(false, m.get(4, 0)?);
    assert_eq!(false, m.get(7, 0)?);
    Ok(())
  }
  #[test]
  fn resize_width_shrink() -> Result<()> {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let mut m = BitMatrix::from_bits(4, 4, bits);
    assert_eq!(4, m.width);
    assert_eq!(true, m.get(3, 0)?);
    assert_eq!(true, m.get(1, 2)?);
    m.resize_width(2);
    assert_eq!(2, m.width);
    assert!(m.get(3, 0).is_err());
    assert_eq!(false, m.get(1, 0)?);
    assert_eq!(true, m.get(1, 2)?);
    Ok(())
  }
  #[test]
  fn resize_height_grow() -> Result<()> {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let mut m = BitMatrix::from_bits(4, 4, bits);
    assert_eq!(4, m.height);
    assert_eq!(true, m.get(0, 3)?);
    assert!(m.get(0, 5).is_err());
    m.resize_height(8);
    assert_eq!(8, m.height);
    assert_eq!(true, m.get(0, 3)?);
    assert_eq!(false, m.get(0, 5)?);
    Ok(())
  }
  #[test]
  fn resize_height_shrink() -> Result<()> {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let mut m = BitMatrix::from_bits(4, 4, bits);
    assert_eq!(4, m.height);
    assert_eq!(true, m.get(0, 3)?);
    assert_eq!(false, m.get(0, 1)?);
    m.resize_height(2);
    assert_eq!(2, m.height);
    assert!(m.get(0, 3).is_err());
    assert_eq!(false, m.get(0, 1)?);
    Ok(())
  }
  #[test]
  fn to_bits() {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    assert_eq!(bits, BitMatrix::from_bits(4, 4, bits.clone()).to_bits());
  }
  #[test]
  fn into_bits() {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    assert_eq!(bits, BitMatrix::from_bits(4, 4, bits.clone()).into_bits());
  }
  #[test]
  fn to_columns() {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let vecs = BitMatrix::from_bits(4, 4, bits.clone()).to_columns();
    assert_eq!(4, vecs.len());
    for column in 0..4 { assert_eq!(4, vecs[column].len()); }
    assert_eq!(vec![bits[0], bits[4], bits[8], bits[12]], vecs[0]);
    assert_eq!(vec![bits[1], bits[5], bits[9], bits[13]], vecs[1]);
    assert_eq!(vec![bits[2], bits[6], bits[10], bits[14]], vecs[2]);
    assert_eq!(vec![bits[3], bits[7], bits[11], bits[15]], vecs[3]);
  }
  #[test]
  fn into_columns() {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let vecs = BitMatrix::from_bits(4, 4, bits.clone()).into_columns();
    assert_eq!(4, vecs.len());
    for column in 0..4 { assert_eq!(4, vecs[column].len()); }
    assert_eq!(vec![bits[0], bits[4], bits[8], bits[12]], vecs[0]);
    assert_eq!(vec![bits[1], bits[5], bits[9], bits[13]], vecs[1]);
    assert_eq!(vec![bits[2], bits[6], bits[10], bits[14]], vecs[2]);
    assert_eq!(vec![bits[3], bits[7], bits[11], bits[15]], vecs[3]);
  }
  #[test]
  fn to_rows() {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let rows = BitMatrix::from_bits(4, 4, bits.clone()).to_rows();
    assert_eq!(4, rows.len());
    for row in 0..4 { assert_eq!(4, rows[row].len()); }
    assert_eq!(bits[0..4].to_vec(), rows[0]);
    assert_eq!(bits[4..8].to_vec(), rows[1]);
    assert_eq!(bits[8..12].to_vec(), rows[2]);
    assert_eq!(bits[12..16].to_vec(), rows[3]);
  }
  #[test]
  fn into_rows() {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let rows = BitMatrix::from_bits(4, 4, bits.clone()).into_rows();
    assert_eq!(4, rows.len());
    for row in 0..4 { assert_eq!(4, rows[row].len()); }
    assert_eq!(bits[0..4].to_vec(), rows[0]);
    assert_eq!(bits[4..8].to_vec(), rows[1]);
    assert_eq!(bits[8..12].to_vec(), rows[2]);
    assert_eq!(bits[12..16].to_vec(), rows[3]);
  }
  #[test]
  fn get_column() -> Result<()> {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let m = BitMatrix::from_bits(4, 4, bits.clone());
    assert_eq!(
      vec![false,false,false,true],
      m.get_column(0)?
    );
    assert_eq!(
      vec![false,false,true,false],
      m.get_column(1)?
    );
    assert_eq!(
      vec![false,true,false,false],
      m.get_column(2)?
    );
    assert_eq!(
      vec![true,false,false,false],
      m.get_column(3)?
    );
    Ok(())
  }
  #[test]
  fn get_row() -> Result<()> {
    let bits = vec![
      false,false,false,true,
      false,false,true,false,
      false,true,false,false,
      true,false,false,false,
    ];
    let m = BitMatrix::from_bits(4, 4, bits.clone());
    assert_eq!(
      vec![false,false,false,true],
      m.get_row(0)?
    );
    assert_eq!(
      vec![false,false,true,false],
      m.get_row(1)?
    );
    assert_eq!(
      vec![false,true,false,false],
      m.get_row(2)?
    );
    assert_eq!(
      vec![true,false,false,false],
      m.get_row(3)?
    );
    Ok(())
  }
  #[test]
  fn shrink_to_fit() {
    let bits = vec![
      true,true,false,false,
      true,false,true,false,
      false,false,false,false,
      false,false,false,false,
    ];
    let mut m = BitMatrix::from_bits(4, 4, bits.clone());
    assert_eq!(4, m.width);
    assert_eq!(4, m.height);
    m.shrink_to_fit();
    assert_eq!(3, m.width);
    assert_eq!(2, m.height);
  }
}
