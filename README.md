# Rustavoid 🚫🛡️

O **Rustavoid** é uma aplicação web autohospedada (self-hosted) projetada para rodar em servidores domésticos (Homelab) via Docker Compose. O objetivo do app é ser um cadastro pessoal de "itens a evitar", prevenindo novas frustrações com marcas, produtos, empresas ou serviços específicos que entregaram uma péssima experiência no passado.

Seja um restaurante com atendimento ruim, uma marca de eletrodoméstico que quebrou rápido demais, ou um software com péssimo suporte — cadastre no **Rustavoid** e consulte antes de gastar seu dinheiro novamente.

---

## 🏗️ Stack Tecnológica (Focada em Homelab)

Desenvolvido em **Rust** para ser extremamente leve, rápido e seguro, garantindo consumo de recursos mínimo no servidor:

* **Backend:** [Axum](https://github.com/tokio-rs/axum) (Web framework moderno, rápido e assíncrono baseado em `tokio`).
* **Banco de Dados:** [SQLite](https://www.sqlite.org/) via [SQLx](https://github.com/launchbadge/sqlx) (Banco em arquivo único com migrações automáticas de esquema na inicialização e queries validadas em compilação).
* **Interface (UI):** [Askama](https://github.com/djc/askama) (Templates HTML tipo Jinja compilados de forma segura diretamente no binário Rust) + [HTMX](https://htmx.org/) (Para interações assíncronas dinâmicas com transições visuais de 350ms e zero Javascript manual) + **Vanilla CSS Moderno** (com suporte a dark mode automático).
* **Deploy:** Multi-stage **Dockerfile** gerando imagem de produção enxuta (~25MB) baseada em `debian:slim` com persistência em volume local `/app/data`.

---

## 📂 Estrutura do Projeto

```text
Rustavoid/
├── .github/workflows/
│   └── docker-image.yml     # CI/CD: Compilação automática e push no GHCR + webhook Runtipi
├── migrations/
│   └── 20260523120000_init.sql # Script automático de setup do banco SQLite
├── src/
│   ├── db.rs                # Modelos de dados e queries CRUD assíncronas do SQLx
│   └── main.rs              # Ponto de entrada, configuração do Axum, static file server e rotas
├── static/
│   └── styles.css           # Estilos visuais premium baseados em HSL e Glassmorphism
├── templates/
│   ├── base.html            # Layout base HTML com carregamento do HTMX e CSS
│   ├── index.html           # Página principal com formulário de cadastro e lista de itens
│   └── item_card.html       # Componente do card individual renderizado/deletado via HTMX
├── Cargo.toml               # Dependências e manifesto Rust do projeto
├── Dockerfile               # Pipeline multi-stage de build e runtime compacta
├── docker-compose.yml       # Setup de orquestração local com volumes de persistência
└── README.md                # Esta documentação
```

---

## 📊 Arquitetura de Dados

A entidade central é o **Item a Evitar** (`avoid_items`):

```sql
CREATE TABLE IF NOT EXISTS avoid_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,          -- Nome da marca, produto ou serviço (ex: "Marca X de TV")
    category TEXT NOT NULL,       -- Categoria (Produto, Serviço, Empresa, Restaurante, etc.)
    reason TEXT NOT NULL,         -- O motivo principal de querer evitar
    alternative TEXT,             -- Alternativa recomendada (opcional)
    severity TEXT NOT NULL,       -- Gravidade (low, medium, high)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🗺️ Progresso do Projeto

- [x] **Fase 1: Setup e Estrutura Inicial**
  - [x] Configuração completa do `Cargo.toml`.
  - [x] Estrutura de diretórios organizada seguindo as convenções do Askama e arquivos estáticos.
  - [x] Arquivos Docker e Docker Compose criados e prontos para Homelab.
- [x] **Fase 2: Banco de Dados e Conectividade**
  - [x] Setup do pool de conexões do SQLite no `src/main.rs`.
  - [x] Migrações automatizadas (`migrations/`) integradas ao startup.
  - [x] Métodos CRUD assíncronas em `src/db.rs`.
- [x] **Fase 3: Rotas Web e Interface (UI/UX)**
  - [x] Configuração dos endpoints do Axum (`GET /`, `POST /items`, `DELETE /items/:id`).
  - [x] Interface estática premium criada com CSS Moderno (Glassmorphism e Dark Mode nativo).
  - [x] Dinâmica de SPA implementada usando HTMX (Cadastro insere card imediatamente no topo da lista; Deletar executa efeito fade-out de 350ms e retira o elemento do DOM).
- [x] **Fase 4: Integração de Pipelines (CI/CD)**
  - [x] Configuração do GitHub Action `.github/workflows/docker-image.yml` para compilar e salvar imagem no **GHCR** e notificar a App Store do Runtipi.

---

## 🚀 Como Executar

### Executando Localmente (Desenvolvimento)
1. Certifique-se de possuir o compilador do Rust instalado e a carga de trabalho de C++ do Visual Studio (no Windows).
2. Execute o app:
   ```bash
   cargo run
   ```
3. Acesse `http://localhost:8080` no seu navegador. O banco de dados local `rustavoid.db` será criado automaticamente na raiz.

### Executando via Docker Compose (Homelab)
1. Com o Docker Desktop ativo, execute:
   ```bash
   docker compose up -d --build
   ```
2. O banco de dados SQLite será criado e persistido de forma segura na pasta local `./data/rustavoid.db` do seu host.

---

## 🤖 Informações Importantes para Agentes de IA

Se você é um agente de IA dando continuidade ao desenvolvimento deste projeto, atente-se às seguintes diretrizes arquiteturais adotadas:

1. **Design System Purista:** Não utilize Tailwind CSS ou dependências Node.js. Toda a identidade visual (incluindo cores HSL, variáveis nativas, responsividade, modo escuro e animações de deleção/adição) está centralizada de forma limpa em `static/styles.css`.
2. **Integração Axum e Askama:**
   - A biblioteca `askama` compila os templates HTML em tempo de compilação.
   - Os templates **obrigatoriamente** devem residir na pasta raiz `templates/` (e não dentro de `src/templates/`) para que a macro `#[derive(Template)]` funcione.
   - A importação da biblioteca `askama_axum` implementa automaticamente o trait `IntoResponse` para todas as structs que herdam `Template`.
3. **Padrão de Resposta HTMX:**
   - A rota `POST /items` recebe os dados do formulário via `Form<db::CreateItemInput>`, insere no banco, e deve retornar **apenas** a renderização de `ItemCardTemplate` (que injeta o card HTML isolado no topo da lista).
   - A rota `DELETE /items/:id` remove o registro e deve retornar apenas um status de sucesso vazio (`StatusCode::OK`). O HTMX se encarrega de ler esse retorno e remover o card ancestral (`hx-target="closest .avoid-card"`) com uma transição CSS suave de 350ms (`hx-swap="outerHTML swap:350ms"`).
4. **CI/CD Remoto:**
   - O GitHub Action utiliza o segredo `${{ secrets.GITHUB_TOKEN }}` para se autenticar automaticamente no GHCR da sua conta do GitHub.
   - O segredo `${{ secrets.DISPATCH_PAT }}` é utilizado no final do pipeline para disparar o webhook que atualiza a App Store customizada do Runtipi. Mantenha essa integração intacta.
5. **Compilação Windows:** Para executar `cargo check` ou `cargo run` no host Windows, é necessário que as ferramentas de build de C++ do Visual Studio estejam instaladas, do contrário o linker falhará.
