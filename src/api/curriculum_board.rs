use std::mem::replace;
use actix_web::{get, post, put, patch, HttpResponse, Responder, web, HttpRequest, error};
use actix_web::middleware::Condition;
use sea_orm::{FromQueryResult, ConnectionTrait, QueryTrait, ModelTrait, ActiveModelTrait, QueryFilter, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, IntoActiveModel, Statement, SelectModel, SelectorRaw};
use sea_orm::ActiveValue::Set;
use serde_json::{json, to_string, Value};
use crate::api::auth::{require_authentication, UserInfo};
use crate::api::error_handler::{ErrorMessage, internal_server_error, not_found};
use crate::entity::prelude::*;
use crate::CourseGroup::{GetMultiCourseGroup, GetSingleCourseGroup, Model, NewCourseGroup};
use crate::entity::{course, course_review, coursegroup, coursegroup_course, review};
use crate::entity::course::GetSingleCourse;
use crate::entity::review::{GetMyReview, GetReview, HistoryReview, NewReview};
use chrono::Local;
use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use reqwest::StatusCode;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body(format!("Welcome to curriculum_board backend.\nBrowse API documents on GitHub at https://github.com/OpenTreeHole/curriculum_board_backend_next please.\n\n\
    Current version: {}\n\
    Build time: {}\n\
    Rust compiler version: {}", env!("VERGEN_GIT_SHA"), env!("VERGEN_BUILD_TIMESTAMP"), env!("VERGEN_RUSTC_SEMVER")))
}
lazy_static! {
    static ref COURSE_GROUP_CACHE: RwLock<Option<String>> = RwLock::new(None);
    static ref COURSE_GROUP_HASH_CACHE: RwLock<Option<String>> = RwLock::new(None);
}
async fn build_course_group_cache(db: &DatabaseConnection) -> Result<RwLockReadGuard<Option<String>>, DbErr> {
    let result: Vec<(coursegroup::Model, Vec<course::Model>)> =
        Coursegroup::find().find_with_related(Course).all(db).await?;
    let mut group_list: Vec<GetMultiCourseGroup> = vec![];
    for x in result {
        group_list.push(GetMultiCourseGroup::new(x.0, x.1));
    }

    let mut cache_writer = COURSE_GROUP_CACHE.write().unwrap();
    *cache_writer = Some(to_string(&group_list).unwrap());
    drop(cache_writer);

    let mut cache_writer = COURSE_GROUP_HASH_CACHE.write().unwrap();
    let cache_reader = COURSE_GROUP_CACHE.read().unwrap();
    *cache_writer = Some(sha1::Sha1::from(cache_reader.as_ref().unwrap()).hexdigest());
    drop(cache_writer);
    drop(cache_reader);

    Ok(COURSE_GROUP_CACHE.read().unwrap())
}

async fn get_course_group_cache(db: &DatabaseConnection) -> Result<RwLockReadGuard<Option<String>>, DbErr> {
    let cache = COURSE_GROUP_CACHE.read().unwrap();
    if cache.is_none() {
        drop(cache);
        Ok(build_course_group_cache(db).await?)
    } else {
        Ok(cache)
    }
}

async fn get_course_group_hash_cache(db: &DatabaseConnection) -> Result<RwLockReadGuard<Option<String>>, DbErr> {
    let cache = COURSE_GROUP_HASH_CACHE.read().unwrap();
    if cache.is_none() {
        drop(cache);
        let _ = build_course_group_cache(db).await?;
        Ok(COURSE_GROUP_HASH_CACHE.read().unwrap())
    } else {
        Ok(cache)
    }
}

