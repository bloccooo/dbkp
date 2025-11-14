# DB Backup - Database Backup & Restore Utility

A simple tool for backing up databases to various cloud storage providers or local filesystems.

Designed to make database migrations easier, this project streamlines copying, backup, and restoration operations. It consists of a **library** and a **CLI tool** that can be used both as a command-line interface for server automation and through an interactive **TUI (Terminal User Interface)** for everyday development tasks like pulling production data into your local environment.

**Important Note:** This is a side project used currently as an internal tool. It is not an industrial-grade solution. It only provides logical backup for the moment and might struggle with massive databases. Works great for development, testing, and smaller projects, but maybe don't bet your mission-critical production systems on it just yet...

If you need more advanced tools please check [Barman](https://pgbarman.org) or [pgbackrest](https://pgbackrest.org).

## Installation

### Quick Installation

#### Linux

```bash
curl -sSL https://raw.githubusercontent.com/bloccooo/dbkp/main/install-cli.sh | sudo bash
```

#### macOS

```bash
curl -sSL https://raw.githubusercontent.com/bloccooo/dbkp/main/install-cli.sh | bash
```

Note: macOS users may need to remove `sudo` depending on their permission settings.

### Install from Source (requires Rust toolchain)

```bash
# Clone the repository
git clone https://github.com/bloccooo/dbkp.git
cd dbkp

# Build the project
cd cli
cargo build --release

# Install the binary
sudo cp target/release/dbkp /usr/local/bin/dbkp
```

## TUI (Terminal User Interface)

The CLI tool includes an interactive TUI mode that provides a visual, menu-driven interface for managing backups and databases. The TUI offers the same functionality as the command-line interface with an intuitive, user-friendly experience.

### Launching the TUI

Simply run `dbkp` without any arguments to launch the TUI:

```bash
dbkp
```

The TUI provides:

- Visual database configuration
- Storage setup with validation
- Backup and restore operations
- Navigation through menus

You can exit the TUI at any time using `esc`.

## CLI Documentation

For detailed CLI usage, commands, parameters, and examples, see the [CLI Documentation](/cli/README.md).

The CLI tool supports multiple usage modes:

- **TUI Mode**: Interactive terminal user interface (launch with `dbkp`)
- **Direct Parameters**: For command-line control for automation

## Features

### Database Support

- **PostgreSQL**: Backup and restore support
- **Version Detection**: Automatic PostgreSQL version detection and compatibility

### Storage Backends

- **S3-Compatible Storage**: Amazon S3, MinIO, DigitalOcean Spaces, and other S3-compatible providers
- **Local Filesystem**: Store backups on local or network-mounted filesystems

### Backup & Restore Operations

- **Streaming Architecture**: Memory-efficient streaming for large databases
- **Logical Backups**: Full schema and data backup using `pg_dump`

### User Experience

- **TUI Mode**: Interactive terminal user interface for visual management

### Automation & Integration

- **CLI Automation**: Command-line interface for scripts and CI/CD
- **Cron Job Ready**: Designed for scheduled backup operations
- **Docker Compatible**: Works in containerized environments

## Quick Start Example

```bash
# TUI mode (recommended for first-time users)
dbkp

# Command-line backup to S3
dbkp backup \
  --database myapp \
  --host localhost \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com

# Restore latest backup
dbkp restore \
  --database myapp \
  --storage-type s3 \
  --bucket my-backups \
  --latest
```

## Architecture

The project is organized into two main components:

- **Core Library** (`/core`): The `dbkp-core` library containing database connections, backup/restore logic, and storage backends. This can be used as a dependency in other Rust projects.
- **CLI Tool** (`/cli`): The `dbkp` command-line tool that provides:
  - **TUI Mode**: Interactive terminal user interface for visual management
  - **Command-Line Mode**: Direct commands for automation and scripting

## Use Cases

### Development & Testing

- Pull production data to local development environments
- Create test data snapshots for consistent testing
- Quick database migrations between environments

### Small to Medium Production

- Automated daily/weekly backups with retention policies
- Database migrations and deployments
- Disaster recovery for smaller applications

## Limitations

- **Logical Backups Only**: Does not support physical/binary backups
- **Single Database Focus**: Optimized for individual database operations
- **Not Industrial-Grade**: Suitable for development and smaller to medium production use cases
- **PostgreSQL Focused**: Currently optimized primarily for PostgreSQL

For enterprise-grade solutions with physical backups, point-in-time recovery, and high-availability features, consider [Barman](https://pgbarman.org) or [pgbackrest](https://pgbackrest.org).

## License

MIT License - see LICENSE file for details.
