use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::content::RawHtml;

pub mod post;
pub mod user;

type HtmlResponse = Result<RawHtml<String>, Status>;

#[get("/<_filename>")]
pub async fn assets(_filename: &str) -> NamedFile {
    todo!("will implement later")
}

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
