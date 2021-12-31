use validator::ValidationError;

pub fn service_kind_oneof(value: &str) -> Result<(), ValidationError> {
    let supported_services = vec!["grpc", "http", "pubsub"];

    if supported_services.iter().any(|&e| e == value) {
        return Ok(());
    }

    Err(ValidationError::new("value is not supported"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    pub fn test_service_kind_oneof() {
        #[derive(Validate)]
        struct Example {
            #[validate(custom(function = "service_kind_oneof"))]
            kind: String,
        }

        let e = Example {
            kind: "consumer".to_string(),
        };
        println!("{:?}", e.validate());
    }
}
