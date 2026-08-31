# Rubik's Cube 3D (Simulador)

Um simulador de Cubo Mágico 3D em **Rust** utilizando o motor de jogos **[Bevy](https://bevyengine.org)** (v0.13), com cores sólidas e vívidas (unlit), animações suaves, controles por toque/mouse inteligentes, botões virtuais na tela e suporte completo a teclado.

---

## Download (Executáveis Prontos)

Não é necessário instalar Rust ou compilar código. Basta baixar o executável pronto para o seu sistema:

| Sistema Operacional | Download Direto | Como Rodar |
| :--- | :--- | :--- |
| **Android** | [Baixar .apk](https://github.com/vitorpaiv4/rubiks_sim/releases/latest/download/rubiks_sim-android.apk) | Baixe no celular e instale o `.apk` |
| **Windows** | [Baixar .exe](https://github.com/vitorpaiv4/rubiks_sim/releases/latest/download/rubiks_sim-windows-x86_64.exe) · [(.zip)](https://github.com/vitorpaiv4/rubiks_sim/releases/latest/download/rubiks_sim-windows-x86_64.zip) | Baixe e dê 2 cliques no `.exe` |
| **Linux** | [Baixar .AppImage](https://github.com/vitorpaiv4/rubiks_sim/releases/latest/download/rubiks_sim-linux-x86_64.AppImage) · [(.tar.gz)](https://github.com/vitorpaiv4/rubiks_sim/releases/latest/download/rubiks_sim-linux-x86_64.tar.gz) | Conceda permissão de execução (`chmod +x`) e dê 2 cliques |
| **macOS** | [Apple Silicon (M1/M2/M3)](https://github.com/vitorpaiv4/rubiks_sim/releases/latest/download/rubiks_sim-macos-apple-silicon.zip) · [Intel](https://github.com/vitorpaiv4/rubiks_sim/releases/latest/download/rubiks_sim-macos-intel.zip) | Descompacte e execute o binário |

> Todas as versões e notas de atualização estão disponíveis na [Página de Releases](https://github.com/vitorpaiv4/rubiks_sim/releases).

---

## Jogabilidade & Controles

O simulador suporta múltiplos estilos de entrada para máxima comodidade:

### 1. Interação Direta com Mouse & Touch (Super Intuitivo)
* **Clique & Arraste na Peça do Cubo**: Puxe qualquer face ou camada do cubo na direção desejada para rotacioná-la.
* **Clique & Arraste no Fundo da Tela**: Orbita a câmera livremente ao redor do cubo.
* **Botão do Meio do Mouse (Arraste)** / **2 Dedos (Touch)**: Mover/Pan a posição do cubo.
* **Scroll do Mouse** / **Pinch-to-Zoom (Touch)**: Zoom in / Zoom out.

### 2. Interface na Tela (Botões Virtuais)
* **Barra de Ações Rápidas**:
  * `[ Scramble ]`: Sorteia e executa uma sequência oficial de scramble WCA.
  * `[ Reset ]`: Retorna instantaneamente o cubo ao estado resolvido.
  * `[ Desfazer ]`: Desfaz o último movimento executado.
  * `[ Auto Cam ]`: Alterna o giro automático contínuo da câmera.
  * `[ Botoes: Visiveis / Ocultos ]`: Alterna a exibição da barra inferior de rotação de faces.
* **Barra de Faces (Opcional / Toggle)**: Botões compactos (`U`, `D`, `R`, `L`, `F`, `B`) com seletor `Inverso: ON/OFF`.

### 3. Atalhos de Teclado (Speedcubing)
* `U, D, R, L, F, B`: Gira a respectiva face no sentido horário.
* `Shift` + `[Tecla]`: Gira a face no sentido inverso (`'`).
* `S`: Embaralhar (Scramble WCA).
* `Z` ou `Ctrl+Z`: Desfazer (Undo).
* `X`: Resetar o cubo.
* `Tab` ou `H`: Alternar exibição dos botões de face na tela.
* `Espaço`: Alternar auto-rotação da câmera.
* `Esc` ou `Q`: Fechar o simulador.

---

## Recursos de Speedcubing & HUD

* **Fila de Movimentos Assíncrona**: Permite digitar sequências e algoritmos rápidos sem perda de inputs.
* **WCA Scramble Generator**: Gera permutações válidas sem repetições redundantes e exibe a fórmula na tela.
* **Histórico de Movimentos & Undo**: Permite desfazer jogadas para corrigir erros de treino.
* **Cronômetro & Contador de Movimentos**: Inicia no primeiro giro após o scramble e registra cada movimento.
* **Detecção Automática de Vitória**: Trava o tempo automaticamente ao concluir a resolução.

---

## Arquitetura Modular

O projeto é 100% Rust e estruturado em plugins modulares do Bevy ECS:

* [`src/main.rs`](src/main.rs): Ponto de entrada, configuração da janela, iluminação e orquestração de plugins.
* [`src/cube.rs`](src/cube.rs):
  * Estrutura e hierarquia dos 27 cubinhos (`Cubie`) e seus adesivos coloridos.
  * Gerenciamento de rotações discretas por camadas lógicas inteiras (`IVec3`) para eliminar *floating-point drift*.
  * Gerador de scramble WCA, fila de comandos (`MoveQueue`) e histórico de desfazer (`MoveHistory`).
* [`src/interaction.rs`](src/interaction.rs):
  * Sistema de raycasting 3D e cálculo vetorial inteligente para transformar o arraste na tela na rotação correta da camada.
  * Câmera orbital esférica e suporte a gestos touch (1 dedo e pinch-to-zoom).
* [`src/ui.rs`](src/ui.rs):
  * Interface completa com barra de status superior, HUD do cronômetro, barra de botões de ação e toolbar de faces.

---

## Compilar a partir do Código-Fonte

```bash
# Executar em modo desenvolvimento
cargo run

# Executar em modo otimizado (release)
cargo run --release
```
