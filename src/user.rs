use crate::db::Db;
use crate::models::RocketAuthUser;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    visited_user: Option<RocketAuthUser>,
}

impl Context {
    // pub async fn err<M: std::fmt::Display>(
    //     db: Connection<Db>,
    //     msg: M,
    //     user: Option<User>,
    // ) -> Context {
    //     Context {
    //         flash: Some(("error".into(), msg.to_string())),
    //         listings: Listing::all(db).await.unwrap_or_default(),
    //         user: user,
    //     }
    // }

    pub async fn raw(
        mut db: Connection<Db>,
        username: String,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Context {
        match RocketAuthUser::single_by_username(&mut db, username).await {
            Ok(rocket_auth_user) => Context {
                flash,
                visited_user: Some(rocket_auth_user),
                user,
                admin_user,
            },
            Err(e) => {
                error_!("DB RocketAuthUser::single_by_username() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    visited_user: None,
                    user: user,
                    admin_user: admin_user,
                }
            }
        }
    }
}

// #[put("/<id>")]
// async fn toggle(id: i32, mut db: Connection<Db>, user: User) -> Result<Redirect, Template> {
//     match Task::toggle_with_id(id, &mut db).await {
//         Ok(_) => Ok(Redirect::to("/")),
//         Err(e) => {
//             error_!("DB toggle({}) error: {}", id, e);
//             Err(Template::render(
//                 "index",
//                 Context::err(db, "Failed to toggle task.", Some(user)).await,
//             ))
//         }
//     }
// }

// #[delete("/<id>")]
// async fn delete(id: i32, mut db: Connection<Db>, user: User) -> Result<Flash<Redirect>, Template> {
//     match Task::delete_with_id(id, &mut db).await {
//         Ok(_) => Ok(Flash::success(Redirect::to("/"), "Listing was deleted.")),
//         Err(e) => {
//             error_!("DB deletion({}) error: {}", id, e);
//             Err(Template::render(
//                 "index",
//                 Context::err(db, "Failed to delete task.", Some(user)).await,
//             ))
//         }
//     }
// }

#[get("/<username>")]
async fn index(
    username: &str,
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "user",
        Context::raw(db, username.to_string(), flash, user, admin_user).await,
    )
}

pub fn user_stage() -> AdHoc {
    AdHoc::on_ignite("User Stage", |rocket| async {
        rocket.mount("/user", routes![index])
    })
}