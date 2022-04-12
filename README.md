##

### 1: Find the ISBN from Goodreads

- API is deprecated, they no longer give out API keys
- my previous API key has been deleted (inactive for 30 days)
- parsing the HTML of the "book" page to get the ISBN 10 and ISBN 13

### 2: Find the relevant books from LibGen

- The public API is hard to use, doesn't have download links (!), and has very limited documentation (https://forum.mhut.org/viewtopic.php?f=17&t=6874&sid=5e516f61ff694e5bfdc2ea129f0265d9)
- Unofficial packages and libraries exist
- https://github.com/harrison-broadbent/libgen-api looks like the best library, by far, and can make things easier compared to trying to use the LibGen API directly

I went for the libgen-api option, because I didn't want to re-invent the wheel and also because it seemed like the most fun way to solve this problem.

#### Using `ligben-api`

Since it is written in Python, my first thought was to create a simple wrapper around it, in Python, to be able to call functions remotely.
I initially thought of an HTTP API, or a gRPC + protobuf API.
The main problem I had with it, is that it made the whole architecture more
complex, added more moving parts (a Python service that needs to be running
side-by-side with this tool) and prevented the tool to be easily stand-alone.

A better solution is to call Python code directly from Rust.
For that, I found 3 solutions:
- https://github.com/PyO3/pyo3, which doesn't have a lot of documentation at all, and adds a whole new API for calling Python which is pretty annoying
- https://github.com/fusion-engineering/inline-python, which is built on top of pyo3, has a great API but absolutely no documentation whatsoever
- 
