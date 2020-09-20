

/* Public Interface Tests */

use crate::{
  K2Tree,
  matrix::BitMatrix
};
use bitvec::prelude::*;

type Result<T> = std::result::Result<T, crate::error::K2TreeError>;

/* Private funcs used in testing */
fn test_tree(k: usize) -> K2Tree {
  match k {
    2 => K2Tree::from_matrix(test_matrix(2), 2, 2).unwrap(),
    3 => K2Tree::from_matrix(test_matrix(3), 3, 3).unwrap(),
    _ => K2Tree::new(),
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

#[test]
fn new() {
  let tree = K2Tree::new();
  assert_eq!(tree.stem_k(), 2);
  assert_eq!(tree.leaf_k(), 2);
  assert_eq!(tree.stems(), &bitvec![0,0,0,0]);
  assert_eq!(tree.leaves(), &bitvec![]);
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
      let tree = K2Tree::with_k(stem_k, leaf_k)?;
      assert_eq!(tree.stem_k(), stem_k);
      assert_eq!(tree.leaf_k(), leaf_k);
      assert_eq!(tree.stems(), &bitvec![0; stem_k.pow(2)]);
      assert_eq!(tree.leaves(), &bitvec![]);
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
  let mut tree = test_tree(2);
  assert_eq!(tree.stem_k(), 2);
  assert!(tree.set_stem_k(3).is_ok());
  let expected_stems = bitvec![
    1,1,0,0,0,0,0,0,0, 0,0,1,0,0,0,1,0,1,
    1,0,0,1,0,0,0,0,0
  ];
  let expected_leaves = bitvec![
    0,1,1,0, 1,0,0,0, 0,1,1,0,
    0,1,0,1, 1,1,0,0
  ];
  assert_eq!(tree.stem_k(), 3);
  assert_eq!(tree.leaf_k(), 2);
  assert_eq!(tree.stems(), &expected_stems);
  assert_eq!(tree.leaves(), &expected_leaves);
}
#[test]
fn set_stem_k_2() {
  let mut tree = test_tree(3);
  assert_eq!(tree.stem_k(), 3);
  assert!(tree.set_stem_k(2).is_ok());
  let expected_stems = bitvec![
    1,0,0,0, 1,1,1,0, 0,1,1,1, 1,0,0,0, 0,0,1,1,
    0,1,0,0, 0,0,1,0, 0,0,0,1, 1,0,0,0, 1,0,0,0, 0,1,0,0
  ];
  let expected_leaves = bitvec![
    0,1,0,1,0,0,0,0,1, 1,0,0,0,0,0,0,0,0,
    0,1,0,1,0,0,0,0,0, 1,0,0,1,0,0,1,0,0,
    1,0,0,0,0,0,0,0,0, 0,1,0,1,0,0,0,0,0
  ];
  assert_eq!(tree.stem_k(), 2);
  assert_eq!(tree.leaf_k(), 3);
  assert_eq!(tree.stems(), &expected_stems);
  assert_eq!(tree.leaves(), &expected_leaves);
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
  let mut tree = test_tree(2);
  assert_eq!(tree.leaf_k(), 2);
  assert!(tree.set_leaf_k(3).is_ok());
  let expected_stems = bitvec![
    1,1,0,0, 0,1,1,1, 1,0,0,0,
  ];
  let expected_leaves = bitvec![
    0,0,1,0,1,0,0,0,0, 0,0,0,1,0,0,0,0,0,
    0,0,0,0,0,1,0,1,0, 0,1,0,0,1,0,1,1,0
  ];
  assert_eq!(tree.stem_k(), 2);
  assert_eq!(tree.leaf_k(), 3);
  assert_eq!(tree.stems(), &expected_stems);
  assert_eq!(tree.leaves(), &expected_leaves);
}
#[test]
fn set_leaf_k_2() {
  let mut tree = test_tree(3);
  assert_eq!(tree.leaf_k(), 3);
  assert!(tree.set_leaf_k(2).is_ok());
  let expected_stems = bitvec![
    1,0,0,1,0,0,0,0,0, 0,1,1,1,1,0,0,0,0, 1,1,0,0,0,0,0,0,0, //final layer starts below
    0,1,1,0,0,1,0,0,0, 1,0,0,1,0,0,0,0,0, 0,0,0,1,0,0,0,0,0,
    0,0,0,0,0,1,0,1,0, 1,0,0,0,0,0,0,0,0, 0,1,1,0,0,0,0,0,0
  ];
  let expected_leaves = bitvec![
    0,0,0,1, 1,0,0,0, 0,1,0,0, 1,0,1,0, 1,0,0,0,
    0,0,1,0, 0,0,1,0, 0,1,0,0, 1,0,0,0, 0,0,0,1,
    1,0,0,0
  ];
  assert_eq!(tree.stem_k(), 3);
  assert_eq!(tree.leaf_k(), 2);
  assert_eq!(tree.stems(), &expected_stems);
  assert_eq!(tree.leaves(), &expected_leaves);
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
    let tree = test_tree(k);
    let matrix = test_matrix(k);
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
  let tree = test_tree(2);
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
  let tree = test_tree(2);
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
  for k in 2..5usize {
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
    let mut tree = test_tree(k);
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
    assert_eq!(k, K2Tree::with_k(k, k)?.stem_k());
  }
  Ok(())
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
    let matrix = test_matrix(k);
    let tree = test_tree(k);
    assert_eq!(tree, K2Tree::from_matrix(matrix, k, k)?);
  }
  Ok(())
}
#[test]
fn to_matrix() -> Result<()> {
  for k in 2..=3usize {
    let tree = test_tree(k);
    let matrix = test_matrix(k);
    assert_eq!(matrix, tree.to_matrix()?);
    assert_eq!(matrix, K2Tree::from_matrix(matrix.clone(), k, k)?.to_matrix()?);
  }
  Ok(())
}
#[test]
fn into_matrix() -> Result<()> {
  for k in 2..=3usize {
    let tree = test_tree(k);
    let matrix = test_matrix(k);
    assert_eq!(matrix, tree.into_matrix()?);
    assert_eq!(matrix, K2Tree::from_matrix(matrix.clone(), k, k)?.into_matrix()?);
  }
  Ok(())
}

#[test]
  fn build() -> Result<()> {
    for i in 2..=3 {
      for stem_k in 2..9 {
        for leaf_k in 2..9 {
          let m = test_matrix(i);
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
    println!("{}", test_tree(3));
  }
  #[test]
  fn eq() -> Result<()> {
    assert_eq!(K2Tree::new(), K2Tree::new());
    assert_eq!(test_tree(2), test_tree(2));
    assert_eq!(test_tree(3), test_tree(3));
    assert_eq!(test_tree(4), test_tree(4));
    for stem_k in 2..10 {
      for leaf_k in 2..10 {
        assert_eq!(
          K2Tree::with_k(stem_k, leaf_k)?,
          K2Tree::with_k(stem_k, leaf_k)?
        );
      }
    }
    Ok(())
  }