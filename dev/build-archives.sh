#!/usr/bin/env bash
# Builds self-contained Linux x86_64 archives for PostgreSQL and MySQL client tools.
# Each archive contains bin/ (the tool binaries) and lib/ (all non-glibc shared libs),
# with RPATH set to $ORIGIN/../lib so no LD_LIBRARY_PATH is needed at runtime.
#
# Output: dist/linux/postgres-{major}-x86_64.tar.xz
#         dist/linux/mysql-{major}.{minor}.{patch}-x86_64.tar.xz
#
# Requirements: docker, xz-utils
# Usage: ./dev/build-archives.sh [--postgres-only] [--mysql-only]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="$SCRIPT_DIR/../dist/linux"
mkdir -p "$DIST_DIR"

PG_VERSIONS=(13 14 15 16 17)
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

GLIBC_LIBS="libc.so|libm.so|libpthread.so|libdl.so|librt.so|libgcc_s.so|libstdc++.so|ld-linux|linux-vdso"

build_postgres() {
    local version=$1
    local out="$DIST_DIR/postgres-${version}-x86_64.tar.xz"
    echo "==> PostgreSQL $version"

    docker run --rm -v "$DIST_DIR:/output" ubuntu:22.04 bash -c "
        set -euo pipefail
        export DEBIAN_FRONTEND=noninteractive

        apt-get update -qq
        apt-get install -y -qq curl ca-certificates gnupg patchelf xz-utils binutils

        curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc \
            | gpg --dearmor -o /etc/apt/trusted.gpg.d/postgresql.gpg
        echo 'deb [signed-by=/etc/apt/trusted.gpg.d/postgresql.gpg] http://apt.postgresql.org/pub/repos/apt jammy-pgdg main' \
            > /etc/apt/sources.list.d/pgdg.list
        apt-get update -qq
        apt-get install -y -qq postgresql-client-${version}

        STAGING=\$(mktemp -d)
        mkdir -p \$STAGING/bin \$STAGING/lib

        for bin in psql pg_dump pg_restore; do
            src=\"/usr/lib/postgresql/${version}/bin/\$bin\"
            if [ -f \"\$src\" ]; then
                cp \"\$src\" \$STAGING/bin/
            else
                echo \"WARNING: \$bin not found, skipping\"
            fi
        done

        # Recursively bundle all non-glibc shared libraries (including transitive deps)
        copy_deps() {
            local target=\$1
            ldd \$target 2>/dev/null | awk '/=>/ { print \$3 }' | while read -r lib; do
                [ -f \"\$lib\" ] || continue
                basename_lib=\$(basename \"\$lib\")
                if ! echo \"\$basename_lib\" | grep -qE '${GLIBC_LIBS}'; then
                    if [ ! -f \"\$STAGING/lib/\$basename_lib\" ]; then
                        cp \"\$lib\" \$STAGING/lib/
                        copy_deps \"\$lib\"
                    fi
                fi
            done
        }
        for bin in \$STAGING/bin/*; do
            copy_deps \$bin
        done

        # Set RPATH so binaries and bundled libs find each other without LD_LIBRARY_PATH
        for bin in \$STAGING/bin/*; do
            patchelf --set-rpath '\$ORIGIN/../lib' \$bin
        done
        for lib in \$STAGING/lib/*.so*; do
            [ -f \"\$lib\" ] && patchelf --set-rpath '\$ORIGIN' \$lib 2>/dev/null || true
        done

        strip \$STAGING/bin/* 2>/dev/null || true
        strip --strip-unneeded \$STAGING/lib/*.so* 2>/dev/null || true

        cd \$STAGING
        XZ_OPT='-9 -T0' tar -cJf /output/postgres-${version}-x86_64.tar.xz .
        echo 'Created: postgres-${version}-x86_64.tar.xz'
    "
}

build_mysql() {
    local major=$1 minor=$2 patch=$3
    local full="${major}.${minor}.${patch}"
    local out="$DIST_DIR/mysql-${full}-x86_64.tar.xz"
    echo "==> MySQL $full"

    # MySQL innovation releases (9+) use a different repo component name
    local repo_component="mysql-${major}.${minor}"

    docker run --rm -v "$DIST_DIR:/output" ubuntu:22.04 bash -c "
        set -euo pipefail
        export DEBIAN_FRONTEND=noninteractive

        apt-get update -qq
        apt-get install -y -qq curl ca-certificates gnupg patchelf xz-utils binutils lsb-release

        # MySQL APT repo
        curl -fsSL https://repo.mysql.com/RPM-GPG-KEY-mysql-2023 \
            | gpg --dearmor -o /etc/apt/trusted.gpg.d/mysql.gpg
        echo 'deb [signed-by=/etc/apt/trusted.gpg.d/mysql.gpg] http://repo.mysql.com/apt/ubuntu jammy ${repo_component}' \
            > /etc/apt/sources.list.d/mysql.list
        apt-get update -qq
        apt-get install -y -qq mysql-client

        STAGING=\$(mktemp -d)
        mkdir -p \$STAGING/bin \$STAGING/lib

        for bin in mysql mysqldump; do
            src=\"\$(command -v \$bin 2>/dev/null || true)\"
            if [ -z \"\$src\" ]; then
                src=\"/usr/bin/\$bin\"
            fi
            if [ -f \"\$src\" ]; then
                cp \"\$src\" \$STAGING/bin/
            else
                echo \"WARNING: \$bin not found, skipping\"
            fi
        done

        copy_deps() {
            local target=\$1
            ldd \$target 2>/dev/null | awk '/=>/ { print \$3 }' | while read -r lib; do
                [ -f \"\$lib\" ] || continue
                basename_lib=\$(basename \"\$lib\")
                if ! echo \"\$basename_lib\" | grep -qE '${GLIBC_LIBS}'; then
                    if [ ! -f \"\$STAGING/lib/\$basename_lib\" ]; then
                        cp \"\$lib\" \$STAGING/lib/
                        copy_deps \"\$lib\"
                    fi
                fi
            done
        }
        for bin in \$STAGING/bin/*; do
            copy_deps \$bin
        done

        for bin in \$STAGING/bin/*; do
            patchelf --set-rpath '\$ORIGIN/../lib' \$bin
        done
        for lib in \$STAGING/lib/*.so*; do
            [ -f \"\$lib\" ] && patchelf --set-rpath '\$ORIGIN' \$lib 2>/dev/null || true
        done

        strip \$STAGING/bin/* 2>/dev/null || true
        strip --strip-unneeded \$STAGING/lib/*.so* 2>/dev/null || true

        cd \$STAGING
        XZ_OPT='-9 -T0' tar -cJf /output/mysql-${full}-x86_64.tar.xz .
        echo 'Created: mysql-${full}-x86_64.tar.xz'
    "
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
