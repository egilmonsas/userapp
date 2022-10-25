use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rocket_db_pools::sqlx::PgConnection;
use scraper::{Html, Selector};
use sqlx::postgres::PgPoolOptions;
use userapp::models::user::User;
use userapp::{self, Config};

#[rocket::async_trait]
trait ModelCleaner {
    async fn clear_all(connection: &mut PgConnection) -> Result<(), String>;
}

#[rocket::async_trait]
impl ModelCleaner for User {
    async fn clear_all(connection: &mut PgConnection) -> Result<(), String> {
        let _ = sqlx::query("TRUNCATE users RESTART IDENTITY CASCADE")
            .execute(connection)
            .await
            .map_err(|_| String::from("error truncating databasse"))?;
        Ok(())
    }
}

#[rocket::async_test]
async fn test_rocket() {
    let rocket = userapp::setup_rocket().await;
    let config_wrapper = rocket.figment().extract();
    assert!(config_wrapper.is_ok());

    let config: Config = config_wrapper.unwrap();
    let db_url = config.get_database_url();
    let db_wrapper = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await;
    assert!(db_wrapper.is_ok());

    let db = db_wrapper.unwrap();
    let pg_connection_wrapper = db.acquire().await;
    assert!(pg_connection_wrapper.is_ok());

    let mut pg_connection = pg_connection_wrapper.unwrap();
    let clear_result_wrapper = User::clear_all(&mut pg_connection).await;
    assert!(clear_result_wrapper.is_ok());

    let client_wrapper = Client::tracked(rocket).await;
    assert!(client_wrapper.is_ok());

    let client = client_wrapper.unwrap();
    let req = client.get("/users");
    let resp = req.dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let body_wrapper = resp.into_string().await;
    assert!(body_wrapper.is_some());

    let body = Html::parse_document(&body_wrapper.unwrap());
    let selector = Selector::parse(r#"mark.tag"#).unwrap();
    let containers = body.select(&selector);
    let num_of_elements = containers.count();
    assert_eq!(num_of_elements, 0);

    let req = client.get("/users/new");
    let resp = req.dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let body_wrapper = resp.into_string().await;
    assert!(body_wrapper.is_some());

    let body = Html::parse_document(&body_wrapper.unwrap());
    let authenticity_token_selector =
        Selector::parse(r#"input[name="authenticity_token"]"#).unwrap();

    let element_wrapper = body.select(&authenticity_token_selector).next();
    assert!(element_wrapper.is_some());

    let element = element_wrapper.unwrap();
    let value_wrapper = element.value().attr("value");
    assert!(value_wrapper.is_some());

    let authenticity_token = value_wrapper.unwrap();
    let username = "testing123";
    let password = "lkjKLAJ09231478mlasdfkjsdkj";
    let req = client
        .post("/users")
        .header(ContentType::Form)
        .body(format!(
            "authenticity_token={}&username={}&email={}@{}.com&password={}&password_confirmation={}&description=",
            authenticity_token, username, username, username, password, password,
        ));
    let resp = req.dispatch().await;
    assert_eq!(resp.status(), Status::SeeOther);

    let req = client.get("/users");
    let resp = req.dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let body_wrapper = resp.into_string().await;
    assert!(body_wrapper.is_some());

    let body = Html::parse_document(&body_wrapper.unwrap());
    let selector = Selector::parse(r#"mark.tag"#).unwrap();
    let containers = body.select(&selector);
    let num_of_elements = containers.count();
    assert_eq!(num_of_elements, 1);
}
