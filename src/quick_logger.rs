use std::fmt::Debug;

use clap::{Parser, Subcommand, ValueEnum};
//log struct

use chrono::{DateTime, Datelike, Days, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use sqlx::{sqlite::SqlitePool, FromRow, Sqlite};

// const SQL_DATE_FORMAT_

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

#[derive(Debug, Parser)]
#[command(name = "qulog")]
struct LogCreateCli {
    #[command(subcommand)]
    command: QuLogCommand,
}

#[derive(Debug, Subcommand)]
enum QuLogCommand {
    Log {
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
        #[arg(value_enum)]
        date_range: Option<QuLogCommandDateRange>,
    },

    Export {
        #[arg(long)]
        tags: Option<String>,
        #[arg(long, short)]
        start_date: Option<String>,
        #[arg(long, short)]
        end_date: Option<String>,
        #[arg(value_enum)]
        date_range: Option<QuLogCommandDateRange>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, ValueEnum)]
enum QuLogCommandDateRange {
    Today,
    ThisWeek,
    ThisMonth,
    ThisYear,
}

impl QuLogCommandDateRange {
    fn date_times(&self) -> (DateTime<Local>, DateTime<Local>) {
        match self {
            QuLogCommandDateRange::Today => (
                Local::now().with_time(NaiveTime::MIN).unwrap(),
                Local::now()
                    .checked_add_days(Days::new(1))
                    .unwrap()
                    .with_time(NaiveTime::MIN)
                    .unwrap(),
            ),
            QuLogCommandDateRange::ThisWeek => {
                let days_since = Local::now().weekday().days_since(chrono::Weekday::Mon);
                let start_of_week = Local::now()
                    .checked_sub_days(chrono::Days::new((days_since - 1) as u64))
                    .unwrap()
                    .with_time(NaiveTime::MIN)
                    .unwrap();

                (start_of_week, Local::now())
            }
            QuLogCommandDateRange::ThisMonth => {
                let days_since = Local::now().day();
                let start_of_month = Local::now()
                    .checked_sub_days(chrono::Days::new((days_since - 1) as u64))
                    .unwrap()
                    .with_time(NaiveTime::MIN)
                    .unwrap();
                (start_of_month, Local::now())
            }
            QuLogCommandDateRange::ThisYear => {
                let current_year = Local::now().year();
                let start_of_year = NaiveDate::from_ymd_opt(current_year - 1, 1, 1)
                    .unwrap()
                    .and_time(NaiveTime::MIN)
                    .and_local_timezone(Local)
                    .unwrap();

                (start_of_year, Local::now())
            }
        }
    }
}

struct QuLogCommandParser {
    start_date: Option<DateTime<Local>>,
    end_date: Option<DateTime<Local>>,
    tags: QuLogTags,
}

impl QuLogCommandParser {
    fn parse(
        tags: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
        date_range: Option<QuLogCommandDateRange>,
    ) -> Self {
        fn parse_local_date_from(value: Option<String>) -> Option<DateTime<Local>> {
            value
                .and_then(|s| NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%d %H:%M:%S").ok())
                .map(|f| f.and_local_timezone(Local).unwrap())
        }

        if date_range.is_some() && (start_date.is_some() || end_date.is_some()) {
            panic!("Date filtering can be done by date_range or start_date and end_date.")
        }

        let range: (Option<DateTime<Local>>, Option<DateTime<Local>>) = match date_range {
            Some(date_range) => {
                let range = date_range.date_times();
                (Some(range.0), Some(range.1))
            }
            None => (
                parse_local_date_from(start_date),
                parse_local_date_from(end_date),
            ),
        };

        let tags = match tags {
            Some(tags) => QuLogTags::from(tags),
            None => QuLogTags::empty(),
        };

        QuLogCommandParser {
            start_date: range.0,
            end_date: range.1,
            tags,
        }
    }
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
        QuLogCommand::Log { text, tags } => {
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

        QuLogCommand::Show {
            tags,
            start_date,
            end_date,
            date_range,
        } => {
            let parameters = QuLogCommandParser::parse(tags, start_date, end_date, date_range);

            let logs = fetch_logs(
                &pool,
                Some(parameters.tags),
                parameters.start_date,
                parameters.end_date,
            )
            .await
            .expect("Unable to fetch logs");

            if logs.is_empty() {
                println!("No record is found");
                return;
            }

            for log in logs {
                println!(
                    "-> {} : {} [{}]",
                    log.create_date.format("%Y-%m-%d %H:%M:%S"),
                    log.text,
                    log.tags.0.join("-")
                );
            }
        }

        QuLogCommand::Export {
            tags,
            start_date,
            end_date,
            date_range,
        } => {
            let parameters = QuLogCommandParser::parse(tags, start_date, end_date, date_range);

            let logs = fetch_logs(
                &pool,
                Some(parameters.tags),
                parameters.start_date,
                parameters.end_date,
            )
            .await
            .expect("Unable to fetch logs");

            if logs.is_empty() {
                println!("No record is found");
                return;
            }

            let mut base = String::from("<table><tr><th>Date</th><th>Log</th><th>Tags</th></tr>");

            for log in logs {
                let row = format!(
                    "<tr><td>{date}</td><td>{log}</td><td>{tags}</td>",
                    date = log.create_date.format("%Y-%m-%d %H:%M:%S"),
                    log = log.text,
                    tags = log.tags.0.join("-")
                );
                base.push_str(row.as_str());
            }

            base.push_str("</table>");

            std::fs::write("export.html", base);
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
    start_date: Option<DateTime<Local>>,
    end_date: Option<DateTime<Local>>,
) -> Result<Vec<QuLog>, sqlx::Error> {
    let tags = match tags {
        Some(tags) if !tags.0.is_empty() => tags.into(),
        _ => "%".to_string(),
    };

    let far_future = Local.with_ymd_and_hms(3000, 01, 01, 00, 00, 00).unwrap();
    let far_past = Local.timestamp_micros(0).unwrap();

    let start_date = start_date
        .unwrap_or(far_past)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    let end_date = end_date
        .unwrap_or(far_future)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    let logs = sqlx::query_as::<Sqlite, QuLogDBO>(
        r#"
    SELECT * FROM qu_log WHERE tags LIKE ($1) 
    AND create_date >= ($2) AND create_date <= ($3)
    "#,
    )
    .bind(tags)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|m| m.into())
    .collect::<Vec<QuLog>>();

    return Ok(logs);
}

mod tests {

    use chrono::{Days, Months, Timelike};

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

        let all: Vec<QuLog> = fetch_logs(&pool, None, None, None)
            .await
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(all.len(), 1);
        assert_eq!(all.first().unwrap().text, text);
    }

    #[tokio::test]
    async fn test_qulog_read_filter_date() {
        let pool = in_memory_pool().await.unwrap();

        async fn insert_sample_with_date(date: &DateTime<Local>, pool: &SqlitePool) {
            let text = "Hello world";
            let tags = Vec::new();
            insert(pool, text, &date, tags).await.unwrap();
        }

        let base = Local::now();
        insert_sample_with_date(&base.checked_add_days(Days::new(1)).unwrap(), &pool).await;
        insert_sample_with_date(&base.checked_add_days(Days::new(2)).unwrap(), &pool).await;
        insert_sample_with_date(&base.checked_add_days(Days::new(3)).unwrap(), &pool).await;

        insert_sample_with_date(&base.checked_add_months(Months::new(2)).unwrap(), &pool).await;
        insert_sample_with_date(&base.checked_add_months(Months::new(3)).unwrap(), &pool).await;

        insert_sample_with_date(&base.checked_add_months(Months::new(24)).unwrap(), &pool).await;

        let all: Vec<QuLog> = fetch_logs(&pool, None, None, None)
            .await
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(all.len(), 6);

        let this_month: Vec<QuLog> = fetch_logs(
            &pool,
            None,
            Some(base),
            Some(base.checked_add_months(Months::new(1)).unwrap()),
        )
        .await
        .unwrap()
        .into_iter()
        .collect();
        assert_eq!(this_month.len(), 3);
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
}

//persist with sql
//read with sql (list, single, filtered)
//delete with sql
