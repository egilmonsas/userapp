use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
pub mod api;
pub mod post;
pub mod session;
pub mod user;
use rocket_dyn_templates::Template;
use std::path::Path;
type HtmlResponse = Result<Template, Status>;

use rocket::Shutdown;
#[get("/shutdown")]
pub async fn shutdown(shutdown: Shutdown) -> &'static str {
    // suppose this variable is from function which
    // produces irrecoverable error
    let result: Result<&str, &str> = Err("err");
    if result.is_err() {
        shutdown.notify();
        return "Shutting down the application.";
    }
    return "Not doing anything.";
}
#[get("/favicon.ico")]
pub async fn favicon() -> NamedFile {
    NamedFile::open(Path::new(relative!("static/favicon.ico")))
        .await
        .ok()
        .unwrap()
}
