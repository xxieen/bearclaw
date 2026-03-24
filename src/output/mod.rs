use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl<T: Serialize> Response<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            count: None,
            error: None,
            code: None,
        }
    }

    pub fn success_with_count(data: T, count: usize) -> Self {
        Self {
            ok: true,
            data: Some(data),
            count: Some(count),
            error: None,
            code: None,
        }
    }
}

impl Response<()> {
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            ok: false,
            data: None,
            count: None,
            error: Some(message.to_string()),
            code: Some(code.to_string()),
        }
    }

    pub fn ok_empty() -> Self {
        Self {
            ok: true,
            data: None,
            count: None,
            error: None,
            code: None,
        }
    }
}

pub fn print_json<T: Serialize>(response: &Response<T>, pretty: bool) {
    let output = if pretty {
        serde_json::to_string_pretty(response)
    } else {
        serde_json::to_string(response)
    };
    match output {
        Ok(json) => println!("{json}"),
        Err(e) => {
            let err = Response::<()>::error("SERIALIZE_ERROR", &e.to_string());
            if let Ok(json) = serde_json::to_string(&err) {
                println!("{json}");
            }
        }
    }
}
