-- Criação da tabela de itens a evitar
CREATE TABLE IF NOT EXISTS avoid_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,          -- Nome da marca, produto ou serviço (ex: "Marca X de TV")
    category TEXT NOT NULL,       -- Categoria (Produto, Serviço, Empresa, Restaurante, etc.)
    reason TEXT NOT NULL,         -- O motivo principal de querer evitar
    alternative TEXT,             -- Alternativa recomendada (opcional, ex: "Usar marca Y")
    severity TEXT NOT NULL,       -- Gravidade (Baixa, Média, Alta - ex: "Nunca mais compre" ou "Evite se puder")
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
