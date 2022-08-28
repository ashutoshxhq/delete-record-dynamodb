use std::collections::HashMap;

use aws_sdk_dynamodb::Client;
use egnitely_client::{Context, Error};
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_0_17::to_item;
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
struct FunctionConfigData {
    pub table_name: String,
    pub primary_key: String,
    pub index_data: HashMap<String, String>,
    pub token_claims: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionInput {
    pub filter: HashMap<String, Value>,
}

pub async fn handler(mut _ctx: Context, _input: FunctionInput) -> Result<Value, Error> {
    let config_data = serde_json::from_value::<FunctionConfigData>(_ctx.config())?;
    let config = aws_config::from_env().region("ap-south-1").load().await;
    let client = Client::new(&config);
    client
        .delete_item()
        .table_name(config_data.table_name)
        .set_key(Some(to_item(_input.filter)?))
        .send()
        .await?;

    Ok(json!({
            "message": "Successfully deleted record"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn trigger_function() {
        let resp = handler(
            Context::new(
                "test".to_string(),
                "test".to_string(),
                json!({
                    "table_name": "functions",
                    "primary_key": "id",
                    "index_data": {
                        "team_id": "team_id-index"
                    },
                    "token_claims": {}
                }),
                json!({}),
            ),
            FunctionInput {
                filter: HashMap::from([("id".to_string(), json!("84203adc-def0-4ed1-9984-e92156cfe6b4"))]),
            },
        )
        .await;

        match resp {
            Ok(res) => {
                println!("{}", res);
                assert_eq!("Successfully deleted record", res["message"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        };
    }
}
