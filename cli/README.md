# DBKP - Database Backup & Restore CLI

A command-line tool for backing up and restoring PostgreSQL and MySQL databases with support for multiple storage backends and an interactive TUI (Terminal User Interface).

## Quick Start

### Installation

See the main [README.md](../README.md) for installation instructions.

### First Run

The easiest way to get started is with the TUI mode:

```bash
dbkp
```

This launches an interactive TUI that guides you through database and storage configuration, and running backups.

## Usage Modes

### 1. TUI Mode (Recommended for Beginners)

Simply run `dbkp` without arguments to launch the TUI:

```bash
dbkp
```

**Features:**

- Database configuration
- Storage setup
- Backup/restore
- Navigation through menus

### 2. Command-Line Mode (For Automation)

Specify all parameters directly in the command:

```bash
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location myapp-backups
```

## Commands Overview

| Command        | Description                  |
| -------------- | ---------------------------- |
| `dbkp`         | Launch TUI mode              |
| `dbkp backup`  | Create database backup       |
| `dbkp restore` | Restore database from backup |
| `dbkp list`    | List available backups       |
| `dbkp cleanup` | Remove old backups           |

## Backup Operations

**PostgreSQL to Local Storage:**

```bash
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --password secret \
  --storage-type local \
  --location /backups/myapp
```

**MySQL to S3:**

```bash
dbkp backup \
  --database-type mysql \
  --database myapp \
  --host db.example.com \
  --port 3306 \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAIOSFODNN7EXAMPLE \
  --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
  --location myapp-backups
```

**With SSH Tunnel:**

```bash
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host 10.0.0.100 \
  --port 5432 \
  --username dbuser \
  --ssh-host bastion.example.com \
  --ssh-username ubuntu \
  --ssh-key-path ~/.ssh/id_rsa \
  --storage-type local \
  --location /backups
```

**With Retention Period:**

```bash
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type local \
  --location /backups/myapp \
  --retention 30d
```

## Restore Operations

**Restore Latest Backup:**

```bash
dbkp restore \
  --database-type postgresql \
  --database myapp_restore \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type local \
  --location /backups \
  --latest
```

**Restore Specific Backup:**

```bash
dbkp restore \
  --database-type postgresql \
  --database myapp_restore \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location myapp-backups \
  --name myapp-2024-01-15-143022-a1b2c3d4.gz \
  --drop-database
```

## List Backups

**Local Storage:**

```bash
dbkp list \
  --storage-type local \
  --location /backups
```

**S3 Storage:**

```bash
dbkp list \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location myapp-backups
```

**Filter by Database:**

```bash
dbkp list \
  --storage-type local \
  --location /backups \
  --database myapp
```

**Show Only Latest:**

```bash
dbkp list \
  --storage-type local \
  --location /backups \
  --database myapp \
  --latest-only
```

**Limit Results:**

```bash
dbkp list \
  --storage-type local \
  --location /backups \
  --limit 20
```

## Cleanup Operations

**Cleanup Old Backups:**

```bash
dbkp cleanup \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location myapp-backups \
  --retention 30d
```

**Dry Run (Preview What Would Be Deleted):**

```bash
dbkp cleanup \
  --storage-type local \
  --location /backups \
  --retention 7d \
  --dry-run
```

**Cleanup Specific Database:**

```bash
dbkp cleanup \
  --storage-type local \
  --location /backups \
  --database myapp \
  --retention 30d
```

## Parameter Reference

### Database Connection

| Parameter         | Description                           | Required | Default |
| ----------------- | ------------------------------------- | -------- | ------- |
| `--database-type` | Database type (`postgresql`, `mysql`) | Yes      | -       |
| `--database`      | Database name                         | Yes      | -       |
| `--host`          | Database host                         | Yes      | -       |
| `--port`          | Database port                         | Yes      | -       |
| `--username`      | Database username                     | Yes      | -       |
| `--password`      | Database password                     | No       | -       |

### SSH Tunnel

| Parameter        | Description          | Required           | Default |
| ---------------- | -------------------- | ------------------ | ------- |
| `--ssh-host`     | SSH host             | Yes (if using SSH) | -       |
| `--ssh-username` | SSH username         | Yes (if using SSH) | -       |
| `--ssh-key-path` | SSH private key path | Yes (if using SSH) | -       |

### Storage - Local

| Parameter        | Description             | Required | Default   |
| ---------------- | ----------------------- | -------- | --------- |
| `--storage-type` | Set to `local`          | Yes      | `local`   |
| `--location`     | Directory path          | Yes      | -         |
| `--storage-name` | Storage name identifier | No       | `default` |

### Storage - S3

