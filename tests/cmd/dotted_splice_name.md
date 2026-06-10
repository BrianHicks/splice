Splice names may contain `.` (e.g. `section.one`). The content of the block must
be preserved across a sync, not silently dropped (see `dotted_splice_name.out/config.txt`).

```console
$ splice sync
 INFO sync:write_files: writing file="./config.txt"

```
