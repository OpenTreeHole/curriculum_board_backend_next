use std::num::ParseIntError;
use actix_web::{get, post, HttpResponse, Responder, web, HttpRequest};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, EntityTrait};
use serde_json::Value;
use crate::api::auth::require_authentication;
use crate::api::error_handler::{ErrorMessage, internal_server_error, not_found};
use crate::entity::prelude::*;
use sea_orm::QueryTrait;
use crate::CourseGroup::{GetMultiCourseGroup, GetSingleCourseGroup};
use crate::entity::{course, coursegroup};
use sea_orm::ModelTrait;
use crate::entity::course::GetSingleCourse;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Welcome to curriculum_board backend. Search for API documents on GitHub please.")
}

#[get("/courses")]
pub async fn get_course_groups(req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let result: Result<Vec<(coursegroup::Model, Vec<course::Model>)>, DbErr> =
        Coursegroup::find().find_with_related(Course).all(db.get_ref()).await;
    match result {
        Ok(groups) => {
            let mut group_list: Vec<GetMultiCourseGroup> = vec![];
            for x in groups {
                group_list.push(GetMultiCourseGroup::new(x.0, x.1));
            }
            HttpResponse::Ok().json(group_list)
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
                        return not_found(format!("Course group with id {} is not found", group_id));
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