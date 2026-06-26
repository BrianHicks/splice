If you have some templated files, but there is a partial failure (we're trying to access a missing argument here) the existing templated files should not change.

```console
$ splice sync
? 1

   0: failed to render files
   1: failed to render `failure.txt` to `./failure.txt`
   2: error: Field `foo` is not defined.
       --> failure.txt:1:9
        |
      1 | {{ args.foo }}
        |         ^^^

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   0: splice::module::render with filename="./failure.txt" template="failure.txt"
      at src/module.rs:[..]
   1: splice::module::render_all
      at src/module.rs:[..]
   2: splice::sync::render_templates
      at src/sync.rs:[..]
   3: splice::sync::sync
      at src/sync.rs:[..]

```
