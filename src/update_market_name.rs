use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, MarketNameInput};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

impl AdminSettings {
    pub fn get_default() -> AdminSettings {
        AdminSettings {
            id: None,
            market_name: "Squeakroad".to_string(),
            fee_rate_basis_points: 500,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    admin_settings: AdminSettings,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        Ok(Context {
            base_context,
            flash,
            admin_settings,
        })
    }
}

#[post("/change", data = "<market_name_form>")]
async fn update(
    market_name_form: Form<MarketNameInput>,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Flash<Redirect> {
    let market_name_input = market_name_form.into_inner();
    let new_market_name = market_name_input.market_name;

    match change_market_name(new_market_name, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_market_name", index())),
            "Market name successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_market_name", index())), e),
    }

    // Flash::error(
    //     Redirect::to(uri!("/update_shipping_options", index(id))),
    //     "not implemented".to_string(),
    // )
}

async fn change_market_name(
    new_market_name: String,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    if new_market_name.is_empty() {
        Err("Market name cannot be empty.".to_string())
    } else if new_market_name.len() >= 64 {
        Err("Market name is too long.".to_string())
    } else {
        let default_admin_settings = AdminSettings::get_default();
        AdminSettings::set_market_name(db, &new_market_name, default_admin_settings)
            .await
            .map_err(|_| "failed to update market name.")?;

        Ok(())
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "updatemarketname",
        Context::raw(db, flash, user, Some(admin_user)).await,
    )
}

pub fn update_market_name_stage() -> AdHoc {
    AdHoc::on_ignite("Update Market Name Stage", |rocket| async {
        rocket
            // .mount("/update_listing_images", routes![index, new])
            .mount("/update_market_name", routes![index, update])
    })
}
