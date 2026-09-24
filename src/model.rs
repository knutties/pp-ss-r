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

impl PaymentPage {
    /// Rejects payloads whose display-critical fields are blank. Deserialization
    /// already rejects *missing* keys; this catches *empty* string values, which
    /// are valid JSON strings but would render a meaningless page (e.g. "Pay £").
    pub fn validate(&self) -> Result<(), String> {
        let required = [
            ("init.merchant_id", &self.init.merchant_id),
            ("process.amount", &self.process.amount),
            ("process.currency", &self.process.currency),
            ("process.order_id", &self.process.order_id),
            ("process.merchant_id", &self.process.merchant_id),
        ];
        for (name, value) in required {
            if value.trim().is_empty() {
                return Err(format!("required field `{name}` is blank"));
            }
        }
        Ok(())
    }
}

/// Parses and validates the embedded sample payload. Returns an error (rather
/// than panicking) so callers can decide how to surface it.
pub fn load_sample_checked() -> Result<PaymentPage, String> {
    let raw = include_str!("../data/order.json");
    let page: PaymentPage = serde_json::from_str(raw)
        .map_err(|e| format!("data/order.json is not valid PaymentPage JSON: {e}"))?;
    page.validate()?;
    Ok(page)
}

pub fn load_sample() -> PaymentPage {
    load_sample_checked().expect("data/order.json must be a valid, complete PaymentPage")
}
