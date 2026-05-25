# Rustavoid

Aplicação web self-hosted para registrar produtos, serviços e empresas que tiveram experiência ruim, evitando recompras futuras.

## Stack
- Rust + Axum
- Askama (templates HTML server-side)
- SQLite + SQLx migrations
- HTMX para interações assíncronas sem frontend JS customizado
- Docker Compose para homelab

## Estrutura
- `src/main.rs`: rotas, servidor e bootstrap
- `src/db.rs`: acesso a dados (CRUD de itens)
- `migrations/`: schema SQLite
- `templates/`: páginas e componentes Askama
- `static/styles.css`: UI

## Executar localmente
```bash
cargo run
```
Acesse `http://localhost:8080`.

## Executar no homelab (Docker)
```bash
docker compose up -d --build
```
Banco persistido em `./data/rustavoid.db`.

## Variáveis de ambiente
- `PORT` (default `8080`)
- `DATABASE_URL` (default `sqlite://rustavoid.db?mode=rwc`)
- `RUST_LOG` (default `rustavoid=info,tower_http=info`)
