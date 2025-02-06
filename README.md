# Building for development

```sh
$ just setup debug

# This command:
# 1. always re-builds rust SO
# 2. conditionally re-compiles cpp files if there are changes
# 3. always re-links final binary (because of the change to SO)
# 4. launches binary
$ just dev
```

`compile_commands.json` is automatically generated by Meson, so make sure to pass `-compile-commands-dir=builddir` to `clangd`.

# Building for packaging

```sh
$ just setup release
$ just install $PWD/installation-dir
$ tree $PWD/installation-dir
<PWD>/installation-dir
└── usr
    ├── lib
    │   └── x86_64-linux-gnu
    │       └── liblayer_shell_io.so
    └── local
        └── bin
            └── layer-shell
```
