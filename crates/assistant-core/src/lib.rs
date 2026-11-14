//! Núcleo hexagonal (Domain & Ports) do assistente pessoal desktop da Ilha Dinâmica.
//! Mantém persistência local-first em SQLite, normalização canônica de contas,
//! protocolo unificado da ponte com celular e motor proativo de notificações.

pub mod bridge;
pub mod db;
pub mod domain;
pub mod normalizer;
pub mod proactive;
