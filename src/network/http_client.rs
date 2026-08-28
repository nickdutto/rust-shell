use crate::engine::engine_state::EngineState;
use crate::error::shell_error::ShellError;
use crate::network::http_method::HttpMethod;
use crate::value::span::Span;
use std::sync::mpsc;
use std::time::Duration;
use url::Url;

pub struct HttpClient;

impl HttpClient {
    pub fn send_request(
        http_method: &HttpMethod,
        url: &Url,
        engine_state: &EngineState,
        span: Span,
    ) -> Result<ureq::http::Response<ureq::Body>, ShellError> {
        match http_method {
            HttpMethod::Get => Self::send_get(url, engine_state, span),
        }
    }

    pub fn send_get(
        url: &Url,
        engine_state: &EngineState,
        span: Span,
    ) -> Result<ureq::http::Response<ureq::Body>, ShellError> {
        let (tx, rx) = mpsc::channel();
        let url = url.to_string();

        std::thread::spawn(move || {
            let res = ureq::get(&url).call();
            _ = tx.send(res);
        });

        loop {
            if engine_state.signals.is_interrupted() {
                return Err(ShellError::Interrupted { span });
            }

            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => return Err(ShellError::Ureq(e)),

                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(ShellError::Generic(
                        "HTTP GET Request thread disconnected unexpectedly".to_owned(),
                    ));
                }
            }
        }
    }
}
