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
## Final `K2Tree`:
Finally, the bit representation of the K2Tree is stored alongside various metadata; so that the original matrix can be recovered and to greatly optimise various operations.
```rust
K2Tree {
  matrix_width: 8,
  k: 2,
  max_slayers: 2,
  slayer_starts: [0, 4],
  stems: [0111110111000100],
  stem_to_leaf: [0, 1, 3, 4, 5, 9],
  leaves: [100010110010101010001100],
}
```

\- GGabi