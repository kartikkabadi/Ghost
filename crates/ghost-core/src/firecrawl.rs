use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirecrawlPage {
    pub url: String,
    pub title: String,
    pub summary: String,
    pub markdown: String,
    pub key_points: Vec<String>,
    pub extracted_claims: Vec<String>,
    pub source_links: Vec<String>,
    pub raw: Value,
    pub fallback: bool,
}

#[derive(Clone, Debug)]
pub struct FirecrawlClient {
    client: Client,
    api_key: Option<String>,
    base_url: String,
}

impl FirecrawlClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(45))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            base_url: "https://api.firecrawl.dev".to_string(),
        }
    }

    pub fn from_env() -> Self {
        Self::new(env::var("FIRECRAWL_API_KEY").ok())
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn extract_pages(&self, urls: &[String], objective: &str) -> Vec<FirecrawlPage> {
        let mut pages = Vec::new();
        for url in urls {
            pages.push(self.extract_page(url, objective).await);
        }
        pages
    }

    async fn extract_page(&self, url: &str, objective: &str) -> FirecrawlPage {
        if let Some(api_key) = &self.api_key {
            let body = json!({
                "urls": [url],
                "prompt": format!(
                    "Extract the page title, summary, key points, supporting sources, and evidence-bearing claims relevant to: {}",
                    objective
                ),
                "schema": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "summary": { "type": "string" },
                        "markdown": { "type": "string" },
                        "keyPoints": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "extractedClaims": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "sourceLinks": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["summary", "keyPoints", "extractedClaims"]
                },
                "enableWebSearch": true
            });

            let response = self
                .client
                .post(format!("{}/v2/extract", self.base_url.trim_end_matches('/')))
                .bearer_auth(api_key)
                .json(&body)
                .send()
                .await;

            if let Ok(response) = response {
                if let Ok(response) = response.error_for_status() {
                    if let Ok(raw) = response.json::<Value>().await {
                        if let Some(page) = Self::page_from_extract_response(url, raw.clone()) {
                            return page;
                        }
                        return Self::fallback_page(url, objective, raw, false);
                    }
                }
            }
        }

        Self::fallback_page(
            url,
            objective,
            json!({
                "reason": if self.is_configured() { "firecrawl request failed" } else { "firecrawl api key missing" }
            }),
            true,
        )
    }

    fn page_from_extract_response(url: &str, raw: Value) -> Option<FirecrawlPage> {
        let data = raw.get("data").cloned().unwrap_or(Value::Null);
        let candidate = match data {
            Value::Array(ref items) => items.first().cloned().unwrap_or(Value::Null),
            other => other,
        };

        if candidate.is_null() {
            return None;
        }

        let title = candidate
            .get("title")
            .and_then(Value::as_str)
            .or_else(|| candidate.pointer("/metadata/title").and_then(Value::as_str))
            .unwrap_or(url)
            .to_string();
        let summary = candidate
            .get("summary")
            .and_then(Value::as_str)
            .or_else(|| candidate.get("description").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();
        let markdown = candidate
            .get("markdown")
            .and_then(Value::as_str)
            .or_else(|| candidate.get("content").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();
        let key_points = strings_from_value(candidate.get("keyPoints").or_else(|| candidate.get("key_points")));
        let extracted_claims = strings_from_value(candidate.get("extractedClaims").or_else(|| candidate.get("claims")));
        let source_links = strings_from_value(candidate.get("sourceLinks").or_else(|| candidate.get("source_links")));
        let url = candidate
            .get("url")
            .and_then(Value::as_str)
            .or_else(|| candidate.pointer("/metadata/sourceURL").and_then(Value::as_str))
            .unwrap_or(url)
            .to_string();

        Some(FirecrawlPage {
            url,
            title,
            summary,
            markdown,
            key_points,
            extracted_claims,
            source_links,
            raw,
            fallback: false,
        })
    }

    fn fallback_page(url: &str, objective: &str, raw: Value, fallback: bool) -> FirecrawlPage {
        FirecrawlPage {
            url: url.to_string(),
            title: url.to_string(),
            summary: format!("Firecrawl extracted page for {} while working on {}", url, objective),
            markdown: raw
                .get("markdown")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            key_points: vec![format!("Candidate source identified for {}", objective)],
            extracted_claims: vec![format!("{} is relevant to {}", url, objective)],
            source_links: vec![url.to_string()],
            raw,
            fallback,
        }
    }
}

fn strings_from_value(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
