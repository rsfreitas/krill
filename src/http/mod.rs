
use crate::grpc::rpc;

/// Gives the default settings for a HTTP service.
pub(crate) fn config(port: i64, name: &str) -> figment::Figment  {
    figment::Figment::from(rocket::Config::default())
        .merge(("log_level", rocket::config::LogLevel::Off))
        .merge(("port", port))
        .merge(("ident", name))
}

pub fn response<S: serde::Serialize>(res: &S) -> rocket::response::content::Json<String> {
    rocket::response::content::Json(serde_json::to_string(res).unwrap())
}

pub fn response_from_rpc<S: serde::Serialize>(res: rpc::Response<S>) -> rocket::response::content::Json<String> {
    let response = match res {
        Ok(r) => match serde_json::to_string(&r.into_inner()) {
            Ok(r) => r,
            Err(_) => "TODO SERIALIZE ERROR".to_string()
        },
        Err(e) => e.to_string()
    };

    rocket::response::content::Json(response)
}

