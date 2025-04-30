use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::types::BotCommandContext;
use oc_bots_sdk_offchain::AgentRuntime;
use oc_bots_sdk::oc_api::client::Client;
use std::sync::LazyLock;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(DexMonitor::definition);

pub struct DexMonitor;

#[async_trait]
impl CommandHandler<AgentRuntime> for DexMonitor {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        client: Client<AgentRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let action = client.context().command.arg::<String>("action").to_lowercase();
        
        match action.as_str() {
            "whale" => {
                // Get whale movements from DEX
                let whale_data = get_whale_movements().await?;
                let message = format!(
                    "🐋 Whale Alert!\n\n{}\n\nLast updated: <t:{}:R>",
                    whale_data.message,
                    whale_data.timestamp
                );
                let response = client
                    .send_text_message(message)
                    .execute_then_return_message(|_, _| ());
                Ok(SuccessResult { message: response })
            },
            "liquidity" => {
                // Get liquidity data
                let liquidity_data = get_liquidity_data().await?;
                let message = format!(
                    "💧 Liquidity Update\n\n{}\n\nLast updated: <t:{}:R>",
                    liquidity_data.message,
                    liquidity_data.timestamp
                );
                let response = client
                    .send_text_message(message)
                    .execute_then_return_message(|_, _| ());
                Ok(SuccessResult { message: response })
            },
            _ => Err("Invalid action".into()),
        }
    }
}

impl DexMonitor {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "dex".to_string(),
            description: Some("Monitor DEX whale movements and liquidity".to_string()),
            placeholder: Some("Fetching DEX data...".to_string()),
            params: vec![BotCommandParam {
                name: "action".to_string(),
                description: Some("Choose what to monitor".to_string()),
                placeholder: Some("Select whale or liquidity".to_string()),
                required: true,
                param_type: BotCommandParamType::StringParam(StringParam {
                    min_length: 1,
                    max_length: 20,
                    choices: vec![
                        BotCommandOptionChoice {
                            name: "Whale Movements".to_string(),
                            value: "whale".to_string(),
                        },
                        BotCommandOptionChoice {
                            name: "Liquidity Tracking".to_string(),
                            value: "liquidity".to_string(),
                        },
                    ],
                    multi_line: false,
                }),
            }],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}

#[derive(Debug)]
struct DexData {
    message: String,
    timestamp: i64,
}

async fn get_whale_movements() -> Result<DexData, String> {
    // TODO: Implement actual DEX API calls
    // For now, return mock data
    Ok(DexData {
        message: "Recent large transactions:\n\
                 • 1000 ICP moved on Sonic DEX\n\
                 • 500 ETH swapped on Uniswap\n\
                 • 10 BTC transferred to exchange".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

async fn get_liquidity_data() -> Result<DexData, String> {
    // TODO: Implement actual DEX API calls
    // For now, return mock data
    Ok(DexData {
        message: "Current DEX Liquidity:\n\
                 • Sonic DEX: 1M ICP\n\
                 • Uniswap: 500K ETH\n\
                 • PancakeSwap: 100K BNB".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
} 