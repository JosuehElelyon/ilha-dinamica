# Ilha Dinâmica — Dynamic Island para Windows

Uma pílula que fica no topo da tela e se expande quando você passa o mouse por cima.

## O que tem
- **Início**: cronômetro, música tocando agora **com a capa do álbum** e controles, e Pomodoro.
- **Ações**: atalhos que você cria e apaga (sites, programas, pastas, comandos como `calc` ou `ms-settings:`). Use **Editar** para remover ou alterar.
- **Clips**: as últimas 8 coisas copiadas, só na memória.
- **Notas**: bloco de notas que salva sozinho.
- **Timer**: contagem regressiva com aviso.
- **Agenda**: calendário com **lembretes** (clique no dia, escolha o horário e escreva).
- **Clima**: agora, próximas 8 horas e 6 dias (Open-Meteo, sem cadastro).
- **Áudio**: escolha a saída (fones, alto-falantes, monitor) e o microfone padrão, com volume e mudo de cada um.
- **Sistema**: brilho da tela (notebooks), CPU, RAM, bateria.
- **Limpeza**: analisa e limpa arquivos temporários, cache dos navegadores (Chrome, Edge, Brave, Opera, Firefox), cache do Discord/VS Code, miniaturas, relatórios de erro, Temp do Windows, cache do Windows Update e Lixeira. Também tem "Otimizar memória" e "Limpar cache DNS". Só apaga cache: senhas, histórico e cookies dos navegadores ficam intactos. Arquivos em uso são pulados.
- **Processos**: lista o que está rodando com CPU e RAM (agrupado por programa ou um por um), busca e botão Finalizar com confirmação. Processos essenciais do Windows ficam protegidos (🔒).
- **••• → Reiniciar como administrador**: libera a limpeza das pastas do Windows e finalizar processos de outros usuários.
- Botão **+** (canto esquerdo): ativa, desativa e reordena as abas.
- **••• → Personalizar**: cor de destaque, tamanho, opacidade, cidade do clima, cor que acompanha a música e quais avisos aparecem.
- **Avisos na pílula**: volume/mudo, microfone, fones conectados/desconectados, bateria fraca/carregando, internet caiu/voltou, música nova, timer, foco e lembretes.

## Como instalar
1. Instale o **Node.js LTS**: https://nodejs.org
2. Extraia esta pasta, abra o terminal nela e rode:
   ```
   npm install
   npm start
   ```
3. Para gerar um instalador `.exe`:
   ```
   npm run dist
   ```
   O instalador fica na pasta `dist/`.

## Atalhos e dicas
- **Ctrl + Alt + Espaço**: abre/fecha a ilha e a mantém aberta.
- Ícone na bandeja (perto do relógio): ativar "Iniciar com o Windows" ou sair.
- Menu **•••** dentro da ilha: as mesmas opções, mais "Manter aberta".
- A área transparente em volta da ilha não bloqueia cliques.

## Observações
- O brilho só funciona na tela do notebook. Monitores externos normalmente não aceitam ajuste pelo Windows.
- A troca de dispositivo padrão usa a mesma interface interna do Windows que apps como EarTrumpet usam.
- Ações e configurações ficam salvas em `%APPDATA%\Ilha Dinâmica\data.json`.

## Personalizar
- Tempos do Pomodoro: `FOCO` e `PAUSA` em `src/renderer.js`.
- Cores e tamanhos: `src/style.css` (`#island.expanded` define o tamanho aberto).
- Largura/altura da janela: `W` e `H` em `src/main.js`.
