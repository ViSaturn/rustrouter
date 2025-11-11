use reqwest::{Client, header};
use eyre::{Result, eyre};
use std::collections::HashMap;

use superstruct::superstruct;
use serde::{Serialize, Deserialize};
use jsonpath_lib as jsonpath;

const API_URL: &'static str = "https://openrouter.ai/api/v1/chat/completions";

fn clean_string(s: &str, c: char) -> String {
    s.trim_matches(c) // remove c from start and end
	.chars()
	.filter(|ch| !ch.is_control()) // remove control chars
	.collect()
}

#[superstruct(variants(Full, Simple), variant_attributes(derive(Clone, Serialize, Deserialize)))]
pub struct OpenRouterParams {
    #[superstruct(only(Full))]
    pub model: String,

    #[superstruct(only(Full))]
    pub messages: serde_json::Value,

    // TODO: top p, and top k, and any other params.

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<serde_json::Value>,
}

pub struct OpenRouterResponse {
    pub response_map: HashMap<String, serde_json::Value>,
}

impl OpenRouterResponse {
    pub fn new(response_map: HashMap<String, serde_json::Value>) -> Self {
	dbg!(&response_map);
	Self { response_map }
    }

    // TODO: output error or warning if finding batch responses or chat history.
    /// Fast way to get the only response of the AI. Useful for simple calls.
    /// NOTE: Only retrieves the last response in the chat history, and expects only one response (not batches).
    pub fn get_response(&self) -> Result<String> {
	let results = jsonpath::select(&self.response_map["choices"], "$[0].message.content");
	results?
	    .into_iter().last().ok_or_else(|| eyre!("Expected at least one result for LLM call."))
	    .map(|value| clean_string(&value.to_string(), '"').replace("\\\"", "\""))
    }
}

impl OpenRouterParamsFull {
    fn build(model: String, messages: serde_json::Value) -> Self {
	Self {
	    model,
	    messages,
	    temperature: None,
	    response_schema: None,
	}
    }

    fn temperature(mut self, temperature: f32) -> Self {
	self.temperature = Some(temperature);
	self
    }
    fn response_schema(mut self, response_schema: serde_json::Value) -> Self {
	self.response_schema = Some(response_schema);
	self
    }
}

pub struct OpenRouter {
    pub api_key: String,
    pub reqwest_client: Client,
}
impl OpenRouter {
    pub fn new(api_key: String) -> Self {
	Self {
	    api_key,
	    reqwest_client: Client::new(),
	}
    }

    pub async fn complex_call(
	&self,
	openrouter_params: OpenRouterParamsFull,
    ) -> Result<OpenRouterResponse> {
	let res: HashMap<String, serde_json::Value> = self.reqwest_client
	    .post(API_URL)
	    .header(header::CONTENT_TYPE, "application/json")
	    .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
	    .json(&serde_json::to_value(&openrouter_params)?)
	    .send().await?
	    .json().await?;
	Ok(OpenRouterResponse::new(res))
    }

    pub async fn call(
	&self,
	model: String, prompt: String,
	openrouter_params: Option<OpenRouterParamsSimple>,
    ) -> Result<OpenRouterResponse> {
	let res = self.complex_call(OpenRouterParamsFull {
	    model: model,
	    messages: serde_json::json!([
		{
		    "role": "user",
		    "content": [
			{
			    "type": "text",
			    "text": prompt
			}
		    ]
		}
	    ]),
	    temperature: openrouter_params.clone().map(|o| o.temperature).flatten(),
	    response_schema: openrouter_params.clone().map(|o| o.response_schema).flatten(),
	});

	res.await
    }

    // NOTE/TODO: this only returns the first response right now, which should be fine under usual circumstances
}

// TODO: add tests
