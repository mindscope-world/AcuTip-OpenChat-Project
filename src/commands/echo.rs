use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::types::BotCommandContext;
use oc_bots_sdk_offchain::AgentRuntime;
use oc_bots_sdk::oc_api::client::Client;
use std::sync::LazyLock;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Echo::definition);

pub struct Echo;

#[async_trait]
impl CommandHandler<AgentRuntime> for Echo {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        client: Client<AgentRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let text = client.context().command.arg::<String>("message").to_string();

        let message = client
            .send_text_message(text)
            .with_block_level_markdown(true)
            .execute_then_return_message(|_, _| ());

        Ok(SuccessResult { message })
    }
}

impl Echo {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "echo".to_string(),
            description: Some("A simple echo bot that repeats your messages".to_string()),
            placeholder: Some("Echoing your message...".to_string()),
            params: vec![BotCommandParam {
                name: "message".to_string(),
                description: Some("The message to echo back".to_string()),
                placeholder: Some("Type your message here".to_string()),
                required: true,
                param_type: BotCommandParamType::StringParam(StringParam {
                    min_length: 1,
                    max_length: 10000,
                    choices: Vec::new(),
                    multi_line: true,
                }),
            }],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
} 