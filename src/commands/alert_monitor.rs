use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use reqwest;
use serde_json::json;
use chrono::{DateTime, Utc};

// Shared state for alerts
pub struct AlertMonitor {
    alerts: Arc<Mutex<Option<HashMap<String, Vec<AlertData>>>>>,
}

#[derive(Debug, Clone)]
pub struct AlertData {
    pub price: f64,
    pub condition: String,
    pub timestamp: DateTime<Utc>,
}

impl AlertMonitor {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(Some(HashMap::new()))),
        }
    }

    pub async fn add_alert(&self, symbol: String, price: f64, condition: String) {
        let mut alerts = self.alerts.lock().await;
        if let Some(alerts_map) = alerts.as_mut() {
            let alert_data = AlertData {
                price,
                condition,
                timestamp: Utc::now(),
            };
            alerts_map.entry(symbol)
                .or_insert_with(Vec::new)
                .push(alert_data);
        }
    }

    pub async fn remove_alert(&self, symbol: &str) {
        let mut alerts = self.alerts.lock().await;
        if let Some(alerts_map) = alerts.as_mut() {
            alerts_map.remove(symbol);
        }
    }

    pub async fn get_alerts(&self) -> Vec<String> {
        let alerts = self.alerts.lock().await;
        if let Some(alerts_map) = alerts.as_ref() {
            let mut result = Vec::new();
            for (symbol, alert_list) in alerts_map {
                for alert in alert_list {
                    result.push(format!(
                        "• {} {} ${:.2} (set {})",
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
            if result.is_empty() {
                result.push("No alerts set".to_string());
            }
            result
        } else {
            vec!["No alerts set".to_string()]
        }
    }

    pub async fn start_monitoring(&self) {
        let alerts = self.alerts.clone();
        tokio::spawn(async move {
            loop {
                // Check prices every minute
                tokio::time::sleep(Duration::from_secs(60)).await;
                
                let alerts_guard = alerts.lock().await;
                if let Some(alerts_map) = alerts_guard.as_ref() {
                    for (symbol, alert_list) in alerts_map {
                        // Get current price
                        if let Ok(current_price) = get_current_price(symbol).await {
                            for alert in alert_list {
                                let should_trigger = match alert.condition.as_str() {
                                    "above" => current_price > alert.price,
                                    "below" => current_price < alert.price,
                                    _ => false,
                                };
                                
                                if should_trigger {
                                    // TODO: Send notification to user
                                    println!("Alert triggered for {}: price {} ${:.2}",
                                        symbol,
                                        alert.condition,
                                        alert.price
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

async fn get_current_price(symbol: &str) -> Result<f64, String> {
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", 
        match symbol {
            "BTC" => "bitcoin",
            "ETH" => "ethereum",
            "ICP" => "internet-computer",
            _ => return Err("Unsupported symbol".into()),
        }
    );
    
    let response = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?;
        
    let data: serde_json::Value = response.json()
        .await
        .map_err(|e| e.to_string())?;
        
    let price = data[symbol.to_lowercase()]["usd"]
        .as_f64()
        .ok_or_else(|| "Failed to parse price".to_string())?;
        
    Ok(price)
} 