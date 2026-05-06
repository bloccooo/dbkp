#!/usr/bin/env bash
# Builds self-contained macOS arm64 archives for PostgreSQL and MySQL client tools.
# Each archive contains bin/ (the tool binaries) and lib/ (all non-system dylibs),
# with RPATH set to @loader_path/../lib so no DYLD_LIBRARY_PATH is needed at runtime.
#
# Output: dist/macos/postgres-{major}-arm64.tar.xz
#         dist/macos/mysql-{major}.{minor}.{patch}-arm64.tar.xz
#
# Requirements: Homebrew, xz  (brew install xz)
# Usage: ./dev/build-archives-macos.sh [--postgres-only] [--mysql-only]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="$SCRIPT_DIR/../dist/macos"
mkdir -p "$DIST_DIR"

PG_VERSIONS=(14 15 16 17)
# major minor patch
MYSQL_VERSIONS=("8 4 5" "9 3 0")

BUILD_POSTGRES=true
BUILD_MYSQL=true

for arg in "$@"; do
    case $arg in
        --postgres-only) BUILD_MYSQL=false ;;
        --mysql-only)    BUILD_POSTGRES=false ;;
    esac
done

# Sanity checks
if [[ "$(uname)" != "Darwin" ]]; then
    echo "Error: this script must run on macOS." >&2
    exit 1
fi
if [[ "$(uname -m)" != "arm64" ]]; then
    echo "Error: this script targets arm64 and must run on Apple Silicon." >&2
    exit 1
fi
if ! command -v brew &>/dev/null; then
    echo "Error: Homebrew is required. Install from https://brew.sh" >&2
    exit 1
fi
if ! command -v xz &>/dev/null; then
    echo "Error: xz is required.  Run: brew install xz" >&2
    exit 1
fi

# Recursively copies non-system dylib dependencies of $target into $staging/lib/,
# fixes their install names to use @rpath, and rewrites all references in $target.
fix_dylib_refs() {
    local target=$1
    local staging=$2

    otool -L "$target" | awk 'NR>1 {print $1}' | while read -r dep; do
        [[ "$dep" == /usr/lib/*          ]] && continue
        [[ "$dep" == /System/*           ]] && continue
        [[ "$dep" == @*                  ]] && continue  # already relative

        local depname
        depname=$(basename "$dep")
        local bundled="$staging/lib/$depname"

        if [ ! -f "$bundled" ]; then
            if [ ! -f "$dep" ]; then
                echo "    WARNING: dependency not found: $dep" >&2
                continue
            fi
            cp "$dep" "$bundled"
            chmod u+w "$bundled"
            # Fix the dylib's own install name so dependents can find it via @rpath
            install_name_tool -id "@rpath/$depname" "$bundled"
            # Let this dylib resolve its own transitive deps from @loader_path
            install_name_tool -add_rpath "@loader_path" "$bundled" 2>/dev/null || true
            fix_dylib_refs "$bundled" "$staging"
        fi

        # Rewrite the reference in the target (binary or dylib being processed)
        install_name_tool -change "$dep" "@rpath/$depname" "$target"
    done
}

build_postgres() {
    local version=$1
    local out="$DIST_DIR/postgres-${version}-arm64.tar.xz"
    echo "==> PostgreSQL $version"

    local formula="postgresql@${version}"
    if ! brew list --formula "$formula" &>/dev/null; then
        echo "    Installing $formula via Homebrew..."
        if ! brew install "$formula" 2>&1; then
            echo "    SKIPPING PostgreSQL $version: brew install failed (likely EOL/disabled)."
            return 0
        fi
    fi

    local brew_prefix
    brew_prefix="$(brew --prefix "$formula")"
    local bin_dir="$brew_prefix/bin"

    local staging
    staging=$(mktemp -d)
    mkdir -p "$staging/bin" "$staging/lib"

    for bin in psql pg_dump pg_restore; do
        local src="$bin_dir/$bin"
        if [ -f "$src" ]; then
            cp "$src" "$staging/bin/$bin"
            chmod u+w "$staging/bin/$bin"
        else
            echo "    WARNING: $bin not found at $src, skipping"
        fi
    done

    for bin in "$staging/bin/"*; do
        [ -f "$bin" ] || continue
        install_name_tool -add_rpath "@loader_path/../lib" "$bin" 2>/dev/null || true
        fix_dylib_refs "$bin" "$staging"
    done

    # Strip local symbols from binaries; -x is safe for dylibs (keeps exported symbols)
    strip "$staging/bin/"* 2>/dev/null || true
    for lib in "$staging/lib/"*.dylib; do
        [ -f "$lib" ] && strip -x "$lib" 2>/dev/null || true
    done

    # Re-sign with an ad-hoc signature so Gatekeeper accepts the modified binaries
    for bin in "$staging/bin/"*; do
        [ -f "$bin" ] && codesign -f -s - "$bin" 2>/dev/null || true
    done
    for lib in "$staging/lib/"*.dylib; do
        [ -f "$lib" ] && codesign -f -s - "$lib" 2>/dev/null || true
    done

    (cd "$staging" && XZ_OPT='-9 -T0' tar -cJf "$out" .)
    rm -rf "$staging"
    echo "    Created: $(basename "$out")"
}

build_mysql() {
    local major=$1 minor=$2 patch=$3
    local full="${major}.${minor}.${patch}"
    local out="$DIST_DIR/mysql-${full}-arm64.tar.xz"
    echo "==> MySQL $full"

    local formula="mysql-client@${major}"
    if ! brew list --formula "$formula" &>/dev/null; then
        echo "    Installing $formula via Homebrew..."
        brew install "$formula"
    fi

    local brew_prefix
    brew_prefix="$(brew --prefix "$formula")"
    local bin_dir="$brew_prefix/bin"

    local staging
    staging=$(mktemp -d)
    mkdir -p "$staging/bin" "$staging/lib"

    for bin in mysql mysqldump; do
        local src="$bin_dir/$bin"
        if [ -f "$src" ]; then
            cp "$src" "$staging/bin/$bin"
            chmod u+w "$staging/bin/$bin"
        else
            echo "    WARNING: $bin not found at $src, skipping"
        fi
    done

    for bin in "$staging/bin/"*; do
        [ -f "$bin" ] || continue
        install_name_tool -add_rpath "@loader_path/../lib" "$bin" 2>/dev/null || true
        fix_dylib_refs "$bin" "$staging"
    done

    strip "$staging/bin/"* 2>/dev/null || true
    for lib in "$staging/lib/"*.dylib; do
        [ -f "$lib" ] && strip -x "$lib" 2>/dev/null || true
    done

    for bin in "$staging/bin/"*; do
        [ -f "$bin" ] && codesign -f -s - "$bin" 2>/dev/null || true
    done
    for lib in "$staging/lib/"*.dylib; do
        [ -f "$lib" ] && codesign -f -s - "$lib" 2>/dev/null || true
    done

    (cd "$staging" && XZ_OPT='-9 -T0' tar -cJf "$out" .)
    rm -rf "$staging"
    echo "    Created: $(basename "$out")"
}

if $BUILD_POSTGRES; then
    for v in "${PG_VERSIONS[@]}"; do
        build_postgres "$v"
    done
fi

if $BUILD_MYSQL; then
    while IFS=' ' read -r major minor patch; do
        build_mysql "$major" "$minor" "$patch"
    done < <(printf '%s\n' "${MYSQL_VERSIONS[@]}")
fi

echo ""
echo "All archives written to: $DIST_DIR"
ls -lh "$DIST_DIR"
