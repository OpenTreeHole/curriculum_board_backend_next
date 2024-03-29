use crate::api::auth::require_authentication;
use crate::api::error_handler::{
    bad_request, conflict, internal_server_error, not_found, unauthorized, ErrorMessage,
};
use actix_web::{get, patch, post, put, web, HttpRequest, HttpResponse, Responder};
use chrono::Local;
use entity::course::{GetSingleCourse, NewCourse};
use entity::coursegroup::{GetMultiCourseGroup, GetSingleCourseGroup, NewCourseGroup};
use entity::prelude::*;
use entity::review::{GetMyReview, GetReview, HistoryReview, NewReview};
use entity::{course, coursegroup, review};
use lazy_static::lazy_static;
use rand::Rng;
use reqwest::StatusCode;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    FromQueryResult, IntoActiveModel, QueryFilter, Statement,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Value};
use sha3::{Digest, Sha3_256};
use std::sync::{RwLock, RwLockReadGuard};
use utoipa::ToSchema;

#[utoipa::path(
responses(
(status = 200, description = "Return debug information"),
)
)]
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
async fn build_course_group_cache(
    db: &DatabaseConnection,
) -> Result<RwLockReadGuard<Option<String>>, DbErr> {
    let result: Vec<(coursegroup::Model, Vec<course::Model>)> = Coursegroup::find()
        .find_with_related(Course)
        .all(db)
        .await?;
    let mut group_list: Vec<GetMultiCourseGroup> = vec![];
    for x in result {
        group_list.push(GetMultiCourseGroup::new(x.0, x.1));
    }

    let mut cache_writer = COURSE_GROUP_CACHE.write().unwrap();
    *cache_writer = Some(to_string(&group_list).map_err(|e| DbErr::Custom(e.to_string()))?);
    drop(cache_writer);

    let mut cache_writer = COURSE_GROUP_HASH_CACHE.write().unwrap();
    let cache_reader = COURSE_GROUP_CACHE.read().unwrap();
    let hash = Sha3_256::digest(cache_reader.as_ref().unwrap().as_bytes());
    let hex_hash = base16ct::lower::encode_string(&hash);
    *cache_writer = Some(hex_hash);
    drop(cache_writer);
    drop(cache_reader);

    Ok(COURSE_GROUP_CACHE.read().unwrap())
}

async fn get_course_group_cache(
    db: &DatabaseConnection,
) -> Result<RwLockReadGuard<Option<String>>, DbErr> {
    let cache = COURSE_GROUP_CACHE.read().unwrap();
    if cache.is_none() {
        drop(cache);
        Ok(build_course_group_cache(db).await?)
    } else {
        Ok(cache)
    }
}

async fn get_course_group_hash_cache(
    db: &DatabaseConnection,
) -> Result<RwLockReadGuard<Option<String>>, DbErr> {
    let cache = COURSE_GROUP_HASH_CACHE.read().unwrap();
    if cache.is_none() {
        drop(cache);
        drop(build_course_group_cache(db).await?);
        Ok(COURSE_GROUP_HASH_CACHE.read().unwrap())
    } else {
        Ok(cache)
    }
}

#[utoipa::path(
responses(
(status = 418, description = "Refresh cache successfully"),
)
)]
#[get("/courses/refresh")]
pub async fn refresh_course_groups_cache(_unused: HttpRequest) -> impl Responder {
    let mut cache_writer = COURSE_GROUP_CACHE.write().unwrap();
    *cache_writer = None;
    let mut cache_writer = COURSE_GROUP_HASH_CACHE.write().unwrap();
    *cache_writer = None;
    HttpResponse::build(StatusCode::IM_A_TEAPOT).body("I'm a brand new empty teapot now!")
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HashMessage {
    pub hash: String,
}

#[utoipa::path(
responses(
(status = 200, description = "Hash of course group cache", body = HashMessage),
)
)]
#[get("/courses/hash")]
pub async fn get_course_groups_hash(
    _unused: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let groups = get_course_group_hash_cache(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;
    let hash_str = groups.clone().ok_or(internal_server_error(
        "Missing cache. The server did build the cache but the cache seems to be none.".to_string(),
    ))?;
    Ok(HttpResponse::Ok().json(HashMessage { hash: hash_str }))
}

#[utoipa::path(
responses(
(status = 200, description = "Course group. Reviews are not included.", body = [GetMultiCourseGroup]),
)
)]
#[get("/courses")]
pub async fn get_course_groups(
    _unused: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let groups = get_course_group_cache(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;
    Ok(HttpResponse::Ok().content_type("application/json").body(
        groups.clone().ok_or(internal_server_error(
            "Missing cache. The server did build the cache but the cache seems to be none."
                .to_string(),
        ))?,
    ))
}

#[utoipa::path(
responses(
(status = 200, description = "Single course group. Reviews are also preloaded.", body = GetSingleCourseGroup),
(status = 404, description = "Course group with given id not found.", body = ErrorMessage,
example = json ! (ErrorMessage { message: "Course group with id 1 not found.".to_string() }))
),
security(("auth" = []))
)]
#[get("/group/{group_id}")]
pub async fn get_course_group(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    let group_id = req
        .match_info()
        .query("group_id")
        .parse::<i32>()
        .map_err(|_| bad_request(String::from("Invalid id syntax")))?;
    let group: Vec<(coursegroup::Model, Vec<course::Model>)> = Coursegroup::find_by_id(group_id)
        .find_with_related(Course)
        .all(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;
    if group.is_empty() {
        return Err(not_found(format!(
            "Course group with id {} is not found.",
            group_id
        )));
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
                return Err(internal_server_error(format!(
                    "Unable to load course group with id {}. Error: {}",
                    group_id,
                    e.to_string()
                )));
            }
        }
    }

    Ok(HttpResponse::Ok().json(GetSingleCourseGroup::new(
        group_and_courses.0.clone(),
        course_list,
    )))
}

