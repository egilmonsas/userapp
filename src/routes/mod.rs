use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
use rocket::response::content::RawHtml;
pub mod post;
pub mod user;
use std::path::{Path, PathBuf};
type HtmlResponse = Result<RawHtml<String>, Status>;

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

#[get("/<filename..>")]
pub async fn assets(filename: PathBuf) -> Option<NamedFile> {
    let mut filename = Path::new(relative!("static")).join(filename);
    if filename.is_dir() {
        filename.push("index.html");
    }
    NamedFile::open(filename).await.ok()
}
