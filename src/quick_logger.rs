use clap::{Parser, Subcommand};
//log struct

use chrono::{DateTime, Local};
use sqlx::{pool, sqlite::SqlitePool, FromRow};

struct QuLog {
    text: String,
    tags: Vec<String>,
    create_date: DateTime<Local>,
}

struct DBConfig {
    in_memory: bool,
}

#[derive(Debug, FromRow)]
struct QuLogDBO {
    text: String,
    #[sqlx(default)]
    tags: String,
    create_date: String,
}

impl QuLogDBO {
    fn to_log(&self) -> QuLog {
        // println!("####   {}", self.create_date);
        QuLog {
            text: self.text.clone(),
            tags: self.tags.split(",").map(|s| s.to_string()).collect(),
            create_date: DateTime::parse_from_rfc3339(&self.create_date)
                .expect("Unable to format date")
                .with_timezone(&Local),
        }
    }

    fn from_log(log: &QuLog) -> Self {
        QuLogDBO {
            text: log.text.clone(),
            tags: log.tags.join(","),
            create_date: log.create_date.to_rfc3339(),
        }
    }
}

//create log from cli
#[derive(Debug, Parser)]
#[command(name = "qulog")]
struct LogCreateCli {
    #[command(subcommand)]
    command: LogCreateCommand,
}

#[derive(Debug, Subcommand)]
enum LogCreateCommand {
    Create {
        text: String,
        #[arg(long, short)]
        tags: Option<String>,
    },

    Show {
        #[arg(long)]
        tags: Option<String>,
        #[arg(long, short)]
        start_date: Option<String>,
        #[arg(long, short)]
        end_date: Option<String>,
    },
}

pub async fn run_qulog() {
    let args = LogCreateCli::parse();

    let pool = match connect_to_db(DBConfig { in_memory: false }).await {
        Ok(pool) => pool,
        Err(err) => panic!("{:?}", err),
    };

    create_log_table_if_not_exists(&pool)
        .await
        .expect("Unable to create log data base");

    match args.command {
        LogCreateCommand::Create { text, tags } => {
            let tags = match tags {
                Some(ref tags) => tags
                    .split(",")
                    .map(String::from)
                    .filter(|s| !s.is_empty())
                    .collect(),
                None => Vec::new(),
            };

            let log = QuLog {
                text,
                tags,
                create_date: Local::now(),
            };

            match create_log(&log, &pool).await {
                Ok(_) => (),
                Err(err) => println!("Unable to save log {:?}", err),
            }
        }

        LogCreateCommand::Show {
            tags,
            start_date,
            end_date,
        } => {
            let logs = fetch_logs(&pool, Vec::new(), None, None)
                .await
                .expect("Unable to fetch logs");
            for log in logs {
                println!(
                    "-> {} : {} [{}]",
                    log.create_date.format("%d-%m-%Y %H:%M"),
                    log.text,
                    log.tags.join("-")
                );
            }
        }
    }
}

async fn create_log_table_if_not_exists(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS qu_log(
            id  INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            tags TEXT DEFAULT '',
            create_date TEXT NOT NULL
        )
    "#,
    )
    .execute(pool)
    .await?;
    return Ok(());
}

async fn create_log(model: &QuLog, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let db_model = QuLogDBO::from_log(model);
    sqlx::query("INSERT INTO qu_log (text, create_date, tags) VALUES ($1, $2, $3)")
        .bind(db_model.text)
        .bind(db_model.create_date)
        .bind(db_model.tags)
        .execute(pool)
        .await?;
    Ok(())
}

async fn connect_to_db(config: DBConfig) -> Result<SqlitePool, sqlx::Error> {
    if config.in_memory {
        return SqlitePool::connect("sqlite::memory").await;
    }

    let dbname = "logs.db";

    let db_file_path = std::env::current_dir()?.join(dbname);
    if !db_file_path.exists() {
        std::fs::File::create(db_file_path).expect("Unable to create db file");
    }

    let db_path = format!("sqlite://{}", dbname);
    return SqlitePool::connect(&db_path).await;
}

async fn fetch_logs(
    pool: &SqlitePool,
    tags: Vec<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<QuLog>, sqlx::Error> {
    let tags = tags.join(",");
    let logs = sqlx::query_as::<_, QuLogDBO>("SELECT * FROM qu_log WHERE tags LIKE ($1)")
        .bind(tags)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|m| m.to_log())
        .collect::<Vec<QuLog>>();

    return Ok(logs);
}

mod tests {
    use super::*;

    #[test]
    fn test_qulog_model_mapping() {
        let text = "Hello world";
        let create_date = Local::now();
        let tags = vec!["hello".to_string(), "world".to_string()];
        let model = QuLog {
            text: text.to_string(),
            tags: tags.clone(),
            create_date: create_date,
        };

        let db_model = QuLogDBO::from_log(&model);
        println!("####   {}", db_model.create_date);
        let model = db_model.to_log();

        assert_eq!(model.text, text);
        assert_eq!(model.create_date, create_date);
        assert_eq!(model.tags, tags);
    }
}

fn tags_string(tags: Vec<&str>) -> String {
    tags.join(",")
}

fn tags_vec(tags: &str) -> Vec<&str> {
    tags.split(",").filter(|s| !s.is_empty()).collect()
}

//persist with sql
//read with sql (list, single, filtered)
//delete with sql
