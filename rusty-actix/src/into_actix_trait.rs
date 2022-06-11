use actix_web::error as actix_error;
use actix_web::http::StatusCode;
use phonebook::Err as AppErr;

pub trait IntoActixResult<T> {
    fn actix_result(self) -> core::result::Result<T, actix_web::Error>;
}

impl<T> IntoActixResult<T> for anyhow::Result<T, anyhow::Error> {
    fn actix_result(self) -> core::result::Result<T, actix_web::Error> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => match err.downcast() {
                Ok(AppErr::Io(inner)) => {
                    Err(actix_error::InternalError::new(inner, StatusCode::INTERNAL_SERVER_ERROR).into())
                }
                Ok(AppErr::Json(inner)) => Err(actix_error::ErrorInternalServerError(inner)),
                // Ok(AppErr::PhonebookEntry(inner)) => Err(actix_error::ErrorBadRequest(inner)),
                Ok(AppErr::PhonebookEntry(inner)) => Err(actix_error::ErrorBadRequest(inner)),
                _ => Err(
                    actix_error::InternalError::new("Something went wrong", StatusCode::INTERNAL_SERVER_ERROR).into(),
                ),
            },
        }
    }
}
