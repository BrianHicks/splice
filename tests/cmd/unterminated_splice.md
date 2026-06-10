A splice block that is opened but never closed (no `SPLICE END`) must be a hard
error. Silently dropping the block's contents is exactly the data loss that
splices exist to prevent, so we refuse rather than write. The original file is
left untouched (see `unterminated_splice.out/config.txt`).

```console
$ splice sync
? 1

   0: failed to collect splices
   1: in `./config.txt`
   2: Reached the end of the file while still inside the `section` splice (missing SPLICE END).

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   0: splice::module::collect_splices
      at src/module.rs:[..]
   1: splice::sync::collect_splices
      at src/sync.rs:[..]
   2: splice::sync::sync
      at src/sync.rs:[..]

```