| Parameter        | Description             | Required | Default     |
| ---------------- | ----------------------- | -------- | ----------- |
| `--storage-type` | Set to `s3`             | Yes      | -           |
| `--bucket`       | S3 bucket name          | Yes      | -           |
| `--endpoint`     | S3 endpoint URL         | Yes      | -           |
| `--access-key`   | S3 access key           | Yes      | -           |
| `--secret-key`   | S3 secret key           | Yes      | -           |
| `--location`     | Prefix/folder in bucket | Yes      | -           |
| `--region`       | S3 region               | No       | `us-east-1` |
| `--storage-name` | Storage name identifier | No       | `default`   |

### Backup Options

| Parameter     | Description                               | Required | Default |
| ------------- | ----------------------------------------- | -------- | ------- |
| `--retention` | Retention period (e.g. `30d`, `1w`, `6m`) | No       | -       |

### Restore Options

| Parameter         | Description                  | Required | Default |
| ----------------- | ---------------------------- | -------- | ------- |
| `--name`          | Specific backup to restore   | No\*     | -       |
| `--latest`        | Use most recent backup       | No\*     | `false` |
| `--drop-database` | Drop database before restore | No       | `false` |

\*Either `--name` or `--latest` is required for restore operations.

### List Options

| Parameter       | Description                 | Required | Default |
| --------------- | --------------------------- | -------- | ------- |
| `--database`    | Filter by database name     | No       | -       |
| `--latest-only` | Show only the latest backup | No       | `false` |
| `--limit`       | Maximum number of results   | No       | `10`    |

### Cleanup Options

| Parameter     | Description                           | Required | Default |
| ------------- | ------------------------------------- | -------- | ------- |
| `--retention` | Keep backups newer than this          | Yes      | -       |
| `--dry-run`   | Show what would be deleted            | No       | `false` |
| `--database`  | Cleanup backups for specific database | No       | -       |

## Environment Variables

| Variable                                  | Description         | CLI Equivalent |
| ----------------------------------------- | ------------------- | -------------- |
| `PGPASSWORD`                              | PostgreSQL password | `--password`   |
| `S3_BUCKET`                               | S3 bucket name      | `--bucket`     |
| `S3_ENDPOINT`                             | S3 endpoint URL     | `--endpoint`   |
| `S3_ACCESS_KEY` or `S3_ACCESS_KEY_ID`     | S3 access key       | `--access-key` |
| `S3_SECRET_KEY` or `S3_SECRET_ACCESS_KEY` | S3 secret key       | `--secret-key` |
| `S3_REGION`                               | S3 region           | `--region`     |

### Using Environment Variables

```bash
# Set environment variables
export PGPASSWORD=mysecret
export S3_BUCKET=my-backups
export S3_ACCESS_KEY=AKIAKEY
export S3_SECRET_KEY=SECRET

# Use in commands (parameters not needed)
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type s3 \
  --endpoint https://s3.amazonaws.com \
  --location myapp-backups
```

## Backup Naming Convention

Backups are automatically named with timestamps:

```
{database-name}-{YYYY-MM-DD-HHMMSS}-{uuid}.{extension}
```

Example:

```
myapp-2024-01-15-143022-a1b2c3d4.gz
```

## Retention Periods

Specify how long to keep backups:

- `30d` - 30 days
- `4w` - 4 weeks
- `6m` - 6 months (calculated as 30 days each)
- `1y` - 1 year (calculated as 365 days)

Examples:

```bash
# Keep backups for 30 days
dbkp cleanup --storage-type local --location /backups --retention 30d

# Keep backups for 6 months
dbkp cleanup --storage-type s3 --bucket my-backups --endpoint https://s3.amazonaws.com --access-key KEY --secret-key SECRET --location backups --retention 6m
```

## Automation Examples

### Cron Job

```bash
# Daily backup at 2 AM
0 2 * * * /usr/local/bin/dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location production-backups \
  2>&1 | logger -t dbkp

# Weekly cleanup
0 3 * * 0 /usr/local/bin/dbkp cleanup \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location production-backups \
  --retention 30d \
  2>&1 | logger -t dbkp
```

### Systemd Timer

Create `/etc/systemd/system/dbkp-backup.service`:

```ini
[Unit]
Description=Database Backup
After=network.target

[Service]
Type=oneshot
User=dbkp
Environment="PGPASSWORD=secret"
Environment="S3_ACCESS_KEY=AKIAKEY"
Environment="S3_SECRET_KEY=SECRET"
ExecStart=/usr/local/bin/dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --location production-backups
```

Create `/etc/systemd/system/dbkp-backup.timer`:

```ini
[Unit]
Description=Run database backup daily
Requires=dbkp-backup.service

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

Enable:

```bash
sudo systemctl enable dbkp-backup.timer
sudo systemctl start dbkp-backup.timer
```

### CI/CD Pipeline

**GitHub Actions:**

```yaml
name: Database Backup
on:
  schedule:
    - cron: "0 2 * * *"

