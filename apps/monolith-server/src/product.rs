use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::{ok, err, AppState};
use crate::jwt::verify_token;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub daily_price: f64,
    pub deposit_amount: Option<f64>,
    pub insurance_required: Option<bool>,
    pub specifications: Option<serde_json::Value>,
    pub address: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub daily_price: Option<f64>,
    pub deposit_amount: Option<f64>,
    pub insurance_required: Option<bool>,
    pub specifications: Option<serde_json::Value>,
    pub address: Option<serde_json::Value>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductResponse {
    pub product_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub daily_price: f64,
    pub deposit_amount: Option<f64>,
    pub insurance_required: bool,
    pub specifications: Option<serde_json::Value>,
    pub address: Option<serde_json::Value>,
    pub avg_rating: Option<f64>,
    pub total_reviews: i32,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductFilters {
    pub category_id: Option<Uuid>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryResponse {
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

fn extract_user_id_from_token(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    if claims.token_type != "access" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match Uuid::parse_str(&claims.sub) {
        Ok(id) => Ok(id),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn create_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateProductRequest>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_token(&headers) {
        Ok(id) => id,
        Err(status) => return err(status, "Unauthorized"),
    };

    // Verify category exists
    let category = sqlx::query!(
        "SELECT category_id FROM product_schema.categories WHERE category_id = $1",
        req.category_id
    )
    .fetch_optional(&state.db)
    .await;

    if category.is_err() || category.unwrap().is_none() {
        return err(StatusCode::BAD_REQUEST, "Invalid category_id");
    }

    // Create product
    let product_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"
        INSERT INTO product_schema.products 
        (product_id, owner_id, category_id, name, description, daily_price, deposit_amount, 
         insurance_required, specifications, address)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        product_id,
        user_id,
        req.category_id,
        req.name,
        req.description,
        req.daily_price,
        req.deposit_amount,
        req.insurance_required.unwrap_or(false),
        req.specifications,
        req.address
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create product");
    }

    // Add tags if provided
    if let Some(tags) = req.tags {
        for tag_name in tags {
            // Get or create tag
            let tag_id = sqlx::query!(
                "INSERT INTO product_schema.tags (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING tag_id",
                tag_name
            )
            .fetch_one(&state.db)
            .await;

            if let Ok(tag) = tag_id {
                // Link tag to product
                let _ = sqlx::query!(
                    "INSERT INTO product_schema.product_tags (product_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    product_id,
                    tag.tag_id
                )
                .execute(&state.db)
                .await;
            }
        }
    }

    let response = serde_json::json!({
        "product_id": product_id,
        "message": "Product created successfully"
    });

    ok(response)
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let product = sqlx::query!(
        r#"
        SELECT p.*, 
               array_agg(DISTINCT t.name) FILTER (WHERE t.name IS NOT NULL) as tags,
               array_agg(DISTINCT pi.image_url) FILTER (WHERE pi.image_url IS NOT NULL) as images
        FROM product_schema.products p
        LEFT JOIN product_schema.product_tags pt ON p.product_id = pt.product_id
        LEFT JOIN product_schema.tags t ON pt.tag_id = t.tag_id
        LEFT JOIN product_schema.product_images pi ON p.product_id = pi.product_id
        WHERE p.product_id = $1
        GROUP BY p.product_id
        "#,
        product_id
    )
    .fetch_one(&state.db)
    .await;

    let product = match product {
        Ok(p) => p,
        Err(_) => return err(StatusCode::NOT_FOUND, "Product not found"),
    };

    let response = ProductResponse {
        product_id: product.product_id,
        owner_id: product.owner_id,
        name: product.name,
        description: product.description,
        category_id: product.category_id,
        daily_price: product.daily_price,
        deposit_amount: product.deposit_amount,
        insurance_required: product.insurance_required,
        specifications: product.specifications,
        address: product.address,
        avg_rating: product.avg_rating,
        total_reviews: product.total_reviews,
        status: product.status,
        created_at: product.created_at,
        tags: product.tags.unwrap_or_default(),
        images: product.images.unwrap_or_default(),
    };

    ok(response)
}

pub async fn list_products(
    State(state): State<AppState>,
    Query(filters): Query<ProductFilters>,
) -> impl IntoResponse {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT p.*, 
               array_agg(DISTINCT t.name) FILTER (WHERE t.name IS NOT NULL) as tags,
               array_agg(DISTINCT pi.image_url) FILTER (WHERE pi.image_url IS NOT NULL) as images
        FROM product_schema.products p
        LEFT JOIN product_schema.product_tags pt ON p.product_id = pt.product_id
        LEFT JOIN product_schema.tags t ON pt.tag_id = t.tag_id
        LEFT JOIN product_schema.product_images pi ON p.product_id = pi.product_id
        WHERE p.status = 'active'
        "#
    );

    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    if let Some(category_id) = filters.category_id {
        param_count += 1;
        conditions.push(format!("p.category_id = ${}", param_count));
        params.push(Box::new(category_id));
    }

    if let Some(min_price) = filters.min_price {
        param_count += 1;
        conditions.push(format!("p.daily_price >= ${}", param_count));
        params.push(Box::new(min_price));
    }

    if let Some(max_price) = filters.max_price {
        param_count += 1;
        conditions.push(format!("p.daily_price <= ${}", param_count));
        params.push(Box::new(max_price));
    }

    if let Some(search) = filters.search {
        param_count += 1;
        conditions.push(format!("(p.name ILIKE ${} OR p.description ILIKE ${})", param_count, param_count));
        params.push(Box::new(format!("%{}%", search)));
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" GROUP BY p.product_id ORDER BY p.created_at DESC LIMIT ");
    query.push_str(&per_page.to_string());
    query.push_str(" OFFSET ");
    query.push_str(&offset.to_string());

    // For now, use a simpler approach without dynamic SQL
    let products = sqlx::query!(
        r#"
        SELECT p.*, 
               array_agg(DISTINCT t.name) FILTER (WHERE t.name IS NOT NULL) as tags,
               array_agg(DISTINCT pi.image_url) FILTER (WHERE pi.image_url IS NOT NULL) as images
        FROM product_schema.products p
        LEFT JOIN product_schema.product_tags pt ON p.product_id = pt.product_id
        LEFT JOIN product_schema.tags t ON pt.tag_id = t.tag_id
        LEFT JOIN product_schema.product_images pi ON p.product_id = pi.product_id
        WHERE p.status = 'active'
        GROUP BY p.product_id
        ORDER BY p.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        per_page,
        offset
    )
    .fetch_all(&state.db)
    .await;

    let products = match products {
        Ok(p) => p,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch products"),
    };

    let total = sqlx::query!(
        "SELECT COUNT(*) as count FROM product_schema.products WHERE status = 'active'"
    )
    .fetch_one(&state.db)
    .await;

    let total = match total {
        Ok(t) => t.count.unwrap_or(0),
        Err(_) => 0,
    };

    let product_responses: Vec<ProductResponse> = products
        .into_iter()
        .map(|p| ProductResponse {
            product_id: p.product_id,
            owner_id: p.owner_id,
            name: p.name,
            description: p.description,
            category_id: p.category_id,
            daily_price: p.daily_price,
            deposit_amount: p.deposit_amount,
            insurance_required: p.insurance_required,
            specifications: p.specifications,
            address: p.address,
            avg_rating: p.avg_rating,
            total_reviews: p.total_reviews,
            status: p.status,
            created_at: p.created_at,
            tags: p.tags.unwrap_or_default(),
            images: p.images.unwrap_or_default(),
        })
        .collect();

    let response = ProductListResponse {
        products: product_responses,
        total,
        page,
        per_page,
    };

    ok(response)
}

pub async fn update_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(product_id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_token(&headers) {
        Ok(id) => id,
        Err(status) => return err(status, "Unauthorized"),
    };

    // Verify product ownership
    let product = sqlx::query!(
        "SELECT owner_id FROM product_schema.products WHERE product_id = $1",
        product_id
    )
    .fetch_one(&state.db)
    .await;

    let product = match product {
        Ok(p) => p,
        Err(_) => return err(StatusCode::NOT_FOUND, "Product not found"),
    };

    if product.owner_id != user_id {
        return err(StatusCode::FORBIDDEN, "Not authorized to update this product");
    }

    // Update product
    let result = sqlx::query!(
        r#"
        UPDATE product_schema.products 
        SET 
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            category_id = COALESCE($4, category_id),
            daily_price = COALESCE($5, daily_price),
            deposit_amount = COALESCE($6, deposit_amount),
            insurance_required = COALESCE($7, insurance_required),
            specifications = COALESCE($8, specifications),
            address = COALESCE($9, address),
            status = COALESCE($10, status),
            updated_at = NOW()
        WHERE product_id = $1
        "#,
        product_id,
        req.name,
        req.description,
        req.category_id,
        req.daily_price,
        req.deposit_amount,
        req.insurance_required,
        req.specifications,
        req.address,
        req.status
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update product");
    }

    ok(serde_json::json!({ "message": "Product updated successfully" }))
}

pub async fn delete_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_token(&headers) {
        Ok(id) => id,
        Err(status) => return err(status, "Unauthorized"),
    };

    // Verify product ownership
    let product = sqlx::query!(
        "SELECT owner_id FROM product_schema.products WHERE product_id = $1",
        product_id
    )
    .fetch_one(&state.db)
    .await;

    let product = match product {
        Ok(p) => p,
        Err(_) => return err(StatusCode::NOT_FOUND, "Product not found"),
    };

    if product.owner_id != user_id {
        return err(StatusCode::FORBIDDEN, "Not authorized to delete this product");
    }

    // Soft delete by setting status to 'deleted'
    let result = sqlx::query!(
        "UPDATE product_schema.products SET status = 'deleted', updated_at = NOW() WHERE product_id = $1",
        product_id
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete product");
    }

    ok(serde_json::json!({ "message": "Product deleted successfully" }))
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(req): Json<CreateCategoryRequest>,
) -> impl IntoResponse {
    let category_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"
        INSERT INTO product_schema.categories (category_id, name, description, parent_category_id)
        VALUES ($1, $2, $3, $4)
        "#,
        category_id,
        req.name,
        req.description,
        req.parent_category_id
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create category");
    }

    let response = serde_json::json!({
        "category_id": category_id,
        "message": "Category created successfully"
    });

    ok(response)
}

pub async fn list_categories(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let categories = sqlx::query!(
        r#"
        SELECT category_id, name, description, parent_category_id, created_at
        FROM product_schema.categories
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await;

    let categories = match categories {
        Ok(c) => c,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch categories"),
    };

    let category_responses: Vec<CategoryResponse> = categories
        .into_iter()
        .map(|c| CategoryResponse {
            category_id: c.category_id,
            name: c.name,
            description: c.description,
            parent_category_id: c.parent_category_id,
            created_at: c.created_at,
        })
        .collect();

    ok(category_responses)
}
