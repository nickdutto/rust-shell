use crate::network::http_method::HttpMethod;
use ureq::Error as UreqError;
use url::Url;

pub struct HttpClient;

impl HttpClient {
    pub fn send_request(http_method: &HttpMethod, url: &Url) -> Result<String, UreqError> {
        match http_method {
            HttpMethod::Get => Self::send_get(url),
        }
    }

    fn send_get(url: &Url) -> Result<String, UreqError> {
        let response: String = ureq::get(url.to_string())
            .call()?
            .body_mut()
            .read_to_string()?;

        Ok(response)
    }
}
