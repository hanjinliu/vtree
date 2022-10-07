# vtree

`vtree` is a crate for building virtual file tree system under any directories.

### Installation

```
$ cargo install --git https://github.com/hanjinliu/vtree
```

### When do I need virtual file trees?

- Many files are stored under a directory.
- The files will not be moved.
- You want to organize them in different file trees but don't want to create shortcuts all the way.
- You want to add some description to some of the directories.

##### Example 1

Under the directory where many experimental data are stored,

```
Data
  :
  ├─ 221006
  │    ├─ experiment_221006-A.csv
  │    ├─ experiment_221006-B.csv
  │    └─ experiment_221006-C.csv
  ├─ 221007
  │    ├─ experiment_221007-A.csv
  │    ├─ experiment_221007-B.csv
  │    └─ experiment_221007-C.csv
  └─ 221008
       ├─ experiment_221008-A.csv
       ├─ experiment_221008-B.csv
       └─ experiment_221008-C.csv
```

create a virtual file tree for certain project.

```
Project_A
  ├─ <shortcut to ./221006/experiment_221006-A.csv>
  ├─ <shortcut to ./221007/experiment_221007-A.csv>
  └─ <shortcut to ./221008/experiment_221008-A.csv>
```