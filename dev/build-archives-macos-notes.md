# Notes for building macOS archives

## Context

dbkp auto-downloads pre-built client tool archives from S3 and caches them in
`~/.cache/vprdbbkp/{postgresql|mysql}/{version}/`. The archive layout is:

```
bin/   psql, pg_dump, pg_restore   (or mysql, mysqldump)
lib/   all non-system shared libraries the binaries depend on
```

The Linux script (`dev/build-archives.sh`) uses Docker + `patchelf` to produce
self-contained archives. macOS needs the same result but uses different tools.

## Key differences vs Linux

| Concern              | Linux                        | macOS                              |
|----------------------|------------------------------|------------------------------------|
| Build environment    | Docker (ubuntu:22.04)        | Native (must run on Apple Silicon) |
| Package manager      | apt + pgdg/mysql repos       | Homebrew                           |
| RPATH tool           | `patchelf --set-rpath`       | `install_name_tool -add_rpath`     |
| Inspect deps         | `ldd`                        | `otool -L`                         |
| RPATH token (bin)    | `$ORIGIN/../lib`             | `@loader_path/../lib`              |
| RPATH token (lib)    | `$ORIGIN`                    | `@loader_path`                     |
| Shared lib extension | `.so` / `.so.N`              | `.dylib`                           |
| Extra step (macOS)   | —                            | Fix embedded install names (see below) |

## macOS-specific: fixing dylib install names

On macOS, each dylib has an embedded "install name" (an absolute path baked in
at build time, e.g. `/opt/homebrew/opt/postgresql@16/lib/libpq.5.dylib`).
When you copy a dylib to `lib/`, its dependents still reference the old path,
so the dynamic linker can't find it.

You must rewrite those references in every binary and dylib you bundle:

```bash
# 1. Change the dylib's own install name to use @rpath
install_name_tool -id "@rpath/libpq.5.dylib" lib/libpq.5.dylib

# 2. In each binary/dylib that references it, replace the old absolute path
install_name_tool -change \
    "/opt/homebrew/opt/postgresql@16/lib/libpq.5.dylib" \
    "@rpath/libpq.5.dylib" \
    bin/psql

# 3. Add the rpath entries so the linker knows where to look
install_name_tool -add_rpath "@loader_path/../lib" bin/psql  # for binaries
install_name_tool -add_rpath "@loader_path"        lib/libpq.5.dylib  # for libs
```

A helper function that automates steps 1-3 for every lib:

```bash
fix_dylib_refs() {
    local target=$1   # binary or dylib being fixed
    otool -L "$target" | awk 'NR>1 {print $1}' | while read -r dep; do
        # Skip system libs (anything in /usr/lib or /System)
        [[ "$dep" == /usr/lib/* ]] && continue
        [[ "$dep" == /System/* ]] && continue
        [[ "$dep" == @* ]] && continue  # already uses @rpath/@loader_path

        local depname
        depname=$(basename "$dep")
        local bundled="lib/$depname"

        # Copy the lib if not already bundled, fix its own install name
        if [ ! -f "$bundled" ]; then
            cp "$dep" "$bundled"
            install_name_tool -id "@rpath/$depname" "$bundled"
            fix_dylib_refs "$bundled"   # recurse for transitive deps
        fi

        # Rewrite the reference in the target
        install_name_tool -change "$dep" "@rpath/$depname" "$target"
    done
}
```

## Homebrew binary locations

PostgreSQL (installed via `brew install postgresql@{major}`):
- Binaries: `/opt/homebrew/opt/postgresql@{major}/bin/`
- Libs:     `/opt/homebrew/opt/postgresql@{major}/lib/`

MySQL client (installed via `brew install mysql-client@{major}`):
- Binaries: `/opt/homebrew/opt/mysql-client@{major}/bin/`
- Libs:     `/opt/homebrew/opt/mysql-client@{major}/lib/`

## Binaries to include

- PostgreSQL: `psql`, `pg_dump`, `pg_restore`
- MySQL:      `mysql`, `mysqldump`

## System libs to NOT bundle (already present on all macOS installs)

Skip anything whose path starts with:
- `/usr/lib/`
- `/System/Library/`
- `@rpath/`, `@loader_path/`, `@executable_path/` (already relative)

## Output filename convention (must match metadata.json)

```
dist/macos/postgres-{major}-arm64.tar.xz      e.g. postgres-16-arm64.tar.xz
dist/macos/mysql-{major}.{minor}.{patch}-arm64.tar.xz
```

Current versions in metadata.json:
- PostgreSQL: 13, 14, 15, 16, 17
- MySQL: 8.4.5, 9.3.0

## After building

Upload to S3:
```
s3://vprdbbkp/macos/postgres-{version}-arm64.tar.xz
s3://vprdbbkp/macos/mysql-{version}-arm64.tar.xz
```

Bucket base URL:
`https://s3.pub1.infomaniak.cloud/object/v1/AUTH_f1ed7eb1a4594d268432025f27acb84f/vprdbbkp/`

## Verification

After extracting to `~/.cache/vprdbbkp/postgresql/{version}/`, run:

```bash
~/.cache/vprdbbkp/postgresql/16/bin/psql --version
~/.cache/vprdbbkp/postgresql/16/bin/pg_dump --version
```

Both should print the version without any dylib errors.
