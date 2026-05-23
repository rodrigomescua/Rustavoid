use askama::Template;
use axum::{
    extract::{Path, State, Form},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
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
