use async_trait::async_trait;
use chrono::Utc;
use oc_bots_sdk::api::command::{CommandDefinition, CommandHandler, CommandResponse};
use oc_bots_sdk::api::definition::CommandDefinitionParameter;
use oc_bots_sdk::api::definition::CommandDefinitionParameterType;
use oc_bots_sdk::api::definition::CommandDefinitionParameters;
use oc_bots_sdk::api::definition::CommandDefinitionResponse;
use oc_bots_sdk::api::definition::CommandDefinitionResponseType;
use oc_bots_sdk::api::definition::CommandDefinitionResponses;
use oc_bots_sdk::api::definition::CommandDefinitionScope;
use oc_bots_sdk::api::definition::CommandDefinitionSummary;
use oc_bots_sdk::api::definition::CommandDefinitionUsage;
use oc_bots_sdk::api::definition::CommandDefinitionUsageExample;
use oc_bots_sdk::api::definition::CommandDefinitionUsageExamples;
use oc_bots_sdk::api::definition::CommandDefinitionUsageParameter;
use oc_bots_sdk::api::definition::CommandDefinitionUsageParameters;
use oc_bots_sdk::api::definition::CommandDefinitionUsageResponse;
use oc_bots_sdk::api::definition::CommandDefinitionUsageResponses;
use oc_bots_sdk::api::definition::CommandDefinitionUsageScope;
use oc_bots_sdk::api::definition::CommandDefinitionUsageSummary;
use oc_bots_sdk::api::definition::CommandDefinitionUsageType;
use oc_bots_sdk::api::definition::CommandDefinitionUsageTypes;
use oc_bots_sdk::api::definition::CommandDefinitionUsageValue;
use oc_bots_sdk::api::definition::CommandDefinitionUsageValues;
use oc_bots_sdk::api::definition::CommandDefinitionValue;
use oc_bots_sdk::api::definition::CommandDefinitionValues;
use oc_bots_sdk::api::definition::CommandDefinitionVersion;
use oc_bots_sdk::api::definition::CommandDefinitionVersions;
use oc_bots_sdk::api::definition::CommandDefinitionVersionsType;
use oc_bots_sdk::api::definition::CommandDefinitionVersionsTypes;
use oc_bots_sdk::oc_api::client::ClientFactory;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Price;

#[derive(Debug, Serialize, Deserialize)]
struct PriceResponse {
    price: String,
    timestamp: i64,
}

#[async_trait]
impl CommandHandler for Price {
    fn definition(&self) -> CommandDefinition {
        CommandDefinition {
            name: "price".to_string(),
            description: "Get the current price of ICP".to_string(),
            scope: CommandDefinitionScope::Public,
            parameters: CommandDefinitionParameters(vec![]),
            responses: CommandDefinitionResponses(vec![
                CommandDefinitionResponse {
                    title: "Current ICP Price".to_string(),
                    response_type: CommandDefinitionResponseType::Message,
                },
            ]),
        }
    }

    async fn execute(
        &self,
        _params: Vec<String>,
        client_factory: Arc<ClientFactory>,
        _now: u64,
    ) -> CommandResponse {
        // Fetch ICP price from CoinGecko API
        let client = reqwest::Client::new();
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=internet-computer&vs_currencies=usd";
        
        match client.get(url).send().await {
            Ok(response) => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if let Some(price) = data["internet-computer"]["usd"].as_f64() {
                            let price_decimal = Decimal::from_f64(price).unwrap_or_default();
                            let response = PriceResponse {
                                price: format!("${:.2}", price_decimal),
                                timestamp: Utc::now().timestamp(),
                            };
                            
                            CommandResponse::Success(serde_json::json!({
                                "text": format!("Current ICP Price: {}\nLast updated: <t:{}:R>", 
                                    response.price, response.timestamp)
                            }))
                        } else {
                            CommandResponse::InternalError("Failed to parse price data".to_string())
                        }
                    }
                    Err(e) => CommandResponse::InternalError(format!("Failed to parse response: {}", e)),
                }
            }
            Err(e) => CommandResponse::InternalError(format!("Failed to fetch price: {}", e)),
        }
    }
} 