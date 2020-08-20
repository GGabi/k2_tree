
# k2_tree
A collection designed to efficiently compress sparsely-populated bit-matrices.

See the original proposal [here](https://users.dcc.uchile.cl/~gnavarro/ps/spire09.1.pdf).

**Note:** This library heavily relies upon [bitvec](https://docs.rs/bitvec/0.17.4/bitvec/) to optimally store its data, which is very slow when compiled without optimisations!

## Usage
Add  `k2_tree`  into your project dependencies:
```none
[dependencies]
k2_tree = "0.3.2"
```
# When `K2Tree`s are Useful:
`K2Tree`s are extremely efficient at representing data that can be encoded as a two-dimensional bit-matrix, especially if said matrix is sparsely populated.

Take a real-world example: representing Web-Graphs.
The connections between webpages can be easily encoded as a 2-d bit-matrix, where each column/row corresponds to a specific page and each bit denotes whether two pages are joined via a hyperlink; 1 if yes, 0 in not.
As it turns out, these matrices tend to be extremely sparse *most of the time*, which makes the `K2Tree` a perfect structure for encoding them!

Another example is representing Triple-Stores, which [this repo](https://github.com/GGabi/RippleDB) demonstrates is effective.
# Example:
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
## Bit-Representation:
`[0111; 1101, 1100, 0100; 1000, 1011, 0010, 1010, 1000, 1100]`

(Where `;` separates layers and `,` separates blocks)
## Final `K2Tree`:
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
For a more in-depth explenation of the explanation process, [check this out](HOWITWORKS.md).
# The Road to 1.0:
- [x] Make `K2Tree` work over any value of K.
- [ ]  Separate the `k` field into two distinct fields: `stem_k`, `leaf_k`.
- [ ]  Attempt to increase compression ratio by removing the `stem_to_leaf` field without compromising operation complexity.
- [ ] Unit test all the things.
- [ ] Stabilise the API.

\- GGabi