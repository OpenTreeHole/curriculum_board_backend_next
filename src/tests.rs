use std::sync::RwLock;
use crate::curriculum_board::*;


#[cfg(test)]
mod tests {
    use std::env;
    use super::*;
    use actix_web::{App, http::{self, header::ContentType}, HttpMessage, test, web};
    use actix_web::body::{BoxBody, MessageBody};
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::test::TestRequest;
    use actix_web::web::Bytes;
    use async_once_cell::OnceCell;
    use sea_orm::{Database, DatabaseBackend, DatabaseConnection};
    use crate::{config, constant};
    use dotenv::dotenv;

    static DB: OnceCell<DatabaseConnection> = OnceCell::new();

    macro_rules! ensure_app_built {
        () => (
            {
                let db = DB.get_or_init(async {
                    dotenv().ok();
                    Database::connect(env::var(constant::ENV_DB_URL).unwrap()).await.unwrap()
                }).await;
                test::init_service(App::new().configure(config).app_data(web::Data::new(db.clone()))).await
            }
        )
    }

    #[actix_web::test]
    async fn test_about() {
        let app = ensure_app_built!();
        let resp = test::call_service(&app, TestRequest::get().uri("/").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    fn get_body(resp: ServiceResponse) -> String {
        // ServiceResponse -> BoxBody -> Bytes
        let resp = resp.into_body().try_into_bytes().unwrap();
        // Bytes -> &[u8] -> String
        String::from_utf8_lossy(&*resp).to_string()
    }


    #[actix_web::test]
    async fn test_group_cache() {
        let app = ensure_app_built!();

        // refresh cache
        let resp = test::call_service(&app, TestRequest::get().uri("/courses/refresh").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::IM_A_TEAPOT);

        // get cache hash
        let resp = test::call_service(&app, TestRequest::get().uri("/courses/hash").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let result = serde_json::from_str::<serde_json::Value>(&get_body(resp)).unwrap();
        assert!(result.as_object().unwrap().contains_key("hash"));
    }

    #[actix_web::test]
    async fn test_group() {
        let app = ensure_app_built!();

        // get course groups
        let resp = test::call_service(&app, TestRequest::get().uri("/courses").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let result = serde_json::from_str::<serde_json::Value>(&get_body(resp)).unwrap();
        assert!(result.as_array().unwrap().len() >= 0);

        // get course group
        let resp = test::call_service(&app, TestRequest::get().uri("/group/1").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let result = serde_json::from_str::<serde_json::Value>(&get_body(resp)).unwrap();
        assert_eq!(result.as_object().unwrap()["id"].as_i64().unwrap(), 1);
    }
}