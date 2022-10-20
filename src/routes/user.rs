use super::HtmlResponse;
use crate::fairings::csrf::Token as CsrfToken;
use crate::fairings::db::DBConnection;
use crate::models::{
    pagination::Pagination,
    user::{EditedUser, NewUser, User},
};
use rocket::form::{Contextual, Form};
use rocket::http::Status;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_db_pools::{sqlx::Acquire, Connection};
use rocket_dyn_templates::{context, Template};

#[get("/users/<uuid>", format = "text/html")]
pub async fn get_user(
    mut db: Connection<DBConnection>,
    uuid: &str,
    flash: Option<FlashMessage<'_>>,
) -> HtmlResponse {
    let connection = db
        .acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let user = User::find(connection, uuid).await.map_err(|e| e.status)?;
    #[derive(Serialize)]
    struct GetUser {
        user: User,
        flash: Option<String>,
    }
    let flash_message = flash.map(|fm| String::from(fm.message()));
    let context = GetUser {
        user,
        flash: flash_message,
    };
    Ok(Template::render("users/show", &context))
}

#[get("/users?<pagination>", format = "text/html")]
pub async fn get_users(
    mut db: Connection<DBConnection>,
    pagination: Option<Pagination>,
) -> HtmlResponse {
    let (users, new_pagination) = User::find_all(&mut db, pagination)
        .await
        .map_err(|_| Status::NotFound)?;
    let context = context! {users: users, pagination: new_pagination.map(|pg|pg.to_context())};
    Ok(Template::render("users/index", context))
}

#[get("/users/new", format = "text/html")]
pub async fn new_user(flash: Option<FlashMessage<'_>>, csrf_token: CsrfToken) -> HtmlResponse {
    let flash_string = flash
        .map(|fl| format!("{}", fl.message()))
        .unwrap_or_else(|| "".to_string());
    let context = context! {
        edit: false,
        form_url: "/users",
        legend: "New User",
        flash: flash_string,
        csrf_token: csrf_token,
    };
    Ok(Template::render("users/form", context))
}

#[post(
    "/users",
    format = "application/x-www-form-urlencoded",
    data = "<user_context>"
)]
pub async fn create_user<'r>(
    mut db: Connection<DBConnection>,
    user_context: Form<Contextual<'r, NewUser<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if user_context.value.is_none() {
        let error_message = format!(
            "<div>{}</div>",
            user_context
                .context
                .errors()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("<br/>")
        );
        return Err(Flash::error(Redirect::to("/users/new"), error_message));
    }
    let new_user = user_context.value.as_ref().unwrap();
    csrf_token
        .verify(&new_user.authenticity_token)
        .map_err(|_| {
            Flash::error(
                Redirect::to("/users/new"),
                "Something went wrong when creating user",
            )
        })?;
    let connection = db.acquire().await.map_err(|_| {
        Flash::error(
            Redirect::to("/users/new"),
            "<div>Something went wrong when creating user</div>",
        )
    })?;
    let user = User::create(connection, new_user).await.map_err(|_| {
        Flash::error(
            Redirect::to("/users/new"),
            "<div>Something went wrong when creating user</div>",
        )
    })?;
    Ok(Flash::success(
        Redirect::to(format!("/users/{}", user.uuid)),
        "<div>Successfully created user</div>",
    ))
}

#[get("/users/edit/<uuid>", format = "text/html")]
pub async fn edit_user(
    mut db: Connection<DBConnection>,
    uuid: &str,
    flash: Option<FlashMessage<'_>>,
    csrf_token: CsrfToken,
) -> HtmlResponse {
    let connection = db
        .acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let user = User::find(connection, uuid)
        .await
        .map_err(|_| Status::NotFound)?;
    let flash_string = flash
        .map(|fl| format!("{}", fl.message()))
        .unwrap_or_else(|| "".to_string());
    let context = context! {
        form_url: format!("/users/{}",&user.uuid ),
        edit: true,
        legend: "Edit User",
        flash: flash_string,
        user,
        csrf_token: csrf_token,
    };

    Ok(Template::render("users/form", context))
}

#[post(
    "/users/<uuid>",
    format = "application/x-www-form-urlencoded",
    data = "<user_context>"
)]
pub async fn update_user<'r>(
    db: Connection<DBConnection>,
    uuid: &str,
    user_context: Form<Contextual<'r, EditedUser<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if user_context.value.is_none() {
        let error_message = format!(
            "<div>{}</div>",
            user_context
                .context
                .errors()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("<br/>")
        );
        return Err(Flash::error(
            Redirect::to(format!("/users/edit/{}", uuid)),
            error_message,
        ));
    }
    let user_value = user_context.value.as_ref().unwrap();
    match user_value.method {
        "PUT" => put_user(db, uuid, user_context, csrf_token).await,
        "PATCH" => patch_user(db, uuid, user_context, csrf_token).await,
        _ => Err(Flash::error(
            Redirect::to(format!("/users/edit/{}", uuid)),
            "<div>Something went wrong when updating user</div>",
        )),
    }
}

#[put(
    "/users/<uuid>",
    format = "application/x-www-form-urlencoded",
    data = "<user_context>"
)]
pub async fn put_user<'r>(
    mut db: Connection<DBConnection>,
    uuid: &str,
    user_context: Form<Contextual<'r, EditedUser<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let user_value = user_context.value.as_ref().unwrap();
    csrf_token
        .verify(&user_value.authenticity_token)
        .map_err(|_| {
            Flash::error(
                Redirect::to(format!("/users/edit/{}", uuid)),
                "Something went wrong when updating user",
            )
        })?;
    let user = User::update(&mut db, uuid, user_value).await.map_err(|_| {
        Flash::error(
            Redirect::to(format!("/users/edit/{}", uuid)),
            "<div>Something went wrong when updating user</div>",
        )
    })?;
    Ok(Flash::success(
        Redirect::to(format!("/users/{}", user.uuid)),
        "<div>Successfully updated user</div>",
    ))
}

#[patch(
    "/users/<uuid>",
    format = "application/x-www-form-urlencoded",
    data = "<user_context>"
)]
pub async fn patch_user<'r>(
    db: Connection<DBConnection>,
    uuid: &str,
    user_context: Form<Contextual<'r, EditedUser<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    put_user(db, uuid, user_context, csrf_token).await
}

#[post("/users/delete/<uuid>", format = "application/x-www-form-urlencoded")]
pub async fn delete_user_entry_point(
    db: Connection<DBConnection>,
    uuid: &str,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    delete_user(db, uuid).await
}

#[delete("/users/<uuid>", format = "application/x-www-form-urlencoded")]
pub async fn delete_user(
    mut db: Connection<DBConnection>,
    uuid: &str,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let connection = db.acquire().await.map_err(|_| {
        Flash::error(
            Redirect::to("/users"),
            "<div>Something went wrong when deleting user</div>",
        )
    })?;
    User::destroy(connection, uuid)
        .await
        .map_err(|e| Flash::error(Redirect::to("/users"), format!("<div>{}</div>", e)))?;

    Ok(Flash::success(
        Redirect::to("/users"),
        "<div>Successfully deleted user</div>",
    ))
}