jobs:
  backup:
    runs-on: ubuntu-latest
    steps:
      - name: Backup Database
        run: |
          dbkp backup \
            --database-type postgresql \
            --database ${{ secrets.DB_NAME }} \
            --host ${{ secrets.DB_HOST }} \
            --port 5432 \
            --username ${{ secrets.DB_USER }} \
            --password ${{ secrets.DB_PASSWORD }} \
            --storage-type s3 \
            --bucket ${{ secrets.S3_BUCKET }} \
            --endpoint ${{ secrets.S3_ENDPOINT }} \
            --access-key ${{ secrets.S3_ACCESS_KEY }} \
            --secret-key ${{ secrets.S3_SECRET_KEY }} \
            --location production-backups
```

**GitLab CI:**

```yaml
backup:
  stage: backup
  script:
    - dbkp backup \
      --database-type postgresql \
      --database $DB_NAME \
      --host $DB_HOST \
      --port 5432 \
      --username $DB_USER \
      --password $DB_PASSWORD \
      --storage-type s3 \
      --bucket $S3_BUCKET \
      --endpoint $S3_ENDPOINT \
      --access-key $S3_ACCESS_KEY \
      --secret-key $S3_SECRET_KEY \
      --location production-backups
  only:
    - schedules
```

### Docker Usage

**Dockerfile:**

```dockerfile
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    postgresql-client \
    mysql-client \
    curl

# Install dbkp
COPY dbkp /usr/local/bin/
RUN chmod +x /usr/local/bin/dbkp

# Create backup script
COPY backup.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/backup.sh

CMD ["/usr/local/bin/backup.sh"]
```

**backup.sh:**

```bash
#!/bin/bash
set -e

dbkp backup \
    --database-type postgresql \
    --database ${DB_NAME} \
    --host ${DB_HOST} \
    --port ${DB_PORT:-5432} \
    --username ${DB_USER} \
    --password ${DB_PASSWORD} \
    --storage-type s3 \
    --bucket ${S3_BUCKET} \
    --endpoint ${S3_ENDPOINT} \
    --access-key ${S3_ACCESS_KEY} \
    --secret-key ${S3_SECRET_KEY} \
    --location ${S3_LOCATION}
```

**docker-compose.yml:**

```yaml
version: "3.8"
services:
  db-backup:
    build: .
    environment:
      - DB_NAME=myapp
      - DB_HOST=database
      - DB_USER=dbuser
      - DB_PASSWORD=secret
      - S3_BUCKET=my-backups
      - S3_ENDPOINT=https://s3.amazonaws.com
      - S3_ACCESS_KEY=AKIAKEY
      - S3_SECRET_KEY=SECRET
      - S3_LOCATION=production-backups
    depends_on:
      - database
```

## Troubleshooting

### Common Issues

**Database Connection Failed**

```bash
[ERROR] Failed to connect to database
```

**Solutions:**

- Verify host, port, username, password
- Check database server is running
- Test connection: `psql -h host -p port -U username -d database`
- Check firewall settings

**Storage Connection Failed**

```bash
[ERROR] Failed to connect to storage
```

**Solutions:**

- Verify S3 credentials and permissions
- Check bucket exists and is accessible
- Test endpoint URL
- For local storage, verify directory permissions

**No Backups Found**

```bash
[ERROR] No backups found in storage
```

**Solutions:**

- Check storage location/bucket path
- Verify backup naming convention
- List files manually to confirm presence

**Permission Denied**

```bash
[ERROR] Permission denied
```

**Solutions:**

- Check file/directory permissions
- Verify user has read/write access
- For S3, check IAM permissions

**pg_dump/mysqldump Not Found**

```bash
[ERROR] Failed to execute pg_dump: command not found
```

**Solutions:**

- Install PostgreSQL client: `sudo apt-get install postgresql-client`
- Install MySQL client: `sudo apt-get install mysql-client`
- On macOS: `brew install postgresql mysql-client`

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type local \
  --location /backups
```

### Test Connections

Use TUI mode to test connections before running automated backups:

```bash
dbkp
# Configure database and storage settings
# Test backup/restore manually
```

## Performance Tips

### Large Databases

For databases > 100GB:

- Use direct network connection (avoid SSH tunnels)
- Run backups during low-traffic periods
- Consider using parallel backup tools for very large databases
- Monitor disk space on both source and destination

### Network Optimization

- Use compression for remote backups
- Consider regional S3 endpoints for faster uploads
- Test network bandwidth before scheduling frequent backups

### Storage Optimization

- Use lifecycle policies on S3 for automatic archival
- Implement retention policies to manage storage costs
- Monitor backup sizes over time

## Related Tools

- **PostgreSQL**: [pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html), [Barman](https://pgbarman.org)
- **MySQL**: [mysqldump](https://dev.mysql.com/doc/refman/8.0/en/mysqldump.html)
- **Enterprise**: [pgbackrest](https://pgbackrest.org)
- **Monitoring**: Integrate with your monitoring stack for backup success/failure alerts

## License

MIT License - See LICENSE file for details.
