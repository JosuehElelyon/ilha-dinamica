# Ilha Dinâmica

Uma **Dynamic Island para Windows**: uma pílula no topo da tela que se expande quando você passa o mouse por cima.

Feita com **Tauri 2** — backend em **Rust**, interface em **Vue 3 + TypeScript** — e usa o WebView2 que já vem no Windows. Consome muito menos memória que a versão antiga em Electron.

## Recursos
- **Início**: cronômetro, música tocando agora com a **capa do álbum** e controles, e Pomodoro (25 + 5 min).
- **Trabalho**: um cartão por órgão/cliente (ENAP, FNDE, BASA… dá para adicionar e remover) com:
  - **VPN com liga/desliga** — OpenVPN Connect, Cisco Secure Client/AnyConnect (conecta sozinho com a senha salva no Gerenciador de Credenciais), FortiClient, GlobalProtect, VPN do Windows, OpenVPN GUI, WireGuard ou comandos próprios. O status vem da placa de rede, então funciona com qualquer cliente; opção "Uma VPN por vez".
  - **Merge Requests do GitLab** — para revisar e os seus, com status do pipeline e conflitos. Avisa quando pedem sua revisão ou um pipeline falha. O token fica no Gerenciador de Credenciais do Windows.
  - Etiquetas na pílula mostram quais VPNs estão ligadas.
- **Ações**: seus atalhos para sites, programas, pastas e comandos (`calc`, `ms-settings:`…). Dá para criar, editar e remover.
- **Ferramentas**: JSON (formatar/validar/minificar), JWT (header, payload e validade), Base64, URL, timestamp ↔ data, UUID, SHA-256/1/512, conversão de case e contagem. No **Clips**, o que você copia é reconhecido (JWT, JSON, timestamp…) e aparece um botão para abrir direto na ferramenta.
- **Snippets**: biblioteca de trechos (Git/GitLab, kubectl, Docker, SQL, ADR, descrição de MR, C4) com busca, categorias e variáveis `{assim}` preenchidas na hora de copiar. Crie, edite e apague os seus.
- **Clips**: as últimas 8 coisas copiadas (só na memória, nunca no disco).
- **Notas**: bloco de notas que salva sozinho.
- **Timer**: contagem regressiva com aviso.
- **Agenda**: calendário com **lembretes**.
- **Clima**: agora, próximas 8 horas e 6 dias (Open-Meteo).
- **Áudio**: escolha a saída (fones, alto-falantes, monitor) e o microfone, com volume e mudo.
- **Sistema**: brilho (notebooks), CPU, RAM e bateria.
- **Limpeza**: temporários, cache dos navegadores e apps, miniaturas, relatórios de erro, Temp do Windows, Windows Update e Lixeira; otimizar memória e limpar DNS.
- **Processos**: CPU/RAM por programa, busca e finalizar (processos essenciais do Windows ficam protegidos).
- Botão **+**: ativa, desativa e reordena as abas.
- **••• → Personalizar**: cor, tamanho, opacidade, cidade do clima e avisos.
- **Avisos na pílula**: volume, microfone, fones conectados, bateria, internet, música nova, timer, foco e lembretes.
- **Arraste para mover**: segure a barra do topo da ilha aberta e solte onde quiser. A posição fica salva; perto do centro do topo ela encaixa sozinha (ímã). Embaixo ou nos cantos, ela abre para o lado certo. Duplo clique na barra, ou **••• → Voltar ao centro do topo**, volta para a posição padrão. **Travar posição** evita arrastar sem querer.
- **Atualização automática** pelas Releases do GitHub (assinadas digitalmente).
- Atalho **Ctrl + Alt + Espaço** abre/fecha a ilha. Ícone na bandeja com "Iniciar com o Windows".

## Instalar (mais fácil)
Baixe o instalador **`Ilha.Dinamica_…_x64-setup.exe`** em [Releases](https://github.com/JosuehElelyon/ilha-dinamica/releases/latest) e execute.
A partir da versão 2.1 a ilha **se atualiza sozinha**: ela verifica ao abrir e a cada 6 horas (••• → *Verificar atualizações* / *Atualizar automaticamente*).

## Publicar uma nova versão
```powershell
npm run release 2.1.1
```
Isso atualiza o número da versão e envia ao GitHub. O GitHub Actions cria a tag, compila no Windows, assina e publica a Release; as ilhas instaladas recebem a atualização.
> Requer o segredo `TAURI_SIGNING_PRIVATE_KEY` no repositório (Settings → Secrets and variables → Actions).

Se a ilha não abrir, veja o log em `%LOCALAPPDATA%\com.josueh.ilhadinamica\ilha.log`.

## Como compilar e instalar (manual)

### Pré-requisitos (uma vez só)
1. **Node.js LTS** — https://nodejs.org
2. **Rust** — https://rustup.rs (baixe e rode o `rustup-init.exe`, opção padrão)
3. **Visual Studio Build Tools** — https://visualstudio.microsoft.com/visual-cpp-build-tools/
   No instalador, marque **"Desenvolvimento para desktop com C++"** (inclui o MSVC e o Windows SDK).
4. **WebView2** — já vem no Windows 10 e 11.

### Compilar
```powershell
git clone https://github.com/JosuehElelyon/ilha-dinamica.git
cd ilha-dinamica
npm install
npm run build
```
A primeira compilação do Rust demora alguns minutos. O instalador fica em:
```
src-tauri\target\release\bundle\nsis\Ilha Dinâmica_2.0.0_x64-setup.exe
```

### Desenvolver
- `npm run dev` — abre o app com recarregamento automático.
- `npm run dev:web` — só a interface, no navegador, com dados simulados (não precisa do Rust).

## Estrutura
```
src/                 interface (Vue 3 + TypeScript)
  App.vue            ilha: expandir/recolher, tema e avisos
  components/        pílula, topo, barra de abas e uma pasta tabs/ com cada aba
  lib/api.ts         ponte com o Rust (invoke/listen) + simulador
  lib/devtools.ts    funções da caixa de ferramentas
  lib/snippets.ts    snippets padrão e variáveis
  store/             estado compartilhado e configurações
src-tauri/           backend (Rust)
  src/hover.rs       click-through da área transparente
  src/position.rs    arrastar, ímã, âncora e posição salva
  src/media.rs       mídia + capa (Windows Media Control)
  src/audio.rs       volume, microfone e dispositivos (Core Audio)
  src/system.rs      CPU, RAM, bateria, brilho e processos
  src/clean.rs       limpeza, Lixeira, memória e DNS
  src/actions.rs     ações, seletor de arquivos, notificações, admin
  src/update.rs      atualização automática
  src/vpn.rs         VPNs: placas de rede, ligar/desligar, teste de acesso
  src/gitlab.rs      Merge Requests (token no Gerenciador de Credenciais)
  src/logger.rs      log de erros (ilha.log)
.github/workflows/ compilação e publicação automática no Windows
```

## Licença
[MIT](LICENSE) © 2026 Josueh
