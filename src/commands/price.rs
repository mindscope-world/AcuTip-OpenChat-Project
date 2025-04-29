use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::types::BotCommandContext;
use oc_bots_sdk_offchain::AgentRuntime;
use oc_bots_sdk::oc_api::client::Client;
use std::sync::LazyLock;
use reqwest;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Price::definition);

pub struct Price;

#[async_trait]
impl CommandHandler<AgentRuntime> for Price {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        client: Client<AgentRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let symbol = client.context().command.arg::<String>("symbol").to_uppercase();
        
        // Use CoinGecko API to get price data
        let http_client = reqwest::Client::new();
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            match symbol.as_str() {
                "BTC" => "bitcoin",
                "ETH" => "ethereum",
                "ICP" => "internet-computer",
                _ => return Err("Unsupported cryptocurrency symbol".into()),
            }
        );

        match http_client.get(&url).send().await {
            Ok(response) => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let price = match symbol.as_str() {
                            "BTC" => data["bitcoin"]["usd"].as_f64(),
                            "ETH" => data["ethereum"]["usd"].as_f64(),
                            "ICP" => data["internet-computer"]["usd"].as_f64(),
                            _ => None,
                        };

                        match price {
                            Some(p) => {
                                let message = client
                                    .send_text_message(format!("💰 Current {} price: ${:.2}", symbol, p))
                                    .execute_then_return_message(|_, _| ());
                                Ok(SuccessResult { message })
                            },
                            None => Err("Failed to parse price data".into()),
                        }
                    }
                    Err(_) => Err("Failed to parse API response".into()),
                }
            }
            Err(_) => Err("Failed to fetch price data".into()),
        }
    }
}

impl Price {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "price".to_string(),
            description: Some("Get current price for a cryptocurrency".to_string()),
            placeholder: Some("Fetching price...".to_string()),
            params: vec![BotCommandParam {
                name: "symbol".to_string(),
                description: Some("The cryptocurrency symbol".to_string()),
                placeholder: Some("Enter BTC, ETH, or ICP".to_string()),
                required: true,
                param_type: BotCommandParamType::StringParam(StringParam {
                    min_length: 1,
                    max_length: 3,
                    choices: Vec::new(),
                    multi_line: false,
                }),
            }],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
} 