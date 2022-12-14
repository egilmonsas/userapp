use crate::errors::our_error::OurError;
use crate::fairings::db::DBConnection;
use crate::guards::auth::APIUser;
use crate::models::{
    pagination::Pagination,
    user::{Auth, JWTLogin, User, UsersWrapper},
};
use crate::states::JWToken;
use hmac::{Hmac, Mac};
use jwt::{Header, Token, VerifyWithKey};
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::{sqlx::Acquire, Connection};
use sha2::Sha256;
use std::collections::BTreeMap;

#[get("/users", format = "json", data = "<pagination>")]
pub async fn users(
    mut db: Connection<DBConnection>,
    pagination: Option<Json<Pagination>>,
) -> Result<Json<UsersWrapper>, Json<OurError>> {
    let parsed_pagination = pagination.map(|p| p.into_inner());
    let (users, new_pagination) = User::find_all(&mut db, parsed_pagination)
        .await
        .map_err(|_| OurError::new_internal_server_error(String::from("Internal Error"), None))?;
    Ok(Json(UsersWrapper {
        users,
        pagination: new_pagination,
    }))
}

#[post("/login", format = "json", data = "<jwt_login>")]
pub async fn login<'r>(
    mut db: Connection<DBConnection>,
    jwt_login: Option<Json<JWTLogin<'r>>>,
    jwt_secret: &State<JWToken>,
) -> Result<Json<Auth>, Json<OurError>> {
    let connection = db
        .acquire()
        .await
        .map_err(|_| OurError::new_internal_server_error(String::from("Cannot1 login"), None))?;
    log::info!("{:?}", jwt_login);
    log::info!("{:?}", jwt_secret);
    let parsed_jwt_login = jwt_login
        .map(|p| p.into_inner())
        .ok_or_else(|| OurError::new_bad_request_error(String::from("Cannot2 login"), None))?;
    log::info!("{:?}", parsed_jwt_login);
    Ok(Json(
        parsed_jwt_login
            .authenticate(connection, &jwt_secret.secret)
            .await
            .map_err(|_| {
                OurError::new_internal_server_error(String::from("Cannot3 login"), None)
            })?,
    ))
}

#[get("/protected_users", format = "json", data = "<pagination>")]
pub async fn authenticated_users(
    db: Connection<DBConnection>,
    pagination: Option<Json<Pagination>>,
    _authorized_user: APIUser,
) -> Result<Json<UsersWrapper>, Json<OurError>> {
    users(db, pagination).await
}
