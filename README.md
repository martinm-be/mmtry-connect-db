# connect-db

A simple Rust program that reads database connection parameters from `.vault/secrets/` files and launches `psql` to connect to the database.

## Usage

```bash
connect-db <database_name>
```

Where `<database_name>` corresponds to the configuration files:
- `.vault/secrets/<database_name>.db.json` - Connection template
- `.vault/secrets/<database_name>.db-role.json` - Credentials

## Configuration Format

### Connection Template (`.vault/secrets/<database_name>.db.json`)
```json
{
  "data": {
    "db_url": "postgresql://{{username}}:{{password}}@hostname:5432/database_name"
  }
}
```

### Credentials (`.vault/secrets/<database_name>.db-role.json`)
```json
{
  "username": "your_username",
  "password": "your_password"
}
```

## Installation

```bash
make install
```

This will build the release binary and copy it to `~/bin/connect-db`.

## Requirements

- Rust toolchain
- `psql` command available in PATH