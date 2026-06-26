Conflicting paths should produce an error.

```console
$ splice sync
? 1

   0: failed to render files
   1: Output already contains `./test.txt`.

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   0: splice::sync::render_templates
      at src/sync.rs:[..]
   1: splice::sync::sync
      at src/sync.rs:[..]

```
