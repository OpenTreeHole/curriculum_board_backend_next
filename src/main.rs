mod api;
mod constant;
mod tests;

use std::env;
use api::curriculum_board;
use api::r#static;
use actix_web::{web, App, HttpServer, middleware};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use migration::{Migrator, MigratorTrait};


mod openapi {
    use actix_web::get;
    use utoipa::{Modify, OpenApi};
    use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
    use crate::{
        curriculum_board,
        r#static,
    };
    use entity::course::{GetSingleCourse, NewCourse};
    use entity::coursegroup::{GetMultiCourseGroup, GetSingleCourseGroup, NewCourseGroup};
    use entity::review::{GetMyReview, GetReview, HistoryReview, NewReview, Userextra};
    use entity::user_achievement::GetAchievement;

    struct AuthorizationAddon;

    impl Modify for AuthorizationAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "auth",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()),
            )
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(curriculum_board::hello,
    curriculum_board::get_course_groups_hash,
    curriculum_board::refresh_course_groups_cache,
    curriculum_board::get_course_groups,
    curriculum_board::get_course_group,
    curriculum_board::add_course,
    curriculum_board::get_course,
    curriculum_board::add_review,
    curriculum_board::modify_review,
    curriculum_board::vote_for_review,
    curriculum_board::get_reviews,
    curriculum_board::get_random_reviews,
    r#static::cedict
    ),
    components(schemas(
    GetMultiCourseGroup,
    GetSingleCourseGroup,
    NewCourseGroup,
    GetSingleCourse,
    GetMyReview,
    GetReview,
    Userextra,
    HistoryReview,
    NewReview,
    NewCourse,
    GetAchievement,
    curriculum_board::HashMessage,
    curriculum_board::NewVote)),
    modifiers(& AuthorizationAddon))]
    pub(crate) struct ApiDoc;

    #[get("/openapi.json")]
    pub async fn get_openapi() -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok()
            .content_type("application/json")
            .body(ApiDoc::openapi().to_pretty_json().unwrap())
    }
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(curriculum_board::hello)
        .service(curriculum_board::get_course_groups_hash)
        .service(curriculum_board::refresh_course_groups_cache)
        .service(curriculum_board::get_course_groups)
        .service(curriculum_board::get_course_group)
        .service(curriculum_board::add_course)
        .service(curriculum_board::get_course)
        .service(curriculum_board::add_review)
        .service(curriculum_board::modify_review)
        .service(curriculum_board::vote_for_review)
        .service(curriculum_board::get_reviews)
        .service(curriculum_board::get_random_reviews)
        .service(r#static::cedict)
        .service(openapi::get_openapi);
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化 dotenv
    dotenv().ok();
    let db: DatabaseConnection = Database::connect(env::var(constant::ENV_DB_URL).unwrap()).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .configure(config)
            .app_data(web::Data::new(db.clone()))
    })
        .bind(("0.0.0.0", 11451))?
        .run()
        .await
}

