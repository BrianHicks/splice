If you have some templated files, but there is a partial failure (we're trying to access a missing argument here) the existing templated files should not change.

```console
$ splice sync
? 1

   0: [91mfailed to render `failure.txt` to `./failure.txt`[0m
   1: [91merror: Field `foo` is not defined.
       --> failure.txt:1:8
        |
      1 | {{ args.foo }}
        |         ^^^[0m

Location:
   [35msrc/module.rs[0m:[35m215[0m

Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.
Warning: SpanTrace capture is Unsupported.
Ensure that you've setup a tracing-error ErrorLayer and the semver versions are compatible

```