#[utoipa::path(
request_body = NewCourse,
responses(
(status = 200, description = "Course created successfully.", body = GetSingleCourse),
),
security(("auth" = []))
)]
#[post("/courses")]
pub async fn add_course(
    new_course: web::Json<NewCourse>,
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    if !user_info.is_admin {
        return Err(unauthorized(String::from("Only admin can add new course")));
    }
    let group: Option<coursegroup::Model> = Coursegroup::find()
        .filter(coursegroup::Column::Code.eq(new_course.code.clone()))
        .one(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;

    let new_course = new_course.into_inner();
    let group_id = match group {
        None => {
            // 创建新的 CourseGroup
            let new_course_group: NewCourseGroup = new_course.clone().into();
            let new_course_group: coursegroup::Model = new_course_group
                .into_active_model()
                .insert(db.get_ref())
                .await
                .map_err(|e| {
                    internal_server_error(format!(
                        "Unable to create new course group. Error: {}",
                        e.to_string()
                    ))
                })?;
            new_course_group.id
        }
        Some(group) => group.id,
    };
    // 创建新的 Course
    let new_course: course::Model = new_course
        .into_active_model(group_id)
        .insert(db.get_ref())
        .await
        .map_err(|e| {
            internal_server_error(format!(
                "Unable to create new course. Error: {}",
                e.to_string()
            ))
        })?;

    Ok(HttpResponse::Ok().json(GetSingleCourse::from(new_course)))
}

#[utoipa::path(
responses(
(status = 200, description = "Course. Reviews are also preloaded.", body = GetSingleCourse),
),
security(("auth" = []))
)]
#[get("/courses/{course_id}")]
pub async fn get_course(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    let course_id = req
        .match_info()
        .query("course_id")
        .parse::<i32>()
        .map_err(|_| bad_request(String::from("Invalid id syntax")))?;

    let course: Option<course::Model> = Course::find_by_id(course_id)
        .one(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;
    if course.is_none() {
        return Err(not_found(format!(
            "Course with id {} is not found.",
            course_id
        )));
    }
    // 载入课程的评论列表
    match GetSingleCourse::load(course.unwrap().clone(), db.get_ref(), user_info.id).await {
        Ok(loaded_course) => Ok(HttpResponse::Ok().json(loaded_course)),
        Err(e) => Err(internal_server_error(format!(
            "Unable to load course with id {}. Error: {}",
            course_id,
            e.to_string()
        ))),
    }
}

#[utoipa::path(
request_body = NewReview,
responses(
(status = 200, description = "Review created successfully.", body = GetReview),
(status = 409, description = "The user has already reviewed this course.", body = ErrorMessage,
example = json ! (ErrorMessage { message: "You cannot post more than one review.".to_string() }))
),
security(("auth" = []))
)]
#[post("/courses/{course_id}/reviews")]
pub async fn add_review(
    new_review: web::Json<NewReview>,
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    let new_review = new_review.into_inner();
    let course_id = req
        .match_info()
        .query("course_id")
        .parse::<i32>()
        .map_err(|_| bad_request(String::from("Invalid id syntax")))?;

    // 检查对应课程是否存在
    let course: Option<course::Model> = Course::find_by_id(course_id)
        .one(db.get_ref())
        .await
        .map_err(|e| {
            internal_server_error(format!(
                "Unable to fetch the course. Error: {}",
                e.to_string()
            ))
        })?;
    let course = course.ok_or(not_found(format!(
        "Course with id {} is not found.",
        course_id
    )))?;
    // 防止同一用户创建两条评论
    let course_with_reviews = GetSingleCourse::load(course, db.get_ref(), user_info.id).await;
    match course_with_reviews {
        Ok(course_with_reviews) => {
            if course_with_reviews.review_list.iter().any(|r| r.is_me) {
                return Err(conflict(String::from(
                    "You cannot post more than one review.",
                )));
            }
        }
        Err(err) => {
            return Err(internal_server_error(format!(
                "Unable to fetch the review list of Course with id {}. Error: {}",
                course_id,
                err.to_string()
            )));
        }
    }
    // 创建新评论
    let review_added: review::Model = new_review
        .into_active_model(user_info.id, course_id)
        .insert(db.get_ref())
        .await
        .map_err(|err| {
            internal_server_error(format!(
                "Unable to create new review. Error: {}",
                err.to_string()
            ))
        })?;

    Ok(HttpResponse::Ok().json(
        GetReview::load(review_added, db.get_ref(), user_info.id)
            .await
            .map_err(|e| {
                internal_server_error(format!("Unable to load review. Error: {}", e.to_string()))
            })?,
    ))
}

