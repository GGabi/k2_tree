#![allow(non_snake_case)]
#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

/*!
A collection designed to efficiently compress sparsely-populated bit-matrices.

See the original proposal [here](https://users.dcc.uchile.cl/~gnavarro/ps/spire09.1.pdf).

**Note:** This library heavily relies upon [bitvec](https://docs.rs/bitvec/0.17.4/bitvec/) to optimally store its data.
If you have `k2_tree` as a dependancy, always try to compile with optimisations!
`bit_vec` is
very slow without them!
*/

/*!
# What's new in version 0.5:
- `K2Tree` now implements serde's Serialize and Deserialize traits.
*/

/*!
# When `K2Tree`s are Useful:

`K2Tree`s are useful when you need to store two-dimensional data efficiently, especially when 
the data is sparsely populated.

A real world example would be representing Web-Graphs. In this scenario, each column and row 
of a bit-matrix would represent a specific webpage, and all bits represent the whether two
pages are joined by a hyperlink; 1 if yes and 0 if no. As it turns out, these types of Web-Graphs 
tend to produce sparsely populated bit-matrices.

Another example would be representing Triple-Stores, which [this repo](https://github.com/GGabi/RippleDB) 
demonstrates is effective.
*/

/*!
# How it Works:

## Original Bit-Matrix:

```ignore
00|00||10|10
00|00||00|11
------------
00|00||00|00
00|00||00|10
============
10|10||00|11
10|00||00|00
------------
00|00||00|00
00|00||00|00
```

As shown above, the 8x8 bit-matrix is sub-divided into sub-matrices where:
* The smallest is width k
* All others are k * children_width

## Modified Matrix

Then, all sub-matrices containing only zeroes are substituted by a single zero, like so:

```ignore
0    ||10|10
     ||00|11
     ||-----
     ||0 |00
     ||  |10
============
10|10||0 |11
10|00||  |00
------------
0 |0 ||0 |0 
  |  ||  |  
```

## `K2Tree` Representation of Modified Matrix

And then the `K2Tree` is built from this modified matrix:

```ignore
               0111
          ______|||________
          |     |         |
          1101  1100      0100
|----|----|     |----|    |
1000 1011 0010  1010 1000 1100
```

From left-to-right in the first layer of the tree, each bit refers to one of the 4 largest quadrants in the modified matrix: 
* `0111` => Upper-left empty, upper-right not empty, lower-left not empty, lower-right not empty.

Each block in the second layer refers to the sub-matrices of each parent:
* The upper-right quadrant (`1101`) contains the following sub-quadrants:
  * Lower-left is empty.
  * Upper-left, upper-right and lower-right are not empty.
* And so on.

The final, or leaf, layer of the tree contains the actual data in the matrix. 
For example, the upper-left sub-quadrant of the upper-right quadrant contains the bits: `1000`.

## Bit Representation of K2Tree

Finally, the above `K2Tree` is stored as a series of bits:

`[0111; 1101, 1100, 0100; 1000, 1011, 0010, 1010, 1000, 1100]`

(Where `;` separates layers and `,` separates blocks)

## Final `K2Tree`:

```ignore
K2Tree {
  stem_k: 2, // usize
  leaf_k: 2, // usize
  max_slayers: 2, // usize
  stems: [0111110111000100], // BitVec
  leaves: [100010110010101010001100], // BitVec
}
```

-- groels
*/

pub use tree::K2Tree;

/// `K2Tree` structure and assosciated types.
pub mod tree;

/// Library error types.
pub mod error;

/// `BitMatrix` struct.
pub mod matrix;