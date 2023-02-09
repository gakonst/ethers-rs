# The ethers-rs book

Everything about `ethers-rs`. Work-in-progress. View online here: <https://www.gakonst.com/ethers-rs>

## Contributing

The book is built with [mdbook](https://github.com/rust-lang/mdBook), which you can install by running `cargo install mdbook`.

To view changes live, run:

```sh
mdbook serve
```

Or with docker:

```sh
docker run -p 3000:3000 -v `pwd`:/book peaceiris/mdbook serve
```

To add a new section (file) to the book, add it to [`SUMMARY.md`](./SUMMARY.md).
