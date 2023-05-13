#![feature(result_option_inspect)]
use anyhow::{Context, Result};
use clap::Parser;
use indicatif::ProgressBar;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, EntityTrait, IntoActiveModel, QueryFilter,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    sync::atomic::{AtomicBool, Ordering},
};

/// A command line tool for importing JSON files into curriculum database.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The database URL
    #[arg(short, long)]
    db_url: String,

    /// The JSON file to import
    #[arg(short, long)]
    json_file: String,

    /// Which year to import
    #[arg(short, long)]
    year: i32,

    /// Which semester to import
    #[arg(short, long)]
    semester: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawCourse {
    name: String,
    no: String,
    department: String,
    teachers: String,
    credits: f64,
}

static TERMINATE: AtomicBool = AtomicBool::new(false);
#[tokio::main]
async fn main() -> Result<()> {
    let _ = ctrlc::set_handler(move || {
        println!("Ctrl+C pressed, exiting...");
        TERMINATE.store(true, Ordering::SeqCst);
    })
    .inspect_err(|e| {
        println!("Error occurred when setting Ctrl+C handler: {}. You will be unable to stop the program during running gracefully!", e);
    });

    let args = Args::parse();

    println!("Reading JSON data from `{}`", args.json_file);
    let content = read_to_string(&args.json_file)
        .with_context(|| format!("Failed to read JSON file from `{}`", args.json_file))?;

    let raw_courses: Vec<RawCourse> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON file from `{}`", args.json_file))?;

    println!("Found {} courses", raw_courses.len());

    println!("Connecting to database at `{}`", args.db_url);
    let db = Database::connect(&args.db_url)
        .await
        .with_context(|| format!("Failed to connect to database at `{}`", args.db_url))?;

    println!("Importing courses into database");
    // start transaction
    let t = db
        .begin()
        .await
        .with_context(|| "Failed to start transaction")?;

    let pb = ProgressBar::new(raw_courses.len() as u64);
    for raw_course in raw_courses {
        if TERMINATE.load(Ordering::SeqCst) {
            // Ask user if he wants to commit or rollback
            let mut input = String::new();
            println!("Commit or rollback the changes? (c/r)");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            match input.trim() {
                "c" => {
                    t.commit()
                        .await
                        .with_context(|| "Failed to commit transaction")?;
                    println!("Changes committed");
                }
                "r" => {
                    t.rollback()
                        .await
                        .with_context(|| "Failed to rollback transaction")?;
                    println!("Changes rolled back");
                }
                _ => {
                    println!("Invalid input, changes rolled back");
                    t.rollback()
                        .await
                        .with_context(|| "Failed to rollback transaction")?;
                }
            }
            return Ok(());
        }

        let len = raw_course.no.len();
        if len <= 3 {
            println!("The no of course `{:?}` is too short", raw_course);
            continue;
        }
        let code = &raw_course.no[0..len - 3];

        let new_course = entity::course::NewCourse {
            name: raw_course.name,
            code: code.to_string(),
            department: raw_course.department,
            teachers: raw_course.teachers,
            credit: raw_course.credits,
            code_id: raw_course.no.clone(),
            campus_name: Default::default(),
            max_student: Default::default(),
            week_hour: Default::default(),
            year: args.year,
            semester: args.semester,
        };

        let group = entity::coursegroup::Entity::find()
            .filter(entity::coursegroup::Column::Code.eq(code))
            .one(&t)
            .await
            .with_context(|| format!("Error occurred when finding course group `{}`", code))?;
        let group_id = match group {
            None => {
                // 创建新的 CourseGroup
                let new_course_group: entity::coursegroup::NewCourseGroup =
                    new_course.clone().into();
                let new_course_group = new_course_group
                    .into_active_model()
                    .insert(&t)
                    .await
                    .with_context(|| {
                        format!("Error occurred when inserting course group `{}`", code)
                    })?;
                new_course_group.id
            }
            Some(group) => group.id,
        };

        let active_course = new_course.into_active_model(group_id);
        active_course
            .insert(&t)
            .await
            .with_context(|| format!("Error occurred when inserting course `{}`", raw_course.no))?;
        pb.inc(1);
    }
    pb.finish_with_message("Committing changes");
    t.commit()
        .await
        .with_context(|| "Failed to commit transaction")?;
    println!("Congratulations! All courses have been imported successfully!");
    Ok(())
}
