use anyhow::Result;
use log::info;
use sqlx::{AssertSqlSafe, MySql, Pool};

const MIGRATIONS: &[(&str, &str)] = &[(
    "V1__create_initial_tables.sql",
    include_str!("migrations/V1__create_initial_tables.sql"),
)];

pub async fn run_migrations(pool: &Pool<MySql>) -> Result<()> {
    info!("Applying migrations...");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    for &(filename, sql) in MIGRATIONS {
        let version = migration_version(filename)?;

        let applied: Option<i64> = sqlx::query_scalar(
            "SELECT version FROM _sqlx_migrations WHERE version = ? AND success = 1",
        )
        .bind(version)
        .fetch_optional(pool)
        .await?;

        if applied.is_some() {
            continue;
        }

        info!("Applying migration: {}", filename);
        let start = std::time::Instant::now();

        for statement in split_statements(sql) {
            match sqlx::query(AssertSqlSafe(statement.as_str()))
                .execute(pool)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    let is_create_index = statement
                        .trim_start()
                        .to_ascii_uppercase()
                        .starts_with("CREATE INDEX");

                    let is_duplicate_index = is_create_index
                        && matches!(&e, sqlx::Error::Database(db_err) if {
                            db_err.code().as_deref() == Some("1061")
                                || db_err.message().to_ascii_lowercase().contains("duplicate key name")
                        });

                    if is_duplicate_index {
                        info!("Skipped (index already exists): {}", short(&statement));
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }

        let execution_time = start.elapsed().as_millis() as i64;
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
             VALUES (?, ?, 1, ?, ?)",
        )
        .bind(version)
        .bind(filename)
        .bind(vec![0u8; 32])
        .bind(execution_time)
        .execute(pool)
        .await?;

        info!("Migration {} applied successfully", filename);
    }

    info!("Migrations applied successfully!");
    Ok(())
}

fn migration_version(filename: &str) -> Result<i64> {
    filename
        .strip_prefix('V')
        .and_then(|s| s.split("__").next())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| anyhow::anyhow!("Invalid migration file name: {}", filename))
}

fn split_statements(sql: &str) -> Vec<String> {
    let cleaned: String = sql
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("--"))
        .collect::<Vec<_>>()
        .join(" ");

    let mut statements = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;

    for ch in cleaned.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ';' if depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }
    statements
}

fn short(s: &str) -> String {
    s[..s.len().min(50)].to_string()
}
