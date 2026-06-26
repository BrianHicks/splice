Splice can pull modules from git:

```console
$ splice sync
 INFO sync:collect_modules:fetch_git: cloning repo="git@github.com:BrianHicks/splice.git" rev=None
 INFO sync:write_files: writing file="output-with-rev/all_context.txt"
 INFO sync:write_files: writing file="output-with-rev/plain_greeting.txt"
 INFO sync:write_files: writing file="output-with-rev/templated_greeting.txt"
 INFO sync:write_files: writing file="output-without-rev/all_context.txt"
 INFO sync:write_files: writing file="output-without-rev/plain_greeting.txt"
 INFO sync:write_files: writing file="output-without-rev/templated_greeting.txt"

```
