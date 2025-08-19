use crate::database::{
    graphql::{mutation::MutationRoot, query::QueryRoot},
    structures::Permission
};

use async_graphql::{
    EmptySubscription,
    http::GraphiQLSource,
    Schema
    
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    response::{self, IntoResponse},
};
use std::sync::Arc;

pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/data").finish())
}

pub async fn graphql_handler(schema: State<Arc<Schema<QueryRoot, MutationRoot, EmptySubscription>>>, Permission(permission): Permission, req: GraphQLRequest) -> GraphQLResponse {
    // Share the permission level with the request
    let mut request = req.into_inner();
    request = request.data(permission.clone());
    
    schema.execute(request).await.into()
}
