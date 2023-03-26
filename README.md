# Lox

[![Build status](https://github.com/smbl64/lox/actions/workflows/ci.yml/badge.svg)](https://github.com/smbl64/lox/actions/workflows/ci.yml)

A [Lox][lox] interpreter based on the amazing [Crafting Interpreters][book] book.

## Use it

Run a Lox file via `cargo r -- filename.lox`.

## Tests

Run the test suite via:

```
cargo test
```

Test data are copied from the author's [GitHub repository][test-data]. I have modified some of the test cases, because in those cases the original one didn't make sense to me!

[book]: http://craftinginterpreters.com/contents.html
[lox]: http://craftinginterpreters.com/the-lox-language.html
[test-data]: https://github.com/munificent/craftinginterpreters/tree/master/test
