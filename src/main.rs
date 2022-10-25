#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};
use userapp::setup_rocket;

#[launch]
async fn rocket() -> Rocket<Build> {
    setup_rocket().await
}
