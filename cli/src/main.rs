use anyhow::{Context, Result};
use clap::Parser;
use either::Either;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string, io::Write, sync::atomic::{AtomicBool, Ordering}
};

/// A command line tool for importing JSON files into curriculum database.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The RESTful API URL of the server, NOT the database URL itself.
    /// 
    /// E.g. `http://localhost:8080/courses`
    #[arg(short, long)]
    db_url: String,

    /// The auth token for the RESTful API, without `Bearer ` prefix.
    #[arg(short, long)]
    auth_token: String,

    /// The JSON file to import
    #[arg(short, long)]
    json_file: String,

    /// Which year to import
    /// 
    /// E.g. `2021` means 2021-2022 academic year.
    #[arg(short, long)]
    year: i32,

    /// Which semester to import
    /// 
    /// E.g. `1` means the autumn semester, `2` means the (next year's) winter holiday, `3` means the (next year's) spring semester, `4` means the (next year's) summer holiday.
    #[arg(short, long)]
    semester: i32,

    /// Proxy server URL, if needed
    #[arg(short, long)]
    proxy: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawCourse {
    name: String,
    no: String,
    teachDepartName: String,
    teachers: String,
    credits: f64,
    maxStudent: i32,
    campusName: String,
    weekHour: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawJwfwCourse {
    name: String,
    no: String,
    teachers: String,
    credits: f64,
    department: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NewCourse {
    campus_name: String,
    code: String,
    code_id: String,
    credit: f64,
    department: String,
    max_student: i32,
    name: String,
    semester: i32,
    teachers: String,
    week_hour: i32,
    year: i32,
}

impl RawCourse {
    fn into_new_course(self, year: i32, semester: i32) -> NewCourse {
        let code = &self.no[0..self.no.len() - 3];
        NewCourse {
            name: self.name,
            code: code.to_string(),
            department: self.teachDepartName,
            teachers: self.teachers,
            credit: self.credits,
            code_id: self.no,
            campus_name: self.campusName,
            max_student: self.maxStudent,
            week_hour: self.weekHour,
            year,
            semester,
        }
    }
}

impl RawJwfwCourse {
    fn into_new_course(self, year: i32, semester: i32) -> NewCourse {
        let code = &self.no[0..self.no.len() - 3];
        NewCourse {
            name: self.name,
            code: code.to_string(),
            department: self.department,
            teachers: self.teachers,
            credit: self.credits,
            code_id: self.no,
            campus_name: Default::default(),
            max_student: Default::default(),
            week_hour: Default::default(),
            year,
            semester,
        }
    }
}

fn parse(raw_json: &String) -> Result<Either<Vec<RawCourse>, Vec<RawJwfwCourse>>> {
    let raw_courses = serde_json::from_str::<Vec<RawCourse>>(&raw_json);
    let raw_jwfw_courses = serde_json::from_str::<Vec<RawJwfwCourse>>(&raw_json);
    match (raw_courses, raw_jwfw_courses) {
        (Ok(raw_courses), _) => Ok(Either::Left(raw_courses)),
        (_, Ok(raw_jwfw_courses)) => Ok(Either::Right(raw_jwfw_courses)),
        (Err(e), Err(_)) => Err(e.into()),
    }
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

    let raw_courses = parse(&content)
        .with_context(|| format!("Failed to parse JSON file from `{}`", args.json_file))?;

    let course_num = match &raw_courses {
        Either::Left(raw_courses) => raw_courses.len(),
        Either::Right(raw_jwfw_courses) => raw_jwfw_courses.len(),
    };
    println!("Found {} courses", course_num);

    let pb = ProgressBar::new(course_num as u64);

    let course_iter = raw_courses
        .map_either(
            |a| a.into_iter().map(|v| Either::Left(v)),
            |b| b.into_iter().map(|v| Either::Right(v)),
        )
        .into_iter();

    let client = reqwest::Client::builder();
    let client = if let Some(proxy) = args.proxy {
        client
            .proxy(reqwest::Proxy::all(proxy).expect("Failed to set proxy"))
            .build()?
    } else {
        client.build()?
    };

    for raw_course in course_iter {
        if TERMINATE.load(Ordering::SeqCst) {
            let mut input = String::new();
            print!("Do you want to stop the program? (y/N) ");
            let _ = std::io::stdout().flush();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            if input.trim().to_lowercase() == "y" {
                return Ok(());
            } else {
                TERMINATE.store(false, Ordering::SeqCst);
            }
        }

        let no = match &raw_course {
            Either::Left(raw_course) => raw_course.no.to_owned(),
            Either::Right(raw_course) => raw_course.no.to_owned(),
        };

        let len = no.len();
        if len <= 3 {
            println!("The no of course `{:?}` is too short", raw_course);
            continue;
        }

        let new_course = match raw_course {
            Either::Left(raw_course) => raw_course.into_new_course(args.year, args.semester),
            Either::Right(raw_course) => raw_course.into_new_course(args.year, args.semester),
        }; 

        let resq = client
            .post(&args.db_url)
            .bearer_auth(&args.auth_token)
            .json(&new_course)
            .send()
            .await?;
        
        pb.inc(1);
        
        if !resq.status().is_success() {
            println!(
                "Failed to import course `{:?}`: {}",
                new_course,
                resq.text().await?
            );
        }
        
    }
    pb.finish();
    println!("Congratulations! All courses have been imported successfully!");
    Ok(())
}
