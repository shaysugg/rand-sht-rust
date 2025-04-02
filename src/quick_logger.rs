use clap::{Parser, Subcommand};
//log struct

use chrono::{DateTime, Local};
use sqlx::{sqlite::SqlitePool, FromRow};

struct QuLog {
    text: String,
    tags: QuLogTags,
    create_date: DateTime<Local>,
}

struct QuLogTags(Vec<String>);

impl From<String> for QuLogTags {
    fn from(value: String) -> Self {
        let tags = value
            .split(",")
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        QuLogTags(tags)
    }
}

impl Into<String> for QuLogTags {
    fn into(self) -> String {
        self.0.join(",")
    }
}

impl QuLogTags {
    fn empty() -> Self {
        QuLogTags(Vec::new())
    }
}

struct DBConfig {
    in_memory: bool,
}

#[derive(Debug, FromRow)]
struct QuLogDBO {
    text: String,
    #[sqlx(default)]
    tags: String,
    create_date: DateTime<Local>,
}

impl From<&QuLog> for QuLogDBO {
    fn from(log: &QuLog) -> Self {
        QuLogDBO {
            text: log.text.clone(),
            tags: log.tags.0.join(","),
            create_date: log.create_date,
        }
    }
}

impl Into<QuLog> for QuLogDBO {
    fn into(self) -> QuLog {
        QuLog {
            text: self.text.clone(),
            tags: QuLogTags::from(self.tags),
            create_date: self.create_date,
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

    let config = DBConfig { in_memory: false };
    let pool = match connect_to_db(&config).await {
        Ok(pool) => pool,
        Err(err) => panic!("{:?}", err),
    };

    create_log_table_if_not_exists(&pool)
        .await
        .expect("Unable to create log data base");

    match args.command {
        LogCreateCommand::Create { text, tags } => {
            let tags: QuLogTags = match tags {
                Some(tags) => QuLogTags::from(tags),

                None => QuLogTags::empty(),
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
            let tags = match tags {
                Some(tags) => QuLogTags::from(tags),
                None => QuLogTags::empty(),
            };
            let logs = fetch_logs(&pool, Some(tags), start_date, end_date)
                .await
                .expect("Unable to fetch logs");
            for log in logs {
                println!(
                    "-> {} : {} [{}]",
                    log.create_date.format("%d-%m-%Y %H:%M:%S"),
                    log.text,
                    log.tags.0.join("-")
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
            create_date DATETIME NOT NULL
        )
    "#,
    )
    .execute(pool)
    .await?;
    return Ok(());
}

async fn create_log(model: &QuLog, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let db_model = QuLogDBO::from(model);
    sqlx::query("INSERT INTO qu_log (text, create_date, tags) VALUES ($1, $2, $3)")
        .bind(db_model.text)
        .bind(db_model.create_date)
        .bind(db_model.tags)
        .execute(pool)
        .await?;
    Ok(())
}

async fn connect_to_db(config: &DBConfig) -> Result<SqlitePool, sqlx::Error> {
    if config.in_memory {
        return SqlitePool::connect("sqlite::memory:").await;
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
    tags: Option<QuLogTags>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<QuLog>, sqlx::Error> {
    let tags = match tags {
        Some(tags) if !tags.0.is_empty() => tags.into(),
        _ => "%".to_string(),
    };

    // AND (create_date >= @startDate OR @startDate IS NULL)
    // AND (create_date <= @endDate OR @endDate IS NULL)

    let logs = sqlx::query_as::<_, QuLogDBO>(
        r#"
    SELECT * FROM qu_log WHERE tags LIKE ($1) 
    "#,
    )
    .bind(tags)
    // .bind(start_date)
    // .bind(end_date)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|m| m.into())
    .collect::<Vec<QuLog>>();

    return Ok(logs);
}

mod tests {

    use chrono::Timelike;

    use super::*;

    #[test]
    fn test_qulog_model_mapping() {
        let text = "Hello world";
        let create_date = Local::now().with_nanosecond(0).unwrap();
        let tags = vec!["hello".to_string(), "world".to_string()];
        let model = QuLog {
            text: text.to_string(),
            tags: QuLogTags(tags.clone()),
            create_date: create_date,
        };

        let db_model = QuLogDBO::from(&model);
        println!("####   {}", db_model.create_date);
        let model: QuLog = db_model.into();

        assert_eq!(model.text, text);
        assert_eq!(model.create_date, create_date);
        assert_eq!(model.tags.0, tags);
    }

    #[tokio::test]
    async fn test_qulog_connect_db_in_memory() {
        let _ = in_memory_pool().await.unwrap();
    }

    #[tokio::test]
    async fn test_qulog_inserting() {
        let pool = in_memory_pool().await.unwrap();

        let text = "Hello world";
        let create_date = Local::now().with_nanosecond(0).unwrap();
        let tags = vec!["hello".to_string(), "world".to_string()];

        insert(&pool, text, &create_date, tags)
            .await
            .expect("Unable to insert");
    }

    #[tokio::test]
    async fn test_qulog_insert_and_read() {
        let pool = in_memory_pool().await.unwrap();

        let text = "Hello world";
        let create_date = Local::now().with_nanosecond(0).unwrap();
        let tags = vec!["hello".to_string(), "world".to_string()];
        insert(&pool, text, &create_date, tags).await.unwrap();

        let all: Vec<QuLog> = fetch_all(&pool).await.unwrap().into_iter().collect();
        assert_eq!(all.len(), 1);
        assert_eq!(all.first().unwrap().text, text);
    }

    async fn in_memory_pool() -> Result<SqlitePool, sqlx::Error> {
        let cnfg = DBConfig { in_memory: true };
        connect_to_db(&cnfg).await
    }

    async fn insert(
        pool: &SqlitePool,
        text: &str,
        create_date: &DateTime<Local>,
        tags: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        create_log_table_if_not_exists(&pool)
            .await
            .expect("Unable to create log data base");

        let model = QuLog {
            text: text.to_string(),
            tags: QuLogTags(tags),
            create_date: create_date.clone(),
        };

        create_log(&model, &pool).await
    }

    async fn fetch_all(pool: &SqlitePool) -> Result<Vec<QuLog>, sqlx::Error> {
        fetch_logs(&pool, None, None, None).await
    }

    // #[test]
    // fn test_ch_dates() {
    //     let date = Local::now();
    //     let str = date.format("%Y-%m-%d %H:%M:%S").to_string();
    //     print!("%%%%%%%  {str}");
    //     let d = NaiveDateTime::parse_from_str(&str, "%Y-%m-%d %H:%M:%S").expect("Unable to parse");
    // }
}

//persist with sql
//read with sql (list, single, filtered)
//delete with sql
