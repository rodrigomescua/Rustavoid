use askama::Template;
use axum::{
    extract::{Path, State, Form},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;

// Template para a página principal (index.html)
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    items: Vec<db::AvoidItem>,
}

// Template para um único card de item (renderizado dinamicamente pelo HTMX)
#[derive(Template)]
#[template(path = "item_card.html")]
struct ItemCardTemplate {
    item: db::AvoidItem,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inicializa o Logging Estruturado
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustavoid=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Iniciando Rustavoid...");

    // 2. Configura a URL de conexão com o banco SQLite
    // Se DATABASE_URL não for informada no ambiente, usa o arquivo local rustavoid.db
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://rustavoid.db?mode=rwc".to_string());

    info!("Conectando ao banco de dados SQLite: {}", database_url);
    let pool = SqlitePool::connect(&database_url).await?;

    // 3. Roda migrações de banco de dados automaticamente na inicialização
    info!("Executando migrações de banco de dados...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    info!("Migrações concluídas com sucesso!");

    // 4. Configura as rotas do Axum
    let app = Router::new()
        .route("/categories", get(categories_handler))
        .route("/categories-manage", get(categories_manage_ui_handler))
        .route("/categories", post(create_category_handler))
        .route("/categories/:name", delete(delete_category_handler))
        .route("/categories/:name", put(rename_category_handler))
        // Rota principal: exibe a lista completa e o formulário
        .route("/", get(index_handler))
        // Rotas de manipulação (integração do HTMX)
        .route("/items", axum::routing::post(create_item_handler))
        .route("/items/:id", delete(delete_item_handler))
        // Serve a pasta de arquivos estáticos (static/) sob a rota /static/
        .nest_service("/static", ServeDir::new("static"))
        // Compartilha o pool de conexões do SQLite com as rotas
        .with_state(pool);

    // 5. Define a porta e inicia o servidor HTTP
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

/// Handler principal: Renderiza a página Home com a lista de itens.
async fn index_handler(State(pool): State<SqlitePool>) -> impl IntoResponse {
    match db::get_all_items(&pool).await {
        Ok(items) => IndexTemplate { items }.into_response(),
        Err(err) => {
            error!("Erro ao buscar itens: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao carregar os itens.").into_response()
        }
    }
}

/// Handler de cadastro (POST /items): Insere o item e retorna APENAS o card recém-criado em HTML.
/// O HTMX adiciona este card no topo da lista sem recarregar a página.
async fn create_item_handler(
    State(pool): State<SqlitePool>,
    Form(input): Form<db::CreateItemInput>,
) -> impl IntoResponse {
    match db::create_item(&pool, input).await {
        Ok(item) => {
            info!("Novo item cadastrado: {}", item.title);
            ItemCardTemplate { item }.into_response()
        }
        Err(err) => {
            error!("Erro ao cadastrar item: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro interno ao cadastrar item.").into_response()
        }
    }
}


/// Handler que devolve a lista de categorias como JSON
async fn categories_handler(State(pool): State<SqlitePool>) -> impl IntoResponse {
    match db::get_all_categories(&pool).await {
        Ok(categories) => (StatusCode::OK, Json(categories)).into_response(),
        Err(err) => {
            error!("Erro ao buscar categorias: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao carregar categorias.").into_response()
        }
    }
}

/// Handler para criar uma nova categoria (recebe JSON { "category": "Nome" })
async fn create_category_handler(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateCategoryPayload>,
) -> impl IntoResponse {
    let category = payload.category.trim();
    if category.is_empty() {
        return (StatusCode::BAD_REQUEST, "Categoria vazia").into_response();
    }
    // Verifica se já existe
    let exists: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT category FROM avoid_items WHERE category = ? LIMIT 1"
    )
    .bind(category)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);
    
    if exists.is_some() {
        return (StatusCode::CONFLICT, "Categoria já existe").into_response();
    }
    (StatusCode::CREATED, Json(Vec::<String>::new())).into_response()
}

#[derive(serde::Deserialize)]
struct CreateCategoryPayload {
    category: String,
}

/// Handler para excluir uma categoria (deleta todos os itens associados)
async fn delete_category_handler(
    State(pool): State<SqlitePool>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match db::delete_items_by_category(&pool, &name).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            error!("Erro ao excluir categoria: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao excluir categoria").into_response()
        }
    }
}

/// Handler para renomear uma categoria (JSON { "new_name": "..." })
async fn rename_category_handler(
    State(pool): State<SqlitePool>,
    Path(old_name): Path<String>,
    Json(payload): Json<RenameCategoryPayload>,
) -> impl IntoResponse {
    match db::rename_category(&pool, &old_name, &payload.new_name).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            error!("Erro ao renomear categoria: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao renomear categoria").into_response()
        }
    }
}

#[derive(serde::Deserialize)]
struct RenameCategoryPayload {
    new_name: String,
}

/// Handler de exclusão (DELETE /items/:id): Exclui o item e retorna 200 OK vazio.
/// O HTMX remove a linha (card) do DOM por receber uma resposta de sucesso vazia.
async fn delete_item_handler(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match db::delete_item(&pool, id).await {
        Ok(_) => {
            info!("Item excluído com sucesso: ID {}", id);
            StatusCode::OK.into_response()
        }
        Err(err) => {
            error!("Erro ao excluir item ID {}: {:?}", id, err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao deletar o item.").into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "categories_manage.html")]
struct CategoriesManageTemplate {
    categories: Vec<String>,
}

async fn categories_manage_ui_handler(State(pool): State<SqlitePool>) -> impl IntoResponse {
    match db::get_all_categories(&pool).await {
        Ok(categories) => {
            info!("Categorias carregadas: {:?}", categories);
            CategoriesManageTemplate { categories }.into_response()
        },
        Err(err) => {
            error!("Erro ao carregar painel de categorias: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao carregar painel").into_response()
        }
    }
}

