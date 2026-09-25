use serde::{Deserialize, Serialize};

/// Request body for `POST /v1/checkout-sessions`.
#[derive(Debug, Serialize)]
pub struct CheckoutRequest {
    pub merchant_reference: String,
    pub mode: String,
    pub objective: Objective,
    pub experience: Experience,
    pub return_urls: ReturnUrls,
}

#[derive(Debug, Serialize)]
pub struct Objective {
    #[serde(rename = "type")]
    pub kind: String,
    pub payment: Payment,
}

#[derive(Debug, Serialize)]
pub struct Payment {
    pub merchant_id: String,
    pub amount: Amount,
    pub processing_mode: String,
    pub payment_purpose: String,
    pub payment_channel: String,
}

#[derive(Debug, Serialize)]
pub struct Amount {
    pub currency: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Experience {
    pub branding_profile: String,
}

#[derive(Debug, Serialize)]
pub struct ReturnUrls {
    pub default: String,
}

/// Response from the checkout API. Loosely typed — we only need the session id.
#[derive(Debug, Default, Deserialize)]
pub struct CheckoutResponse {
    #[serde(default)]
    pub id: Option<String>,
}

/// Assembles a HOSTED_PAGE / AUTH_AND_CAPTURE payment checkout request.
pub fn build_request(
    amount: &str,
    currency: &str,
    merchant_id: &str,
    return_url: &str,
    branding_profile: &str,
    merchant_reference: &str,
) -> CheckoutRequest {
    CheckoutRequest {
        merchant_reference: merchant_reference.to_string(),
        mode: "HOSTED_PAGE".to_string(),
        objective: Objective {
            kind: "PAYMENT".to_string(),
            payment: Payment {
                merchant_id: merchant_id.to_string(),
                amount: Amount {
                    currency: currency.to_string(),
                    value: amount.to_string(),
                },
                processing_mode: "AUTH_AND_CAPTURE".to_string(),
                payment_purpose: "PAYMENT".to_string(),
                payment_channel: "ECOMMERCE".to_string(),
            },
        },
        experience: Experience {
            branding_profile: branding_profile.to_string(),
        },
        return_urls: ReturnUrls {
            default: return_url.to_string(),
        },
    }
}

/// Creates a checkout session by POSTing to `{base_url}/v1/checkout-sessions`
/// with the `X-API-Key` header and a fresh `Idempotency-Key`.
pub async fn create_session(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    idempotency_key: &str,
    req: &CheckoutRequest,
) -> Result<CheckoutResponse, String> {
    let url = format!("{base_url}/v1/checkout-sessions");
    let resp = client
        .post(&url)
        .header("X-API-Key", api_key)
        .header("Idempotency-Key", idempotency_key)
        .json(req)
        .send()
        .await
        .map_err(|e| format!("checkout request to {url} failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("checkout API returned {status}: {body}"));
    }
    resp.json::<CheckoutResponse>()
        .await
        .map_err(|e| format!("decoding checkout response failed: {e}"))
}
