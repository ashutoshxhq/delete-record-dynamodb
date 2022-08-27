use std::collections::HashMap;

use aws_sdk_dynamodb::Client;
use egnitely_client::{HandlerError, RequestContext, Result};
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_0_17::to_item;
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
struct FunctionContextData {
    pub table_name: String,
    pub primary_key: String,
    pub index_data: HashMap<String, String>,
    pub token_claims: Value,
}

pub async fn handler(mut _ctx: RequestContext, _input: Value) -> Result<Value> {
    let context_data = serde_json::from_value::<FunctionContextData>(_ctx.data())?;
    let input_data = serde_json::from_value::<HashMap<String, Value>>(_input)?;
    if let Some(sdk_config) = _ctx.aws_sdk_config() {
        let client = Client::new(&sdk_config);
        client
            .delete_item()
            .table_name(context_data.table_name)
            .set_key(Some(to_item(input_data)?))
            .send()
            .await?;
        Ok(json!({
                "message": "Successfully deleted record"
        }))
    } else {
        return Err(Box::new(HandlerError::new(
            "NO_SDK_CONFIG".to_string(),
            "No aws sdk config found in handler context".to_string(),
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_dynamodb::Credentials;

    #[tokio::test]
    async fn trigger_function() {
        let config = aws_config::from_env()
            .credentials_provider(Credentials::new(
                "PUT_ACCESS_TOKEN".to_string(),
                "PUT_ACCESS_SECRET".to_string(),
                None,
                None,
                "local",
            ))
            .region("ap-south-1")
            .load()
            .await;

        let resp = handler(
            RequestContext::new(
                "test".to_string(),
                "test".to_string(),
                Some(config),
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
            json!({
                "id": "a6c18e06-aa03-45ea-9e9e-6d9328746951",
            }),
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
