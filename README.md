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

## Install

`TODO`

After `ngrep` is installed you have to import some Word Embeddings model to start matching.  
Here are the steps to use the English FastText embeddings. First download the vectors:

```bash
> curl https://dl.fbaipublicfiles.com/fasttext/vectors-crawl/cc.en.300.vec.gz
> gzip -d cc.en.300.vec.gz
```

Then import and use them:

```bash
> ngrep import --path cc.en.300.vec.gz --name ften
> echo 'hello world' | ngrep '~(hey)+ ~(planet)+'
```

Alternatively you can import any embeddings in the `txt` format and configure the default model with `ngrep config`:

 - [FastText Word vectors for 157 languages](https://fasttext.cc/docs/en/crawl-vectors.html#models)
 - [Wikipedia2Vec with ENTITY vectors](https://wikipedia2vec.github.io/wikipedia2vec/pretrained/)
 - [GloVe: Global Vectors for Word Representation](https://nlp.stanford.edu/projects/glove/)

## A note on performance

`ngrep`'s current focus is primarily on exploration, not performance (despite being built on the great 🦀 [fancy-regex](https://github.com/fancy-regex/fancy-regex) library!). For instance, it doesn't preload or cache vectors and performs numerous disk accesses, and `~` matches are not compiled into standard regex when possibile. This is a deliberate choice to provide a simple way to explore and extend this concept (small LLMs models I'm looking to you!)

To give you a glimpse of the current performance, it takes about 45 seconds to find the most common ways to refer to a big animal in the book _Moby-Dick_ on MacBook Pro M4 (1.2MB of text, 22K lines, English FastText 300d):
```
> ngrep -o '~(big) ~(animal;0.4)' moby.txt | sort | uniq -c | sort -n
   1 big whale
   1 gigantic creature
   1 great rat
   1 huge elephant
   1 huge reptile
   1 large creature
   1 large herd
   1 little fur
   1 small cub
   1 small fowl
   1 tremendous whale
   2 enormous creature
   4 large whale
  15 great whale
```
