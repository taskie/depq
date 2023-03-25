# depq

![Example](images/example.gif)

## Examples

### Show

```sh-session
$ cat test.txt
a b
b c
b d
b e
c e
$ depq show test.txt
a b
b c
b d
b e
c e
$ depq show -t json test.txt
{"a":["b"],"b":["c","d","e"],"c":["e"]}
$ depq show -t dot test.txt
digraph {
    n0 [label="a"];
    n1 [label="b"];
    n2 [label="c"];
    n3 [label="d"];
    n4 [label="e"];

    n0 -> n1;
    n1 -> n2;
    n1 -> n3;
    n1 -> n4;
    n2 -> n4;
}
```

### DFS

```sh-session
$ depq dfs test.txt
a b 1
b c 2
c e 3
b d 2
b e 2
$ depq dfs -T test.txt
* a
    * b
        * c
            * e
        * d
        * e
$ depq dfs -P test.txt
a
a b
a b c
a b c e
a b d
a b e
```

### BFS

```sh-session
$ depq bfs test.txt
a b 1
b c 2
b d 2
b e 2
c e 3
$ depq bfs -P test.txt
a
a b
a b c
a b d
a b e
a b c e
```

## License

MIT or Apache-2.0
