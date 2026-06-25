/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for handling foreign exchange currency conversions.
*/

use std::collections::HashMap;

use crate::models::Amount;

#[derive(Clone)]
pub struct ForeignExchange {
    cache: RatesCache,
    api_key: String,
}

#[derive(Clone)]
struct RatesCache {
    data: std::sync::Arc<std::sync::Mutex<Option<std::collections::HashMap<String, f32>>>>,
    last_update: std::sync::Arc<std::sync::Mutex<chrono::DateTime<chrono::Utc>>>,
}

const BASE_CURRENCY: &str = "XAF";

impl ForeignExchange {
    pub async fn convert(
        &self,
        from: Amount,
        to: String,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let to = to.to_uppercase();

        match self.cache.ensure_freshness(&self.api_key).await {
            Result::Err(e) => {
                let err: Box<dyn FXError> = Box::new(e);
                return Result::Err(err);
            }
            Result::Ok(_) => {}
        };

        let rates = self.cache.data.clone();
        let rates = rates.lock().unwrap();
        let rates = rates.clone();

        if let Option::None = rates {
            return Result::Err(Box::new(NoCurrencyData {}));
        }

        fn get_rate_for(
            currency: &String,
            rates: &HashMap<String, f32>,
        ) -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
            Result::Ok(*match rates.get(currency) {
                Option::None => {
                    let err = InvalidCurrency {};

                    let err: Box<dyn FXError> = Box::new(err);
                    return Result::Err(err);
                }
                Option::Some(v) => v,
            })
        }

        let rates = rates.unwrap();

        let from_rate = get_rate_for(&from.currency, &rates)?;
        let to_rate = get_rate_for(&to, &rates)?;

        let value = ((from.value as f32) / (from_rate / to_rate)) as i32;

        Result::Ok(value)
    }

    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: RatesCache {
                data: std::sync::Arc::new(std::sync::Mutex::new(Option::None)),
                last_update: std::sync::Arc::new(std::sync::Mutex::new(
                    chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap(),
                )),
            },
        }
    }
}

impl RatesCache {
    /// This method will fetch new currency rates if necessary.
    pub async fn ensure_freshness(&self, api_key: &String) -> Result<(), APIError> {
        const EXPIRY: chrono::TimeDelta = chrono::Duration::hours(1);

        // However, if the process is taking too long, and we already have some data, let's use it.
        let last_update = {
            let guard = self.last_update.lock().unwrap();
            *guard
        };

        if (last_update + EXPIRY) > chrono::Utc::now() {
            // No need to fetch again. It's still fresh.
            return Result::Ok(());
        }
        // Now, if the cache is not fresh,let's update it
        let data_ref = std::sync::Arc::clone(&self.data);
        let last_update_ref = std::sync::Arc::clone(&self.last_update);

        let api_key = api_key.clone();
        let future = tokio::spawn(async move {
            let values = RatesCache::fetch_new(&api_key).await?;
            let mut local_ref = data_ref.lock().unwrap();
            *local_ref = Option::Some(values);
            let mut last_update_ref = last_update_ref.lock().unwrap();
            *last_update_ref = chrono::Utc::now();
            Result::<(), Box<dyn std::error::Error + Send + Sync>>::Ok(())
        });

        // However, if the process is taking too long, and we already have some data, let's use it.
        let has_some = {
            let guard = self.data.lock().unwrap();
            guard.is_some()
        };

        if has_some {
            match tokio::time::timeout(tokio::time::Duration::from_secs(2), future).await {
                Result::Ok(_) => {}
                Result::Err(e) => {
                    println!("Could not fetch fresh rates\n{e}\n");
                }
            };
        } else {
            // But, if we have no choice, let's wait.
            match future.await.unwrap() {
                Result::Ok(_) => {}
                Result::Err(e) => {
                    print!("Unknown error\n{e}");
                    return Result::Err(APIError::ConnectionFailed {});
                }
            };
        }

        Result::Ok(())
    }

    /// This method directly fetches new data for the cache, without any questions asked.
    async fn fetch_new(
        api_key: &String,
    ) -> Result<std::collections::HashMap<String, f32>, Box<dyn std::error::Error + Send + Sync>>
    {
        let raw = reqwest::Client::new()
            .get(format!(
                "https://v6.exchangerate-api.com/v6/{api_key}/latest/{BASE_CURRENCY}"
            ))
            .send()
            .await
            .map_err(|_| APIError::ConnectionFailed {})?
            .json::<serde_json::Value>()
            .await
            .map_err(|_| APIError::InvalidResponse {})?;
        let raw = raw.get("conversion_rates");

        let raw = match raw {
            Option::None => {
                return Result::Err(Box::new(APIError::ConnectionFailed {}));
            }
            Option::Some(v) => v,
        };

        Result::Ok(
            serde_json::from_value::<std::collections::HashMap<String, f32>>(raw.clone())
                .map_err(|_| APIError::InvalidResponse {})?,
        )
    }
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid response from currency conversion API.")
    }
}

#[derive(std::fmt::Debug)]
enum APIError {
    InvalidResponse,
    ConnectionFailed,
}

unsafe impl Send for APIError {}
unsafe impl Sync for APIError {}

#[derive(std::fmt::Debug)]
struct InvalidCurrency {}
pub trait FXError: std::error::Error + Send + Sync {}

impl FXError for APIError {}

impl FXError for InvalidCurrency {}

impl std::fmt::Display for InvalidCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The currency is invalid")
    }
    //
}

impl std::error::Error for InvalidCurrency {
    //
}

impl std::error::Error for APIError {}

#[derive(std::fmt::Debug)]
struct NoCurrencyData {}

impl FXError for NoCurrencyData {}

impl std::fmt::Display for NoCurrencyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The system doesn't have any currency data.")
    }
    //
}

impl std::error::Error for NoCurrencyData {}
