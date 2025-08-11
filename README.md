### `ngrep`
![ngrep](./assets/ngrep.svg)
<div align="center"> <sup>ngrep matching a paragraph of <em>Flatland: A Romance of Many Dimensions</em></sup></div>

---

### Table of Contents

- [What is ngrep?](http://localhost)
- [The `~` operator](http://localhost)

## What is ngrep?
`ngrep` is an experimental way to help you find text by its meaning rather than solely by its textual similarity. It extends known regular expressions with a new _neural operator_ `~` that express matches in the space of word-embeddings, integrating with well known regular expressions operators such as `+`, `*`, `()` .

## The `~` operator

To express semantic matches, `ngrep` introduces the `~` operator. It defines a match based on a similarity in the embedding space between a given word and the provided text. For example the expression `~(fruit)+` matches any sequences of characters that is similar to `fruit`, i.e. any sequences of chars which embedding has a similairty within a given a threshold with the embedding of `fruit`, such as:

**Banana** are yellow!
**Apple** are red!
**Carrot** are orange!

_<tiny> built with ❤️ with zed, 🦀 and fancy-regex </tiny>_
