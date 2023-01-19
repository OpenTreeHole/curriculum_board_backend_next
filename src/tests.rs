

#[cfg(test)]
mod tests {
    use actix_web::{App, http::{self}, test, web};
    use actix_web::body::{MessageBody};
    use actix_web::dev::{ServiceResponse};
    use actix_web::test::TestRequest;
    use async_once_cell::OnceCell;
    use lazy_static::lazy_static;
    use sea_orm::{Database, DatabaseConnection};
    use crate::{config};
    use migration::{Migrator, MigratorTrait};

    static DB: OnceCell<DatabaseConnection> = OnceCell::new();
    macro_rules! ensure_app_built {
        () => (
            {
                let db = DB.get_or_init(async {
                    let db = Database::connect("sqlite::memory:").await.unwrap();
                    setup_schema(&db).await;
                    db
                }).await;
                test::init_service(App::new().configure(config).app_data(web::Data::new(db.clone()))).await
            }
        )
    }
    async fn setup_schema(db: &DatabaseConnection) {
        Migrator::fresh(db).await.unwrap();
    }

    #[actix_web::test]
    async fn test_all() {
        test_about().await;
        test_group_cache().await;
        test_group().await;
        // test_random().await;
    }

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


    async fn test_group_cache() {
        let app = ensure_app_built!();

        // refresh cache
        let resp = test::call_service(&app, TestRequest::get().uri("/courses/refresh").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::IM_A_TEAPOT);
        // get cache hash
        let resp = test::call_service(&app, TestRequest::get().uri("/courses/hash").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = get_body(resp);
        let result = serde_json::from_str::<serde_json::Value>(&body).unwrap();
        assert!(result.as_object().unwrap().contains_key("hash"));
    }

    async fn test_group() {
        let app = ensure_app_built!();

        // get course groups
        let resp = test::call_service(&app, TestRequest::get().uri("/courses").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let result = serde_json::from_str::<serde_json::Value>(&get_body(resp)).unwrap();
        assert!(result.as_array().unwrap().len() >= 0);

        // get course group
        // let resp = test::call_service(&app, TestRequest::get().uri("/group/1").to_request()).await;
        // assert_eq!(resp.status(), http::StatusCode::OK);
        // let result = serde_json::from_str::<serde_json::Value>(&get_body(resp)).unwrap();
        // assert_eq!(result.as_object().unwrap()["id"].as_i64().unwrap(), 1);
    }

    async fn test_random() {
        let app = ensure_app_built!();
        let resp = test::call_service(&app, TestRequest::get().uri("/reviews/random").to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
        let result = serde_json::from_str::<serde_json::Value>(&get_body(resp)).unwrap();
    }
}