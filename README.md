# Rustavoid 🚫🛡️

O **Rustavoid** é uma aplicação web autohospedada (self-hosted) projetada para rodar em servidores domésticos (Homelab) via Docker Compose. O objetivo do app é ser um cadastro pessoal de "itens a evitar", prevenindo novas frustrações com marcas, produtos, empresas ou serviços específicos que entregaram uma péssima experiência no passado.

Seja um restaurante com atendimento ruim, uma marca de eletrodoméstico que quebrou rápido demais, ou um software com péssimo suporte — cadastre no **Rustavoid** e consulte antes de gastar seu dinheiro novamente.

---

## 🏗️ Stack Tecnológica (Focada em Homelab)

Desenvolvido em **Rust** para ser extremamente leve, rápido e seguro, garantindo consumo de recursos mínimo no servidor:

* **Backend:** [Axum](https://github.com/tokio-rs/axum) (Web framework moderno, rápido e assíncrono baseado em `tokio`).
* **Banco de Dados:** [SQLite](https://www.sqlite.org/) via [SQLx](https://github.com/launchbadge/sqlx) (Banco leve em arquivo único, com validação de queries SQL em tempo de compilação).
* **Interface (UI):** [Askama](https://github.com/djc/askama) (Templates HTML tipo Jinja compilados diretamente no binário Rust) + [HTMX](https://htmx.org/) (Para interações SPA modernas sem a complexidade de frameworks JS pesados) + **Vanilla CSS**.
* **Deploy:** Multi-stage **Dockerfile** gerando uma imagem de produção ultra-compacta (~25MB) baseada em Alpine.

---

## 📊 Arquitetura de Dados (Schema Proposto)

A entidade central é o **Item a Evitar** (`avoid_items`):

```sql
CREATE TABLE IF NOT EXISTS avoid_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,          -- Nome da marca, produto ou serviço (ex: "Marca X de TV")
    category TEXT NOT NULL,       -- Categoria (Produto, Serviço, Empresa, Restaurante, etc.)
    reason TEXT NOT NULL,         -- O motivo principal de querer evitar
    alternative TEXT,             -- Alternativa recomendada (opcional, ex: "Usar marca Y")
    severity TEXT NOT NULL,       -- Gravidade (Baixa, Média, Alta - ex: "Nunca mais compre" ou "Evite se puder")
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🗺️ Roteiro de Implementação (Roadmap)

Este roteiro serve como guia de progresso para desenvolvedores e agentes de IA continuarem a implementação.

### Fase 1: Setup e Estrutura Inicial ⏳ (Aguardando Aprovação do Plano)
- [ ] Inicialização do projeto Cargo (`cargo init`).
- [ ] Configuração do arquivo `Cargo.toml` com dependências de Web, Banco e Template.
- [ ] Configuração do `Dockerfile` multi-stage e `docker-compose.yml` para desenvolvimento/produção local.
- [ ] Setup do logging estruturado (`tracing`).

### Fase 2: Banco de Dados e Conectividade 📅
- [ ] Integração com `SQLx` e setup do pool de conexões do SQLite.
- [ ] Criação de migrações automáticas de banco de dados (`migrations/`).
- [ ] Implementação de queries de inserção, leitura e deleção de itens.

### Fase 3: Rotas Web e UI Básica 📅
- [ ] Configuração do roteador do `Axum` e tratamento de erros.
- [ ] Criação dos templates HTML com `Askama` (Home, Cadastro, Lista).
- [ ] Estilização visual premium com Vanilla CSS moderno (modo escuro automático, design limpo e focado em leitura rápida).
- [ ] Integração do `HTMX` para atualizar a lista de itens ou deletar sem recarregar a página inteira.

---

## 🚀 Como Executar Localmente (Futuro)

### Pré-requisitos
* Rust (instalado via `rustup`)
* Docker e Docker Compose (para rodar em produção/homelab)

### Executando em Desenvolvimento
1. Execute as migrações e rode o app:
   ```bash
   cargo run
   ```
2. Acesse `http://localhost:8080` no seu navegador.

### Executando via Docker Compose
1. Suba o container com:
   ```bash
   docker compose up -d --build
   ```
2. O banco de dados SQLite será persistido localmente na pasta vinculada no volume.
