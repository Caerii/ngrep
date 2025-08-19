### `ngrep`
![ngrep](./assets/ngrep.svg)
<div align="center"> <sup><code>ngrep</code> matching a paragraph from the book <em>Flatland: A Romance of Many Dimensions</em></sup></div>

---

## What is ngrep?

`ngrep` is an experimental way to help you find text by its meaning rather than solely by syntactic matching. It extends known regular expressions with a new _neural operator_ `~` that express matches in the space of word-embeddings, integrating with well known operators such as `+`, `*`, `()` allowing you to combine semantic and literal patterns in one expression.

## The `~` operator

The `~` operator defines a match based on _semantic_ similarity. It finds text that is contextually similar to a given word by leveraging neural [Word Embeddings](https://en.wikipedia.org/wiki/Word_embedding) (yes, 2010s nostalgia).

For example, the expression `~(fruit)+` matches any sequence of characters whose Word Embedding is contextually similar to the Word Embedding of `fruit`:

![fruits](./assets/fruits.svg)

_<tiny> built with ❤️ with zed, 🦀 and fancy-regex </tiny>_
