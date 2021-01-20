
[![Build Status](https://github.com/GGabi/k2_tree/workflows/build%20&%20tests/badge.svg)](https://github.com/GGabi/k2_tree/actions)
[![](http://meritbadge.herokuapp.com/k2_tree)](https://crates.io/crates/k2_tree)
[![API](https://docs.rs/k2_tree/badge.svg)](https://docs.rs/k2_tree)

# k2_tree
A collection designed to efficiently compress sparsely-populated bit-matrices.

See the original proposal [here](https://users.dcc.uchile.cl/~gnavarro/ps/spire09.1.pdf).

**Note:** This library heavily relies upon [bitvec](https://docs.rs/bitvec/0.17.4/bitvec/) to optimally store its data, which is very slow when compiled without optimisations!

## Usage
Add  `k2_tree`  into your project dependencies:
```toml
[dependencies]
k2_tree = "0.5.1"
```

# When `K2Tree`s are Useful:
`K2Tree`s are extremely efficient at representing data that can be encoded as a two-dimensional bit-matrix, especially if said matrix is sparsely populated.

Take a real-world example: representing Web-Graphs.

Hyperlinks between webpages can be encoded as a 2-d bit-matrix, where each column/row corresponds to a specific page and each bit denotes whether two pages are joined via a hyperlink; 1 if yes, 0 if not.

These adjacency-matrices tend to be extremely sparse ***most of the time***, making the `K2Tree` the perfect structure for encoding them!

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
  stem_k: 2, // usize
  leaf_k: 2, // usize
  max_slayers: 2, // usize
  stems: [0111110111000100], // BitVec
  leaves: [100010110010101010001100], // BitVec
}
```
For a more in-depth explanation of the compression process, [check this out](HOWITWORKS.md).
# The Road to 1.0:
- [x] Make `K2Tree` work over any value of K.
- [x] Separate the `k` field into two distinct fields: `stem_k`, `leaf_k`.
- [x] Increase compression ratio by removing the `stem_to_leaf` and `slayer_starts` field without compromising operation complexity.
- [x] Implement serde's `Serialize` and `Deserialize` traits.
- [ ] Unit test all the things.
- [ ] Stabilise the API.

\- GGabi
