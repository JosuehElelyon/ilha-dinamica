// Biblioteca inicial de snippets (você pode editar, apagar e criar os seus na aba Snippets)
export interface Snippet { id: number; title: string; cat: string; body: string }

/** Variáveis no formato {nome} viram campos a preencher antes de copiar */
export const VAR_RE = /\{([A-Za-z_][\w-]*)\}/g
export const varsOf = (body: string) => [...new Set([...body.matchAll(VAR_RE)].map(m => m[1]))]
export const fill = (body: string, values: Record<string, string>) =>
  body.replace(VAR_RE, (m, k: string) => (k in values && values[k] !== '' ? values[k] : m))

let n = 0
const s = (cat: string, title: string, body: string): Snippet => ({ id: ++n, cat, title, body })

export const DEFAULT_SNIPPETS: Snippet[] = [
  // Git / GitLab
  s('Git', 'Nova branch a partir da main', 'git checkout main && git pull && git checkout -b {tipo}/{descricao}'),
  s('Git', 'Desfazer último commit (mantém mudanças)', 'git reset --soft HEAD~1'),
  s('Git', 'Atualizar branch com a main (rebase)', 'git fetch origin && git rebase origin/main'),
  s('Git', 'Push criando Merge Request no GitLab', 'git push -u origin HEAD -o merge_request.create -o merge_request.target=main -o merge_request.title="{titulo}"'),
  s('Git', 'Commit convencional', '{tipo}({escopo}): {descricao}'),
  s('Git', 'Apagar branches locais já mescladas', 'git branch --merged main | grep -v "main" | xargs git branch -d'),
  // Kubernetes
  s('K8s', 'Contexto e namespace atuais', 'kubectl config current-context && kubectl config view --minify -o jsonpath="{..namespace}"'),
  s('K8s', 'Logs do deployment (seguindo)', 'kubectl logs -f deploy/{deployment} -n {namespace} --tail=200'),
  s('K8s', 'Port-forward de serviço', 'kubectl port-forward svc/{servico} {porta_local}:{porta} -n {namespace}'),
  s('K8s', 'Reiniciar deployment', 'kubectl rollout restart deploy/{deployment} -n {namespace}'),
  s('K8s', 'Pods com problema', 'kubectl get pods -A --field-selector=status.phase!=Running'),
  // Docker
  s('Docker', 'Subir compose em segundo plano', 'docker compose up -d --build'),
  s('Docker', 'Limpar tudo que não está em uso', 'docker system prune -af --volumes'),
  s('Docker', 'Shell dentro do container', 'docker exec -it {container} sh'),
  // SQL
  s('SQL', 'PostgreSQL: maiores tabelas', "SELECT relname AS tabela, pg_size_pretty(pg_total_relation_size(relid)) AS tamanho\nFROM pg_catalog.pg_statio_user_tables\nORDER BY pg_total_relation_size(relid) DESC\nLIMIT 20;"),
  s('SQL', 'PostgreSQL: consultas lentas em execução', "SELECT pid, now() - query_start AS duracao, state, query\nFROM pg_stat_activity\nWHERE state <> 'idle' AND now() - query_start > interval '30 seconds'\nORDER BY duracao DESC;"),
  // Documentos de arquitetura
  s('Docs', 'ADR (Architecture Decision Record)', '# ADR-{numero}: {titulo}\n\n- **Status:** Proposto\n- **Data:** {data}\n- **Projeto:** {projeto}\n\n## Contexto\n\n\n## Decisão\n\n\n## Alternativas consideradas\n\n\n## Consequências\n- Positivas:\n- Negativas:\n'),
  s('Docs', 'Descrição de Merge Request', '## O que muda\n\n\n## Por quê\n\n\n## Como testar\n1. \n\n## Checklist\n- [ ] Testes\n- [ ] Documentação\n- [ ] Sem segredos no código\n\nRelacionado: #{issue}'),
  s('Docs', 'Diagrama C4 (Mermaid) – contexto', 'C4Context\n  title Contexto - {sistema}\n  Person(usuario, "Usuário")\n  System(sistema, "{sistema}")\n  System_Ext(externo, "{sistema_externo}")\n  Rel(usuario, sistema, "Usa")\n  Rel(sistema, externo, "Integra via", "HTTPS/REST")')
]
