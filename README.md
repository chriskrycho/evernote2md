# evernote2md

A *very* simple script in Rust to convert Evernote `.enex` export files to Markdown files.

It mostly works!

I may or may not expand it later; I mostly had the idea and wanted to see how far I could get in an afternoon. (Pretty far! The Rust ecosystem is great!)

Feel free to do whatever you want with it.

## How to use it

If you feel like using it (and consider yourself warned: I wasn't joking when I said this was an afternoon project and that it only *mostly* works):

- install [pandoc](http://pandoc.org)
- clone this crate, and run `cargo build --release` in the root of the repo
- run `./target/release/evernote2md <path to .enex file> <output directory>`
