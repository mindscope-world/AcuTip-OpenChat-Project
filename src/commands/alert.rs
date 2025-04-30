use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::types::BotCommandContext;
use oc_bots_sdk_offchain::AgentRuntime;
use oc_bots_sdk::oc_api::client::Client;
use std::sync::LazyLock;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Alert::definition);

#[derive(Clone)]
pub struct Alert {
    alerts: Arc<Mutex<HashMap<String, Vec<AlertData>>>>,
}

#[derive(Debug, Clone)]
struct AlertData {
    price: f64,
    condition: String,
    timestamp: DateTime<Utc>,
}

impl Alert {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "alert".to_string(),
            description: Some("Set and manage price alerts".to_string()),
            placeholder: Some("Managing alerts...".to_string()),
            params: vec![
                BotCommandParam {
                    name: "action".to_string(),
                    description: Some("Choose alert action".to_string()),
                    placeholder: Some("Select an action".to_string()),
                    required: true,
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 10,
                        choices: vec![
                            BotCommandOptionChoice {
                                name: "Set Alert".to_string(),
                                value: "set".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "List Alerts".to_string(),
                                value: "list".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "Remove Alert".to_string(),
                                value: "remove".to_string(),
                            },
                        ],
                        multi_line: false,
                    }),
                },
                BotCommandParam {
                    name: "symbol".to_string(),
                    description: Some("Cryptocurrency symbol".to_string()),
                    placeholder: Some("Select a cryptocurrency".to_string()),
                    required: true,
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 10,
                        choices: vec![
                            BotCommandOptionChoice {
                                name: "Bitcoin (BTC)".to_string(),
                                value: "BTC".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "Ethereum (ETH)".to_string(),
                                value: "ETH".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "Internet Computer (ICP)".to_string(),
                                value: "ICP".to_string(),
                            },
                        ],
                        multi_line: false,
                    }),
                },
                BotCommandParam {
                    name: "price".to_string(),
                    description: Some("Alert price in USD".to_string()),
                    placeholder: Some("Enter price (e.g. 50000)".to_string()),
                    required: true,
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: Vec::new(),
                        multi_line: false,
                    }),
                },
                BotCommandParam {
                    name: "condition".to_string(),
                    description: Some("Alert condition".to_string()),
                    placeholder: Some("Select a condition".to_string()),
                    required: true,
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 10,
                        choices: vec![
                            BotCommandOptionChoice {
                                name: "Price Above".to_string(),
                                value: "above".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "Price Below".to_string(),
                                value: "below".to_string(),
                            },
                        ],
                        multi_line: false,
                    }),
                },
            ],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}

#[async_trait]
impl CommandHandler<AgentRuntime> for Alert {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        client: Client<AgentRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let action = client.context().command.arg::<String>("action").to_lowercase();
        
        match action.as_str() {
            "set" => {
                let symbol = client.context().command.arg::<String>("symbol").to_uppercase();
                let price_str = client.context().command.arg::<String>("price");
                let price = price_str.parse::<f64>().map_err(|_| "Invalid price format")?;
                let condition = client.context().command.arg::<String>("condition").to_lowercase();
                
                let alert_data = AlertData {
                    price,
                    condition: condition.clone(),
                    timestamp: Utc::now(),
                };
                
                let mut alerts = self.alerts.lock().unwrap();
                alerts.entry(symbol.clone())
                    .or_insert_with(Vec::new)
                    .push(alert_data);
                
                let message = format!(
                    "🔔 Price alert set for {} when price {} ${:.2}",
                    symbol,
                    match condition.as_str() {
                        "above" => "rises above",
                        "below" => "falls below",
                        _ => return Err("Invalid condition".into()),
                    },
                    price
                );
                
                let response = client
                    .send_text_message(message)
                    .execute_then_return_message(|_, _| ());
                Ok(SuccessResult { message: response })
            },
            "list" => {
                let mut message = "📋 Your Price Alerts:\n\n".to_string();
                let alerts = self.alerts.lock().unwrap();
                
                if alerts.is_empty() {
                    message.push_str("No alerts set");
                } else {
                    for (symbol, alert_list) in alerts.iter() {
                        for alert in alert_list {
                            message.push_str(&format!(
                                "• {} {} ${:.2} (set {})\n",
                                symbol,
                                match alert.condition.as_str() {
                                    "above" => "above",
                                    "below" => "below",
                                    _ => "unknown",
                                },
                                alert.price,
                                alert.timestamp.format("%Y-%m-%d %H:%M")
                            ));
                        }
                    }
                }
                
                let response = client
                    .send_text_message(message)
                    .execute_then_return_message(|_, _| ());
                Ok(SuccessResult { message: response })
            },
            "remove" => {
                let symbol = client.context().command.arg::<String>("symbol").to_uppercase();
                let mut alerts = self.alerts.lock().unwrap();
                alerts.remove(&symbol);
                let message = format!("✅ Removed price alert for {}", symbol);
                let response = client
                    .send_text_message(message)
                    .execute_then_return_message(|_, _| ());
                Ok(SuccessResult { message: response })
            },
            _ => Err("Invalid action".into()),
        }
    }
} 