A splice block that is opened but never closed (no `SPLICE END`) must be a hard
error. Silently dropping the block's contents is exactly the data loss that
splices exist to prevent, so we refuse rather than write. The original file is
left untouched (see `unterminated_splice.out/config.txt`).

```console
$ splice sync
? 1

   0: [91min `./config.txt`[0m
   1: [91mReached the end of the file while still inside the `section` splice (missing SPLICE END).[0m

Location:
   [35msrc/module.rs[0m:[35m186[0m

Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```