#[get("/courses/refresh")]
pub async fn refresh_course_groups_cache(_: HttpRequest) -> impl Responder {
    let mut cache_writer = COURSE_GROUP_CACHE.write().unwrap();
    *cache_writer = None;
    let mut cache_writer = COURSE_GROUP_HASH_CACHE.write().unwrap();
    *cache_writer = None;
    HttpResponse::build(StatusCode::from_u16(418).unwrap()).body("I'm a brand new teapot!")
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HashMessage {
    pub hash: String,
}

#[get("/courses/hash")]
pub async fn get_course_groups_hash(_: HttpRequest, db: web::Data<DatabaseConnection>) -> actix_web::Result<HttpResponse> {
    match get_course_group_hash_cache(db.get_ref()).await {
        Ok(groups) => {
            let hash_str = groups.clone().unwrap();
            Ok(HttpResponse::Ok().json(HashMessage { hash: hash_str }))
        }
        Err(e) => Err(internal_server_error(e.to_string()))
    }
}

#[get("/courses")]
pub async fn get_course_groups(_: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    match get_course_group_cache(db.get_ref()).await {
        Ok(groups) => {
            HttpResponse::Ok().content_type("application/json").body(groups.clone().unwrap())
        }
        Err(e) => internal_server_error(e.to_string())
    }
}

#[get("/group/{group_id}")]
pub async fn get_course_group(req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    let user_info = user_info.unwrap();
    let group_id = req.match_info().query("group_id").parse::<i32>();
    match group_id {
        Ok(group_id) => {
            let result: Result<Vec<(coursegroup::Model, Vec<course::Model>)>, DbErr> = Coursegroup::find_by_id(group_id).find_with_related(Course).all(db.get_ref()).await;
            match result {
                Ok(group) => {
                    if group.is_empty() {
                        return not_found(format!("Course group with id {} is not found.", group_id));
                    }
                    // 载入课程的评论列表
                    let group_and_courses = &group[0];
                    let mut course_list: Vec<GetSingleCourse> = vec![];
                    for x in &group_and_courses.1 {
                        match GetSingleCourse::load(x.clone(), db.get_ref(), user_info.id).await {
                            Ok(loaded_course) => {
                                course_list.push(loaded_course);
                            }
                            Err(e) => {
                                return internal_server_error(format!("Unable to load course group with id {}. Error: {}", group_id, e.to_string()));
                            }
                        }
                    }

                    HttpResponse::Ok().json(GetSingleCourseGroup::new(group_and_courses.0.clone(), course_list))
                }
                Err(e) => internal_server_error(e.to_string())
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ErrorMessage { message: "Invalid id syntax.".to_string() })
    }
}

#[post("/courses")]
pub async fn add_course(new_course: web::Json<course::NewCourse>, req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    if !user_info.unwrap().is_admin {
        return HttpResponse::Unauthorized().json(ErrorMessage { message: "Only admin can add new course".to_string() });
    }
    let group: Result<Option<coursegroup::Model>, DbErr> =
        Coursegroup::find().filter(coursegroup::Column::Code.eq(new_course.code.clone())).one(db.get_ref()).await;
    let new_course = new_course.into_inner();
    match group {
        Err(e) => internal_server_error(e.to_string()),
        Ok(group) => {
            let group_id = match group {
                None => {
                    // 创建新的 CourseGroup
                    let new_course_group: NewCourseGroup = new_course.clone().into();
                    let new_course_group: Result<coursegroup::Model, DbErr> =
                        new_course_group.into_active_model().insert(db.get_ref()).await;
                    if let Err(e) = new_course_group {
                        return internal_server_error(format!("Unable to create new course group. Error: {}", e.to_string()));
                    }
                    new_course_group.unwrap().id
                }
                Some(group) => { group.id }
            };
            // 创建新的 Course
            let new_course: Result<course::Model, DbErr> =
                new_course.into_active_model().insert(db.get_ref()).await;
            if let Err(e) = new_course {
                return internal_server_error(format!("Unable to create new course. Error: {}", e.to_string()));
            }
            let new_course = new_course.unwrap();

            // 连接两者
            if let Err(e) = coursegroup_course::link(group_id, new_course.id, db.get_ref()).await {
                return internal_server_error(format!("Unable to link between course and course group. Error: {}", e.to_string()));
            }

            HttpResponse::Ok().json(GetSingleCourse::from(new_course))
        }
    }
}

#[get("/courses/{course_id}")]
pub async fn get_course(req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    let user_info = user_info.unwrap();
    let course_id = req.match_info().query("course_id").parse::<i32>();
    match course_id {
        Ok(course_id) => {
            let result: Result<Vec<course::Model>, DbErr> = Course::find_by_id(course_id).all(db.get_ref()).await;
            match result {
                Ok(course) => {
                    if course.is_empty() {
                        return not_found(format!("Course with id {} is not found.", course_id));
                    }
                    // 载入课程的评论列表
                    match GetSingleCourse::load(course[0].clone(), db.get_ref(), user_info.id).await {
                        Ok(loaded_course) => {
                            HttpResponse::Ok().json(loaded_course)
                        }
                        Err(e) => {
                            internal_server_error(format!("Unable to load course with id {}. Error: {}", course_id, e.to_string()))
                        }
                    }
                }
                Err(e) => internal_server_error(e.to_string())
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ErrorMessage { message: "Invalid id syntax.".to_string() })
    }
}

#[post("/courses/{course_id}/reviews")]
pub async fn add_review(new_review: web::Json<NewReview>, req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    let user_info = user_info.unwrap();
    let new_review = new_review.into_inner();
    let course_id = req.match_info().query("course_id").parse::<i32>();
    match course_id {
        Ok(course_id) => {
            // 检查对应课程是否存在
            let course: Result<Option<course::Model>, DbErr> =
                Course::find_by_id(course_id).one(db.get_ref()).await;
            if let Err(err) = course {
                return internal_server_error(format!("Unable to fetch the course. Error: {}", err.to_string()));
            }
            let course = course.unwrap();
            if let None = course {
                return not_found(format!("Course with id {} is not found.", course_id));
            }
            let course = course.unwrap();
            // 防止同一用户创建两条评论
            let course_with_reviews = GetSingleCourse::load(course, db.get_ref(), user_info.id).await;
            if let Ok(course_with_reviews) = course_with_reviews {
                if course_with_reviews.review_list.iter().any(|r| r.is_me) {
                    return HttpResponse::Conflict().json(ErrorMessage { message: "You cannot post more than one review.".to_string() });
                }
            } else {
                return internal_server_error(format!("Unable to fetch the review list of Course with id {}. Error: {}", course_id, course_with_reviews.unwrap_err().to_string()));
            }
            // 创建新评论
            let review_added: Result<review::Model, DbErr> = new_review.into_active_model(user_info.id).insert(db.get_ref()).await;
            if let Err(err) = review_added {
                return internal_server_error(format!("Unable to create new review. Error: {}", err.to_string()));
            }
            let review_added = review_added.unwrap();
            // 连接两者
            if let Err(e) = course_review::link(course_id, review_added.id, db.get_ref()).await {
                return internal_server_error(format!("Unable to link between review and course. Error: {}", e.to_string()));
            }

            HttpResponse::Ok().json(GetReview::new(review_added, user_info.id))
        }
        Err(_) => HttpResponse::BadRequest().json(ErrorMessage { message: "Invalid id syntax.".to_string() })
    }
}

#[put("/reviews/{review_id}")]
pub async fn modify_review(new_review: web::Json<NewReview>, req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    let user_info = user_info.unwrap();
    let new_review = new_review.into_inner();
    let review_id = req.match_info().query("review_id").parse::<i32>();
    match review_id {
        Ok(review_id) => {
            let result: Result<Vec<review::Model>, DbErr> = Review::find_by_id(review_id).all(db.get_ref()).await;
            match result {
                Ok(review) => {
                    if review.is_empty() {
                        return not_found(format!("Review with id {} is not found.", review_id));
                    }
                    let review = &review[0];
                    if review.reviewer_id != user_info.id && !user_info.is_admin {
                        return HttpResponse::Unauthorized().json(ErrorMessage { message: "You have no permission to modify this review!".to_string() });
                    }

                    // 储存目前的 Review
                    let snapshot = serde_json::to_value(&review.clone().into() as &HistoryReview).unwrap();

                    let mut history = (*review.history.as_array().unwrap()).clone();
                    history.push(json!({
                        "alter_by":user_info.id,
                        "time": Local::now().naive_utc(),
                        "original": snapshot
                    }));
                    //更新字段
                    let mut updated_review: review::ActiveModel = review.clone().into();
                    updated_review.history = Set(Value::Array(history));
                    updated_review.update_with(new_review);
                    let updated_review: Result<review::Model, DbErr> = updated_review.update(db.get_ref()).await;

                    match updated_review {
                        Ok(updated_review) => HttpResponse::Ok().json(GetReview::new(updated_review, user_info.id)),
                        Err(err) => internal_server_error(format!("Unable to update the review. Error: {}", err.to_string()))
                    }
                }
                Err(e) => internal_server_error(e.to_string())
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ErrorMessage { message: "Invalid id syntax.".to_string() })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewVote {
    pub upvote: bool,
}

#[patch("/reviews/{review_id}")]
pub async fn vote_for_review(vote_data: web::Json<NewVote>, req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    let user_info = user_info.unwrap();
    let review_id = req.match_info().query("review_id").parse::<i32>();
    let vote_data = vote_data.into_inner();
    match review_id {
        Ok(review_id) => {
            let result: Result<Vec<review::Model>, DbErr> = Review::find_by_id(review_id).all(db.get_ref()).await;
            match result {
                Ok(review) => {
                    if review.is_empty() {
                        return not_found(format!("Review with id {} is not found.", review_id));
                    }
                    let review = &review[0];

                    // 复制并设置 *voters 列表
                    let mut upvoters = (*review.upvoters.as_array().unwrap()).clone();
                    let mut downvoters = (*review.downvoters.as_array().unwrap()).clone();
                    let up_pos = upvoters.iter().position(|upvoter_id| upvoter_id.as_i64().unwrap_or(-1) as i32 == user_info.id);
                    let down_pos = downvoters.iter().position(|downvoter_id| downvoter_id.as_i64().unwrap_or(-1) as i32 == user_info.id);
                    if vote_data.upvote {
                        match up_pos {
                            None => {
                                upvoters.push(user_info.id.into());
                                if let Some(down_pos) = down_pos {
                                    downvoters.swap_remove(down_pos);
                                }
                            }
                            Some(position) => {
                                upvoters.swap_remove(position);
                            }
                        }
                    } else {
                        match down_pos {
                            None => {
                                downvoters.push(user_info.id.into());
                                if let Some(up_pos) = up_pos {
                                    upvoters.swap_remove(up_pos);
                                }
                            }
                            Some(position) => {
                                downvoters.swap_remove(position);
                            }
                        }
                    }
                    // 更新字段
                    let mut updated_review: review::ActiveModel = review.clone().into();
                    updated_review.upvoters = Set(Value::Array(upvoters));
                    updated_review.downvoters = Set(Value::Array(downvoters));
                    let updated_review: Result<review::Model, DbErr> = updated_review.update(db.get_ref()).await;

                    match updated_review {
                        Ok(updated_review) => HttpResponse::Ok().json(GetReview::new(updated_review, user_info.id)),
                        Err(err) => internal_server_error(format!("Unable to update the review. Error: {}", err.to_string()))
                    }
                }
                Err(e) => internal_server_error(e.to_string())
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ErrorMessage { message: "Invalid id syntax.".to_string() })
    }
}


#[get("/reviews/me")]
pub async fn get_reviews(req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    let user_info = user_info.unwrap();
    let result: Result<Vec<(review::Model, Vec<course::Model>)>, DbErr> =
        Review::find().filter(review::Column::ReviewerId.eq(user_info.id)).find_with_related(Course).all(db.get_ref()).await;
    match result {
        Ok(results) => {
            let mut review_list: Vec<GetMyReview> = vec![];
            for x in results {
                let review = &x.0;
                let course = &x.1[0];
                review_list.push(GetMyReview::new(review.clone(), course.clone(), user_info.id))
            }

            HttpResponse::Ok().json(review_list)
        }
        Err(e) => internal_server_error(e.to_string())
    }
}

#[derive(Debug, FromQueryResult)]
pub struct Counts {
    cnt: i32,
}

#[get("/reviews/random")]
pub async fn get_random_reviews(req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let user_info = require_authentication(&req).await;
    if let Err(e) = user_info {
        return e;
    }
    let user_info = user_info.unwrap();
    let review_count: Result<Option<Counts>, DbErr> = Counts::find_by_statement(Statement::from_string(db.get_ref().get_database_backend(), r#"SELECT COUNT(*) AS cnt FROM review"#.to_string())).one(db.get_ref()).await;
    if let Err(err) = review_count {
        return internal_server_error(format!("Unable to count the reviews. Error: {}", err.to_string()));
    }
    let review_count = review_count.unwrap();
    if let None = review_count {
        return internal_server_error(format!("Unable to count the reviews since database returns no result."));
    }
    let review_count = review_count.unwrap().cnt;
    let mut rng = rand::thread_rng();
    // 重试 5 次
    for _ in 1..5 {
        let id = rng.gen_range(1..=review_count) as i32;
        let result: Result<Vec<(review::Model, Vec<course::Model>)>, DbErr> =
            Review::find_by_id(id).find_with_related(Course).all(db.get_ref()).await;
        if let Ok(results) = result {
            if !results.is_empty() {
                let result = results.first().unwrap().clone();
                return HttpResponse::Ok().json(GetMyReview::new(result.0, result.1.first().unwrap().clone(), user_info.id));
            }
        }
    }
    internal_server_error("Unable to fetch a random review. Retry later.".to_string())
}
