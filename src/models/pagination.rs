use super::our_date_time::OurDateTime;
use rocket::form::FromForm;

#[derive(FromForm)]
pub struct Pagination {
    pub next: OurDateTime,
    pub limit: usize,
}
