[![codecov](https://codecov.io/gh/tsauvajon/libreads/branch/master/graph/badge.svg?token=dmbsZKZho2)](https://codecov.io/gh/tsauvajon/libreads)

## LibReads

LibReads is a tool used to simplify finding download links, from LibGen, of books found on Goodreads.

In: a Goodreads book URL.
Out: an IPFS download link for the book, hopefully with the Mobi format, or in other good formats
if available.

Of course, you need to only use this for public domain books, not for any copyrighted material.

## Requirements

You'll need [ebook-convert](https://manual.calibre-ebook.com/generated/en/ebook-convert.html) installed
and available (hint: try `which ebook-convert`).

You can install it on MacOS as part of the great [Calibre](https://calibre-ebook.com/) suite,
with `brew install --cask calibre`.

## What does it do? How does it work?

### 1: Find the ISBN from Goodreads

- API is deprecated, they no longer give out API keys
- my previous API key has been deleted (inactive for 30 days)
- parsing the HTML of the "book" page to get the ISBN 10 and ISBN 13

### 2: Find the relevant books from LibGen

- The public API is hard to use, doesn't have download links (!), and has very limited documentation (https://forum.mhut.org/viewtopic.php?f=17&t=6874&sid=5e516f61ff694e5bfdc2ea129f0265d9)
- Unofficial packages and libraries exist
- https://github.com/harrison-broadbent/libgen-api looks like the best library, by far, and can make things easier compared to trying to use the LibGen API directly

I initially went for the libgen-api option, because I didn't want to re-invent the wheel and also because it seemed like the most fun way to solve this problem.

#### Using `ligben-api`

Since it is written in Python, my first thought was to create a simple wrapper around it, in Python, to be able to call functions remotely.
I initially thought of an HTTP API, or a gRPC + protobuf API.
The main problem I had with it, is that it made the whole architecture more
complex, added more moving parts (a Python service that needs to be running
side-by-side with this tool) and prevented the tool to be easily stand-alone.

A better solution is to call Python code directly from Rust.
For that, I found a couple of solutions:
- https://github.com/PyO3/pyo3, which doesn't have a lot of documentation at all, and adds a whole new API for calling Python which is pretty annoying
- https://github.com/fusion-engineering/inline-python, which is built on top of pyo3, has a great API but absolutely no documentation whatsoever
- https://github.com/indygreg/PyOxidizer, which seems solid but doesn't really fit my need

Inline-Python didn't even compile for me, so I went with PyO3.

I was able to make it work, and call the Python code and get some download links.
The only problem is that it doesn't allow searching by ISBN, but only title and author.

I opened a [PR](https://github.com/harrison-broadbent/libgen-api/pull/26), but in the meantime
I reverted to using the API.

### Using the LibGen JSON API

We're back with our initial problem: the API is not well documented (but it's relatively easy
to figure simple searches out), and it doesn't include download links.

By browsing the website, I saw that the Mirror 1 (the one I always use) has URLs
that look reproducible:
http://library.lol/main/AB13556B96D473C8DFAD7165C4704526

The last part looks like a hash or an ID of some sort. By querying the LibGen API,
I was able to find that `AB13556B96D473C8DFAD7165C4704526` is the MD5 hash of the book.

The page contains some download links.

## 3. Download the books

As explained above, the library.lol page contains an HTTP download link, and
four IPFS download links, which is exactly what I need.

My priority for these links is:
IPFS.io > Cloudflare > Infura > Pinata > HTTP.

In other words, I'll try downloading the book from IPFS.io, if the download fails
for any reason, fall back to Cloudflare etc.

## 4. Convert books to Mobi

I need to dig further into it, but I generally use Calibre to convert ebooks from Epub to Mobi.
I'm thinking of giving https://manual.calibre-ebook.com/generated/en/ebook-convert.html#mobi-output-options a try,
to convert my Epub books into Mobi

## 5. Sending to Kindle

TODO


# Todos

Always fall back:
- if all download links fail for a book, pick the next one in the list instead of exiting
- if we can't find the MD5, the librarylol link or anything else for a book, use the next one
- if we can't find the book by ISBN, find it by title and author
- if we can't find the ISBN on Goodreads, same as above
