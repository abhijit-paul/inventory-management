use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Debug)]
pub enum PaymentErr {
    AwsRegionNotSet,
    DeleteFieldsMissing,
    InventoryDetailsNotFound,
    InventoryNotFound,
    NoPreviousPayment,
    RusotoDynamodbError,
}

impl warp::reject::Reject for PaymentErr {}

impl std::error::Error for PaymentErr {}

impl std::fmt::Display for PaymentErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PaymentErr::AwsRegionNotSet => write!(f, "AWS region not set"),
            PaymentErr::DeleteFieldsMissing => write!(
                f,
                "One or more of the fields are missing: entity_type, views, expiry"
            ),
            PaymentErr::InventoryDetailsNotFound => write!(f, "Payment details not found"),
            PaymentErr::InventoryNotFound => write!(f, "Payment detail not found"),
            PaymentErr::NoPreviousPayment => write!(f, "No previous payment details found"),
            PaymentErr::RusotoDynamodbError => write!(f, "Rusoto DynamoDB Error"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorMessage {
    code: i8,
    status_code: u16,
    message: String,
}

pub async fn customize_error(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(err) = err.find::<PaymentErr>() {
        let (code, status_code) = match err {
            PaymentErr::DeleteFieldsMissing => (-4, StatusCode::BAD_REQUEST),
            PaymentErr::InventoryDetailsNotFound => (-2, StatusCode::BAD_REQUEST),
            PaymentErr::NoPreviousPayment => (-4, StatusCode::NOT_FOUND),
            _ => (-6, StatusCode::INTERNAL_SERVER_ERROR),
        };
        let message = err.to_string();

        let json = warp::reply::json(&ErrorMessage {
            code,
            message,
            status_code: status_code.as_u16(),
        });
        Ok(warp::reply::with_status(json, status_code))
    } else {
        let code = -5;
        let message = "Internal Server Error".to_string();
        let status_code = StatusCode::INTERNAL_SERVER_ERROR;
        let err_msg = ErrorMessage {
            code,
            message,
            status_code: status_code.as_u16(),
        };
        let json = warp::reply::json(&err_msg);
        Ok(warp::reply::with_status(json, status_code))
    }
}
