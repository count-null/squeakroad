CREATE TABLE bounties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id INTEGER NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    price_sat UNSIGNED BIG INT NOT NULL,
    fee_rate_basis_points INTEGER NOT NULL,
    viewed BOOLEAN NOT NULL,
    submitted BOOLEAN NOT NULL,
    approved BOOLEAN NOT NULL,
    deactivated_by_seller boolean NOT NULL,
    deactivated_by_admin boolean NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL
);

CREATE TABLE bountyimages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    bounty_id INTEGER NOT NULL,
    image_data BLOB NOT NULL,
    is_primary BOOLEAN NOT NULL
);

CREATE TABLE shippingoptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    bounty_id INTEGER NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    price_sat UNSIGNED BIG INT NOT NULL
);

CREATE TABLE cases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    buyer_user_id INTEGER NOT NULL,
    seller_user_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    bounty_id INTEGER NOT NULL,
    case_details VARCHAR NOT NULL,
    amount_owed_sat UNSIGNED BIG INT NOT NULL,
    seller_credit_sat UNSIGNED BIG INT NOT NULL,
    paid BOOLEAN NOT NULL,
    awarded BOOLEAN NOT NULL,
    canceled_by_seller boolean NOT NULL,
    canceled_by_buyer boolean NOT NULL,
    invoice_payment_request VARCHAR NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    payment_time_ms UNSIGNED BIG INT NOT NULL
);

CREATE TABLE withdrawals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id INTEGER NOT NULL,
    amount_sat UNSIGNED BIG INT NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    invoice_payment_request VARCHAR NOT NULL
);

CREATE TABLE useraccounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id Integer NOT NULL,
    amount_owed_sat UNSIGNED BIG INT NOT NULL,
    paid BOOLEAN NOT NULL,
    disabled boolean NOT NULL,
    invoice_payment_request VARCHAR NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    payment_time_ms UNSIGNED BIG INT NOT NULL
);

CREATE TABLE usersettings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    pgp_key VARCHAR NOT NULL
);

CREATE TABLE adminsettings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_name VARCHAR NOT NULL,
    market_info VARCHAR NOT NULL,
    fee_rate_basis_points INTEGER NOT NULL,
    user_bond_price_sat UNSIGNED BIG INT NOT NULL,
    pgp_key VARCHAR NOT NULL,
    max_allowed_users UNSIGNED BIG INT NOT NULL
);

