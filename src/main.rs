use actix_web::{web, App, HttpServer, HttpResponse};
use async_graphql::{Schema, EmptyMutation, Object, Context};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use reqwest::Client;
use serde_json::json;
use dotenv::dotenv;
use std::env;

struct Query;

#[Object]
impl Query {
    async fn get_recommendation(&self, ctx: &Context<'_>, thought_map: String) -> async_graphql::Result<String> {
        let recommendation = get_groq_recommendation(thought_map).await?;
        Ok(recommendation)
    }
}

async fn get_groq_recommendation(thought_map: String) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set");
    let client = Client::new();
    
    let response = client.post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "mixtral-8x7b-32768",
            "messages": [
                {
                    "role": "system",
                    "content": "You are an AI assistant that provides recommendations based on thought maps."
                },
                {
                    "role": "user",
                    "content": format!("Given this thought map, provide a recommendation: {}", thought_map)
                }
            ]
        }))
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    let recommendation = result["choices"][0]["message"]["content"].as_str()
        .ok_or("Failed to parse recommendation")?
        .to_string();

    Ok(recommendation)
}

type ThoughtMapSchema = Schema<Query, EmptyMutation, async_graphql::EmptySubscription>;

async fn graphql(schema: web::Data<ThoughtMapSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let schema = Schema::build(Query, EmptyMutation, async_graphql::EmptySubscription).finish();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/graphql").guard(web::post().guard(web::Header("content-type", "application/json"))).to(graphql))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}