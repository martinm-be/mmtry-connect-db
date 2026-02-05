use anyhow::{Context, Result};
use clap::Parser;
use exec::Command;
use serde::Deserialize;
use std::{env, fs};

#[derive(Parser, Debug)]
#[command(name = "connect-db")]
#[command(about = "Connect to a database using psql")]
struct Args {
    /// Database name (matches .vault/secrets/<dbname> files)
    database_name: String,
}

#[derive(Deserialize, Debug)]
struct DatabaseConfig {
    data: DatabaseData,
}

#[derive(Deserialize, Debug)]
struct DatabaseData {
    db_url: String,
}

#[derive(Deserialize, Debug)]
struct DatabaseCredentials {
    username: String,
    password: String,
}

#[derive(Debug)]
struct ConnectionParams {
    host: String,
    port: String,
    username: String,
    password: String,
    database: String,
}

fn load_database_config(database_name: &str) -> Result<(DatabaseConfig, DatabaseCredentials)> {
    let config_path = format!(".vault/secrets/{}.db.json", database_name);
    let creds_path = format!(".vault/secrets/{}.db-role.json", database_name);

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path))?;

    let creds_content = fs::read_to_string(&creds_path)
        .with_context(|| format!("Failed to read credentials file: {}", creds_path))?;

    let config: DatabaseConfig = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {}", config_path))?;

    let credentials: DatabaseCredentials = serde_json::from_str(&creds_content)
        .with_context(|| format!("Failed to parse credentials file: {}", creds_path))?;

    Ok((config, credentials))
}

fn parse_connection_url(db_url: &str) -> Result<ConnectionParams> {
    // Parse URL like: postgresql://username:password@host:port/database
    let url = db_url
        .strip_prefix("postgresql://")
        .or_else(|| db_url.strip_prefix("postgres://"))
        .with_context(|| format!("Invalid PostgreSQL URL format: {}", db_url))?;

    // Split by '@' to separate auth from host
    let parts: Vec<&str> = url.split('@').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid URL format: missing '@' separator"));
    }

    let auth_part = parts[0];
    let host_part = parts[1];

    // Parse auth (username:password)
    let auth_parts: Vec<&str> = auth_part.split(':').collect();
    if auth_parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid auth format: expected 'username:password'"
        ));
    }
    let username = auth_parts[0].to_string();
    let password = auth_parts[1].to_string();

    // Parse host part (host:port/database)
    let host_db_parts: Vec<&str> = host_part.split('/').collect();
    if host_db_parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid host format: expected 'host:port/database'"
        ));
    }

    let host_port = host_db_parts[0];
    let database = host_db_parts[1].to_string();

    // Parse host:port
    let host_port_parts: Vec<&str> = host_port.split(':').collect();
    if host_port_parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid host format: expected 'host:port'"));
    }

    let host = host_port_parts[0].to_string();
    let port = host_port_parts[1].to_string();

    Ok(ConnectionParams {
        host,
        port,
        username,
        password,
        database,
    })
}

fn connect_with_psql(params: &ConnectionParams) -> Result<()> {
    let conn_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        params.username, params.password, params.host, params.port, params.database
    );
    println!("Connection string: {}", conn_string);
    println!(
        "Connecting to database '{}' at {}:{}",
        params.database, params.host, params.port
    );

    let mut cmd = Command::new("psql");
    cmd.arg("-h")
        .arg(&params.host)
        .arg("-p")
        .arg(&params.port)
        .arg("-U")
        .arg(&params.username)
        .arg("-d")
        .arg(&params.database);

    // Set PGPASSWORD environment variable
    unsafe {
        env::set_var("PGPASSWORD", &params.password);
    }

    // This will replace the current process with psql
    // If successful, this function will never return
    let err = cmd.exec();

    // If we reach this point, exec failed
    Err(anyhow::anyhow!("Failed to exec psql: {}", err))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Load database configuration and credentials
    let (config, credentials) = load_database_config(&args.database_name)?;

    // Substitute placeholders in the database URL
    let database_url = config
        .data
        .db_url
        .replace("{{username}}", &credentials.username)
        .replace("{{password}}", &credentials.password);

    // Parse connection parameters
    let params = parse_connection_url(&database_url)?;

    // Connect using psql
    connect_with_psql(&params)?;

    Ok(())
}