#[utoipa::path(
request_body = NewReview,
responses(
(status = 200, description = "Review modified successfully.", body = GetReview),
),
security(("auth" = []))
)]
#[put("/reviews/{review_id}")]
pub async fn modify_review(
    new_review: web::Json<NewReview>,
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    let new_review = new_review.into_inner();
    let review_id = req
        .match_info()
        .query("review_id")
        .parse::<i32>()
        .map_err(|_| bad_request(String::from("Invalid id syntax")))?;

    let review: Option<review::Model> = Review::find_by_id(review_id)
        .one(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;
    if review.is_none() {
        return Err(not_found(format!(
            "Review with id {} is not found.",
            review_id
        )));
    }
    // 检查用户对 Review 的修改权限
    let review = review.unwrap();
    if review.reviewer_id != user_info.id && !user_info.is_admin {
        return Err(unauthorized(String::from(
            "You have no permission to modify this review!",
        )));
    }

    // 储存目前的 Review
    let snapshot = serde_json::to_value(&review.clone().into() as &HistoryReview).map_err(|e| {
        internal_server_error(format!(
            "Unable to encode the review into JSON value. Original error: {}",
            e.to_string()
        ))
    })?;
    let array_parsing_error = internal_server_error(String::from(
        "Unable to parse the original review's history fields.",
    ));
    let mut history = (*review.history.as_array().ok_or(array_parsing_error)?).clone();
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
        Ok(updated_review) => Ok(HttpResponse::Ok().json(
            GetReview::load(updated_review, db.get_ref(), user_info.id)
                .await
                .map_err(|e| {
                    internal_server_error(format!(
                        "Unable to load updated review. Error: {}",
                        e.to_string()
                    ))
                })?,
        )),
        Err(err) => Err(internal_server_error(format!(
            "Unable to update the review. Error: {}",
            err.to_string()
        ))),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewVote {
    pub upvote: bool,
}

#[utoipa::path(
request_body = NewVote,
responses(
(status = 200, description = "Review voted successfully.", body = GetReview),
),
security(("auth" = []))
)]
#[patch("/reviews/{review_id}")]
pub async fn vote_for_review(
    vote_data: web::Json<NewVote>,
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    let review_id = req
        .match_info()
        .query("review_id")
        .parse::<i32>()
        .map_err(|_| bad_request(String::from("Invalid id syntax")))?;

    let vote_data = vote_data.into_inner();
    let review: Option<review::Model> = Review::find_by_id(review_id)
        .one(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;

    if review.is_none() {
        return Err(not_found(format!(
            "Review with id {} is not found.",
            review_id
        )));
    }
    let review = review.unwrap();

    // 复制并设置 *voters 列表
    let voters_parsing_error = "Unable to parse the review's voter fields.";
    let mut upvoters = (*review
        .upvoters
        .as_array()
        .ok_or(internal_server_error(voters_parsing_error.to_string()))?)
        .clone();
    let mut downvoters = (*review
        .downvoters
        .as_array()
        .ok_or(internal_server_error(voters_parsing_error.to_string()))?)
        .clone();
    let up_pos = upvoters
        .iter()
        .position(|upvoter_id| upvoter_id.as_i64().unwrap_or(-1) as i32 == user_info.id);
    let down_pos = downvoters
        .iter()
        .position(|downvoter_id| downvoter_id.as_i64().unwrap_or(-1) as i32 == user_info.id);
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
        Ok(updated_review) => Ok(HttpResponse::Ok().json(
            GetReview::load(updated_review, db.get_ref(), user_info.id)
                .await
                .map_err(|e| internal_server_error(e.to_string()))?,
        )),
        Err(err) => Err(internal_server_error(format!(
            "Unable to update the review. Error: {}",
            err.to_string()
        ))),
    }
}

