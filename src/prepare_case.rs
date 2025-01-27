use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{Bounty, BountyDisplay, Case, CaseInfo,  UserSettings};
use crate::user_account::ActiveUser;
use crate::util;
use pgp::composed::{Deserializable, Message};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket::State;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

const MAX_UNPAID_ORDERS: u32 = 100;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    bounty_display: Option<BountyDisplay>,
    quantity: i32,
    seller_user_settings: UserSettings,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        bounty_id: &str,
        quantity: i32,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let bounty_display = BountyDisplay::single_by_public_id(&mut db, bounty_id)
            .await
            .map_err(|_| "failed to get admin settings.")?;
        let seller_user_settings = UserSettings::single(&mut db, bounty_display.bounty.user_id)
            .await
            .map_err(|_| "failed to get visited user settings.")?;
        Ok(Context {
            base_context,
            flash,
            bounty_display: Some(bounty_display),
            quantity,
            seller_user_settings,
        })
    }
}

#[post("/<id>/new", data = "<case_form>")]
async fn new(
    id: &str,
    case_form: Form<CaseInfo>,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    _admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let case_info = case_form.into_inner();

    match create_case(
        id,
        case_info.clone(),
        &mut db,
        active_user.user.clone(),
        config.inner().clone(),
    )
    .await
    {
        Ok(case_id) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "case", case_id)),
            "Case successfully created.",
        )),
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(Flash::error(
                Redirect::to(uri!(
                    "/prepare_case",
                    index(id, 1)
                )),
                e,
            ))
        }
    }
}

async fn create_case(
    bounty_id: &str,
    case_info: CaseInfo,
    db: &mut Connection<Db>,
    user: User,
    config: Config,
) -> Result<String, String> {
    let bounty = Bounty::single_by_public_id(db, bounty_id)
        .await
        .map_err(|_| "failed to get bounty")?;
    let now = util::current_time_millis();
    let case_details = case_info.case_details;
    let quantity = case_info.quantity.unwrap_or(0);

    let amount_owed_sat: u64 = (quantity as u64) * bounty.price_sat; 
    // let market_fee_sat: u64 = (amount_owed_sat * (bounty.fee_rate_basis_points as u64)) / 10000;
    let market_fee_sat: u64 = divide_round_up(
        amount_owed_sat * (bounty.fee_rate_basis_points as u64),
        10000,
    );
    let seller_credit_sat: u64 = amount_owed_sat - market_fee_sat;

    let (message, _) =
        Message::from_string(&case_details).map_err(|_| "Invalid PGP message.")?;
    info!("message: {:?}", &message);

    if case_details.is_empty() {
        return Err("Case details cannot be empty.".to_string());
    };
    if case_details.len() > 4096 {
        return Err("Case details length is too long.".to_string());
    };
    if bounty.user_id == user.id() {
        return Err("Bounty belongs to same user as buyer.".to_string());
    };
    if !bounty.approved {
        return Err("Bounty has not been approved by admin.".to_string());
    };
    if bounty.deactivated_by_seller || bounty.deactivated_by_admin {
        return Err("Bounty has been deactivated.".to_string());
    };
    if user.is_admin {
        return Err("Admin user cannot create an case.".to_string());
    };
    if quantity == 0 {
        return Err("Quantity must be postive.".to_string());
    };

    let mut lightning_client = lightning::get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let invoice = lightning_client
        .add_invoice(tonic_openssl_lnd::lnrpc::Invoice {
            value_msat: (amount_owed_sat as i64) * 1000,
            ..Default::default()
        })
        .await
        .expect("failed to get new invoice")
        .into_inner();

    let case = Case {
        id: None,
        public_id: util::create_uuid(),
        quantity,
        buyer_user_id: user.id(),
        seller_user_id: bounty.user_id,
        bounty_id: bounty.id.unwrap(),
        case_details: case_details.to_string(),
        amount_owed_sat,
        seller_credit_sat,
        paid: false,
        awarded: false,
        canceled_by_seller: false,
        canceled_by_buyer: false,
        invoice_hash: util::to_hex(&invoice.r_hash),
        invoice_payment_request: invoice.payment_request,
        created_time_ms: now,
        payment_time_ms: 0,
    };

    match Case::insert(case, MAX_UNPAID_ORDERS, db).await {
        Ok(case_id) => match Case::single(db, case_id).await {
            Ok(new_case) => Ok(new_case.public_id),
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("New case could not be found after inserting.".to_string())
            }
        },
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(e)
        }
    }
}

fn divide_round_up(dividend: u64, divisor: u64) -> u64 {
    (dividend + divisor - 1) / divisor
}

#[get("/<id>?<quantity>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    quantity: usize,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(
        db,
        id,
        quantity.try_into().unwrap(),
        flash,
        active_user.user,
        admin_user,
    )
    .await
    .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("preparecase", context))
}

pub fn prepare_case_stage() -> AdHoc {
    AdHoc::on_ignite("Prepare Case Stage", |rocket| async {
        rocket.mount("/prepare_case", routes![index, new])
    })
}
