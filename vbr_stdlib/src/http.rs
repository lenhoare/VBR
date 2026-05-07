use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

pub struct Http;
pub struct HttpResponse(reqwest::blocking::Response);

impl Http {

    /// Simple GET request returning body as String
    /// VBA equivalent: WinHTTP.WinHttpRequest GET
    pub fn get(url: &str) -> Result<String, String> {
        reqwest::blocking::get(url)
            .map_err(|e| e.to_string())?
            .text()
            .map_err(|e| e.to_string())
    }

    /// GET request with custom headers
    pub fn get_with_headers(
        url: &str,
        headers: HashMap<String, String>
    ) -> Result<String, String> {
        let client = Client::new();
        let mut header_map = HeaderMap::new();
        for (key, value) in &headers {
            header_map.insert(
                HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| e.to_string())?,
                HeaderValue::from_str(value)
                    .map_err(|e| e.to_string())?
            );
        }
        client.get(url)
            .headers(header_map)
            .send()
            .map_err(|e| e.to_string())?
            .text()
            .map_err(|e| e.to_string())
    }

    /// POST request with a string body
    /// VBA equivalent: WinHTTP.WinHttpRequest POST
    pub fn post(url: &str, body: &str) -> Result<String, String> {
        Client::new()
            .post(url)
            .body(body.to_string())
            .send()
            .map_err(|e| e.to_string())?
            .text()
            .map_err(|e| e.to_string())
    }

    /// POST request with a JSON body
    pub fn post_json(
        url: &str,
        body: &serde_json::Value
    ) -> Result<String, String> {
        Client::new()
            .post(url)
            .json(body)
            .send()
            .map_err(|e| e.to_string())?
            .text()
            .map_err(|e| e.to_string())
    }

    /// GET request returning full response object
    /// Use when you need status code or headers
    pub fn get_response(url: &str) -> Result<HttpResponse, String> {
        Client::new()
            .get(url)
            .send()
            .map_err(|e| e.to_string())
            .map(HttpResponse)
    }

    /// Create an empty headers HashMap
    pub fn headers() -> HashMap<String, String> {
        HashMap::new()
    }
}

impl HttpResponse {

    /// Get the HTTP status code
    pub fn status(&self) -> u16 {
        self.0.status().as_u16()
    }

    /// Get the response body as String
    pub fn text(self) -> Result<String, String> {
        self.0.text()
            .map_err(|e| e.to_string())
    }

    /// Get a response header value
    pub fn header(&self, key: &str) -> Result<String, String> {
        self.0.headers()
            .get(key)
            .ok_or_else(|| format!("Header '{}' not found", key))?
            .to_str()
            .map_err(|e| e.to_string())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let response = Http::get("https://httpbin.org/get").unwrap();
        assert!(response.contains("url"));
    }

    #[test]
    fn test_post() {
        let response = Http::post(
            "https://httpbin.org/post",
            "test body"
        ).unwrap();
        assert!(response.contains("test body"));
    }

    #[test]
    fn test_get_response_status() {
        let response = Http::get_response("https://httpbin.org/get").unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_headers() {
        let mut headers = Http::headers();
        headers.insert("Authorization".to_string(), "Bearer test".to_string());
        let response = Http::get_with_headers(
            "https://httpbin.org/headers",
            headers
        ).unwrap();
        assert!(response.contains("Bearer test"));
    }
}
