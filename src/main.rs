use askama::Template;
use axum::{
    extract::{Form, Path, Query, State},
    http::HeaderMap,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
    Router,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    items: Vec<db::AvoidItem>,
    categories: Vec<String>,
}

#[derive(Template)]
#[template(path = "item_card.html")]
struct ItemCardTemplate {
    item: db::AvoidItem,
}

#[derive(Template)]
#[template(path = "items_list.html")]
struct ItemsListTemplate {
    items: Vec<db::AvoidItem>,
}

#[derive(serde::Deserialize)]
struct SearchParams {
    q: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustavoid=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Iniciando Rustavoid...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://rustavoid.db?mode=rwc".to_string());

    info!("Conectando ao banco SQLite: {}", database_url);
    let pool = SqlitePool::connect(&database_url).await?;
    db::init(&pool).await?;

    info!("Executando migrações...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrações concluídas.");

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/items/search", get(search_items_handler))
        .route("/items", post(create_item_handler))
        .route("/items/:id", delete(delete_item_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Rustavoid escutando em http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn search_items_handler(
    State(pool): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();
    match db::search_items_by_title(&pool, &query).await {
        Ok(items) => ItemsListTemplate { items }.into_response(),
        Err(err) => {
            error!("Erro ao buscar itens por nome: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao buscar itens.").into_response()
        }
    }
}

async fn index_handler(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let items = db::list_items(&pool).await;
    let categories = db::list_categories(&pool).await;

    match (items, categories) {
        (Ok(items), Ok(categories)) => IndexTemplate { items, categories }.into_response(),
        (Err(err), _) | (_, Err(err)) => {
            error!("Erro ao buscar itens: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao carregar os itens.").into_response()
        }
    }
}

async fn create_item_handler(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Form(input): Form<db::CreateItemInput>,
) -> impl IntoResponse {
    let new_item = match db::NewAvoidItem::try_from(input) {
        Ok(value) => value,
        Err(msg) => return (StatusCode::BAD_REQUEST, msg).into_response(),
    };

    let is_htmx = headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    match db::insert_item(&pool, new_item).await {
        Ok(item) => {
            info!("Novo item cadastrado: {}", item.title);
            if is_htmx {
                ItemCardTemplate { item }.into_response()
            } else {
                Redirect::to("/").into_response()
            }
        }
        Err(err) => {
            error!("Erro ao cadastrar item: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro interno ao cadastrar item: {}", err),
            )
                .into_response()
        }
    }
}

async fn delete_item_handler(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match db::remove_item(&pool, id).await {
        Ok(true) => {
            info!("Item excluído com sucesso: ID {}", id);
            StatusCode::OK.into_response()
        }
        Ok(false) => (StatusCode::NOT_FOUND, "Item nao encontrado.").into_response(),
        Err(err) => {
            error!("Erro ao excluir item ID {}: {:?}", id, err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao deletar o item.").into_response()
        }
    }
}
