# 🧊 Rubik's Cube 3D (Simulador Retrô)

Um simulador de Cubo Mágico 3D em **Rust** utilizando o motor de jogos **[Bevy](https://bevyengine.org)** (v0.13), com estética retrô CRT/PS1, animações suaves e controles precisos.

---

## 🎮 Controles

### Rotação das Faces (Notação Oficial WCA)
* `U`: Face Superior (Up - horário)
* `D`: Face Inferior (Down - horário)
* `R`: Face Direita (Right - horário)
* `L`: Face Esquerda (Left - horário)
* `F`: Face Frontal (Front - horário)
* `B`: Face Traseira (Back - horário)
* `Shift` + `[Tecla]`: Sentido anti-horário (inverso/prime, ex: `U'`, `R'`)

### Câmera e Visualização
* **Clique & Arraste (Botão Esquerdo ou Direito do Mouse)**: Orbitar a câmera livremente ao redor do cubo.
* **Scroll do Mouse**: Zoom in / Zoom out.
* `Espaço`: Alternar auto-rotação da câmera.

### Ações de Jogo & HUD
* `S`: **Embaralhar (Scramble)** com animação fluida (sequência aleatória de 20 movimentos sem repetições conflitantes).
* `X`: **Reset** instantâneo do cubo para a posição original resolvida.
* `Esc` ou `Q`: Fechar a aplicação.

---

## ⏱️ HUD e Detecção de Vitória (Speedcubing)

* **Fila de Movimentos Assíncrona**: Permite digitar algoritmos rápidos (ex: *sexy move* `R U R' U'`) sem perda de inputs.
* **Timer Integrado**: Inicia automaticamente no primeiro movimento após o embaralhamento (`S`).
* **Contador de Movimentos**: Registra cada rotação executada pelo jogador.
* **Detecção de Cubo Resolvido**: Identifica automaticamente quando todas as peças e orientações retornaram ao estado resolvido, travando o tempo final.

---

## 🛠️ Arquitetura

O projeto é estruturado em plugins modulares do Bevy ECS:

* [`src/main.rs`](src/main.rs): Inicialização da janela (640x480 sem decorações), iluminação, câmera orbital esférica, atalhos de saída e renderização da HUD superior/inferior.
* [`src/cube.rs`](src/cube.rs):
  * Estrutura hierárquica dos 27 cubinhos (`Cubie`) e seus respectivos adesivos coloridos.
  * Gerenciamento de rotações discretas por camadas lógicas inteiras (`IVec3`) para eliminar desvios de precisão numérica (*floating-point drift*).
  * Interpolação suave de rotação (`smoothstep`).
  * Fila de comandos de movimento (`MoveQueue`) e controle de estado do timer (`GameTimerState`).
* [`src/retro.rs`](src/retro.rs): Geração procedural de textura de *scanlines* CRT sobreposta na tela com `Msaa::Off` para manter a estética clássica de jogos da era 32-bit.

---

## 🚀 Como Executar

Certifique-se de ter o [Rust e Cargo](https://rustup.rs/) instalados:

```bash
cargo run --release
```