#[utoipa::path(
responses(
(status = 200, description = "Get my reviews. `is_me` is not included.", body = [GetMyReview])
),
security(("auth" = []))
)]
#[get("/reviews/me")]
pub async fn get_reviews(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    let results: Vec<(review::Model, Option<course::Model>)> = Review::find()
        .filter(review::Column::ReviewerId.eq(user_info.id))
        .find_also_related(Course)
        .all(db.get_ref())
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;
    let mut review_list: Vec<GetMyReview> = vec![];
    for x in results {
        let review = x.0;
        let course = x.1.ok_or_else(|| {
            internal_server_error(format!(
                "Unable to find the course of review {}.",
                review.id
            ))
        })?;

        if let Some(group_id) = course.coursegroup_id {
            review_list.push(GetMyReview::new(
                review.clone(),
                course.clone(),
                group_id,
                user_info.id,
            ));
        } else {
            review_list.push(GetMyReview::new(
                review.clone(),
                course.clone(),
                -1,
                user_info.id,
            ));
        }
    }

    Ok(HttpResponse::Ok().json(review_list))
}

#[derive(Debug, FromQueryResult)]
pub struct Counts {
    cnt: i32,
}

#[utoipa::path(
responses(
(status = 200, description = "Get a random review. `is_me` is not included.", body = [GetMyReview])
),
security(("auth" = []))
)]
#[get("/reviews/random")]
pub async fn get_random_reviews(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user_info = require_authentication(&req).await?;
    let review_count: Option<Counts> = Counts::find_by_statement(Statement::from_string(
        db.get_ref().get_database_backend(),
        r#"SELECT COUNT(*) AS cnt FROM review"#.to_string(),
    ))
        .one(db.get_ref())
        .await
        .map_err(|e| {
            internal_server_error(format!(
                "Unable to count the reviews. Error: {}",
                e.to_string()
            ))
        })?;

    let review_count = review_count
        .ok_or(internal_server_error(format!(
            "Unable to count the reviews since database returns no result."
        )))?
        .cnt;
    let mut rng = rand::thread_rng();
    // 重试 5 次
    if review_count == 0 {
        return Err(not_found("No review is found.".to_string()));
    }
    for _ in 1..5 {
        let id = rng.gen_range(1..=review_count) as i32;
        let result: Result<Vec<(review::Model, Option<course::Model>)>, DbErr> =
            Review::find_by_id(id)
                .find_also_related(Course)
                .all(db.get_ref())
                .await;
        if let Ok(results) = result {
            if !results.is_empty() {
                let result = results[0].clone();

                let course = result.1.ok_or_else(|| {
                    internal_server_error(format!(
                        "Unable to find the course of review {}.",
                        result.0.id
                    ))
                })?;
                return if let Some(course_group_link) = course.coursegroup_id {
                    Ok(HttpResponse::Ok().json(GetMyReview::new(
                        result.0.clone(),
                        course.clone(),
                        course_group_link,
                        user_info.id,
                    )))
                } else {
                    Ok(HttpResponse::Ok().json(GetMyReview::new(
                        result.0,
                        course.clone(),
                        -1,
                        user_info.id,
                    )))
                };
            }
        }
    }
    Err(internal_server_error(
        "Unable to fetch a random review. Retry later.".to_string(),
    ))
}

// 将评论标记为删除
// #[delete("/reviews/{review_id}")]
// pub async fn delete_review(req: HttpRequest, review_id: web::Path<i32>, db: web::Data<DatabaseConnection>) -> actix_web::Result<HttpResponse> {
//     let user_info = require_authentication(&req).await?;
//     let review_id = review_id.into_inner();
//     let review: Vec<review::Model> = Review::find_by_id(review_id.into_inner()).all(db.get_ref()).await
//         .map_err(|e| internal_server_error(e.to_string()))?;
//     if review.is_empty() {
//         return Err(not_found(format!("Review with id {} is not found.", review_id)));
//     }
//     let review = &review[0];
//     if review.reviewer_id != user_info.id && !user_info.is_admin {
//         return Err(forbidden("You are not allowed to delete this review.".to_string()));
//     }
//     let mut updated_review: review::ActiveModel = review.clone().into();
//     updated_review.deleted = true;
//     let updated_review: Result<review::Model, DbErr> = updated_review.update(db.get_ref()).await;
//     match updated_review {
//         Ok(updated_review) => Ok(HttpResponse::Ok().json(GetReview::new(updated_review, user_info.id))),
//         Err(err) => Err(internal_server_error(format!("Unable to update the review. Error: {}", err.to_string())))
//     }
// }
