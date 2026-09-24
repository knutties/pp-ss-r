use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaymentPage {
    pub init: InitPayload,
    pub process: ProcessPayload,
}

#[derive(Debug, Deserialize)]
pub struct InitPayload {
    pub client_auth_token: String,
    pub client_id: String,
    pub environment: String,
    pub merchant_id: String,
    pub return_url: String,
    pub language: String,
    pub is_payment_link: bool,
    pub tenant: TenantInfo,
}

#[derive(Debug, Deserialize)]
pub struct TenantInfo {
    pub assets_domain: String,
    pub tenant_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessPayload {
    pub amount: String,
    pub currency: String,
    pub order_id: String,
    pub description: String,
    pub order_type: String,
    pub payment_links_expiry: String,
    pub source_object: String,
    pub merchant_id: String,
    pub metadata: String,
}

pub fn load_sample() -> PaymentPage {
    let raw = include_str!("../data/order.json");
    serde_json::from_str(raw).expect("data/order.json must be valid PaymentPage JSON")
}
