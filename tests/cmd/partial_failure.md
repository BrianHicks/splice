If you have some templated files, but there is a partial failure (we're trying to access a missing argument here) the existing templated files should not change.

```console
$ splice sync
? 1

   0: [91mFailed to render 'failure.txt'[0m
   1: [91mVariable `args.foo` not found in context while rendering 'failure.txt'[0m

Location:
   [35msrc/module.rs[0m:[35m90[0m

Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.
Warning: SpanTrace capture is Unsupported.
Ensure that you've setup a tracing-error ErrorLayer and the semver versions are compatible

```
