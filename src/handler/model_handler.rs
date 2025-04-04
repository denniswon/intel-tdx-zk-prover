use crate::dto::model_dto::ModelReadDto;
use crate::entity::model::Model;
use crate::response::api_response::ApiSuccessResponse;
use axum::{Extension, Json};

pub async fn model(
    Extension(model): Extension<Model>,
) -> Json<ApiSuccessResponse<ModelReadDto>> {
    Json(ApiSuccessResponse::send(ModelReadDto::from(model)))
}
