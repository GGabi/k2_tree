# k2_tree
A collection designed to efficiently compress sparsely-populated bit-matrices.

See the original proposal [here](https://users.dcc.uchile.cl/~gnavarro/ps/spire09.1.pdf).

**Note:** This library heavily relies upon [bitvec](https://docs.rs/bitvec/0.17.4/bitvec/) to optimally store its data.
If you have `k2_tree` as a dependancy, always try to compile with optimisations! `bit_vec` is very slow without them!

# When `K2Tree`s are Useful:
`K2Tree`s are useful when you need to store two-dimensional data efficiently, especially when the data is sparsely populated. A real world example would be representing Web-Graphs. In this scenario, each column and row of a bit-matrix would represent a specific webpage, and all bits represent the whether two pages are joined by a hyperlink; 1 if yes and 0 if no. As it turns out, these types of Web-Graphs tend to produce sparsely populated bit-matrices. Another example would be representing Triple-Stores, which [this repo](https://github.com/GGabi/RippleDB) demonstrates is effective.
# How it Works:
## Original Bit-Matrix:
```
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
* The smallest is width `k`.
* All others are `k * child_width`.
## Modified Matrix
Then, all sub-matrices containing only zeroes are substituted by a single zero, like so:
```
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
```
             0111
          ____|||_________
          |    |         |
          1101 1100      0100
|----|----|    |----|    |
1000 1011 0010 1010 1000 1100

```
In the first layer of the tree, each bit refers to one of the 4 largest quadrants in the modified matrix in the order:
* The top-left contains nothing.
* The top-right contains something.
* The bottom-left contains something.
* The bottom-right contains something.

Then, for the second layer each block refers to the sub-matrices of each quadrant:
* The top-right quadrant contains the following sub-quadrants:
  * The top-left, top-right and bottom-right contain something.
  * The bottom-left contains nothing.
* The bottom-left qudrant contains the following:
  * Top-left and top-right contains something.
  * Bottom-left and bottom-right contains nothing.
* And so on for the final quadrant.

The final layer is referred to as the leaf-layer and contains the actual data in the matrix:
* The top-left sub-quadrant of the top-right quadrant contains the bits: `1000`
* Etc.
## Bit Representation of K2Tree
The above `K2Tree` is stored as a series of bits:
`[0111; 1101, 1100, 0100; 1000, 1011, 0010, 1010, 1000, 1100]`
(Where `;` separates layers and `,` separates blocks)

\- GGabi
