# Plano de Implementacao: Suporte a Android e Geracao de APK

Este documento estabelece o plano tecnico e as etapas necessarias para adaptar o simulador de Cubo Magico 3D (Rust + Bevy Engine) para dispositivos moveis Android, gerando arquivos APK prontos para instalacao e execucao.

---

## 1. Visao Geral e Objetivos

- **Plataforma Alvo:** Android (arm64-v8a e armeabi-v7a).
- **Objetivo Principal:** Permitir a compilacao de arquivos `.apk` autônomos, instalaveis em celulares e tablets Android.
- **Experiencia do Usuario:** Adaptacao completa da jogabilidade para telas sensiveis ao toque (touch screen), dispensando o uso de teclado fisico e mouse.

---

## 2. Fases do Projeto

### Fase 1: Adaptacao de Entradas (Touch e Gestos)
- **Suporte a Touch:** Utilizar o recurso `Touches` da Bevy Engine para capturar toques simultaneos.
- **Orbita e Zoom por Gesto:**
  - 1 dedo arrastando: Orbita a camera ao redor do cubo.
  - 2 dedos (Pinch-to-zoom): Controla a distancia da camera (zoom in / zoom out).
  - 2 dedos arrastando: Deslocamento (pan) da camera.
- **Camada Unificada de Entrada:** Criar uma abstracao para que o codigo aceite tanto comandos de desktop (teclado/mouse) quanto de mobile (touch) de forma transparente.

### Fase 2: Interface de Usuario Mobile (Botoes Virtuais)
- **Botoes de Rotação de Faces:** Adicionar painel de botoes virtuais na tela para as 6 faces (`U`, `D`, `R`, `L`, `F`, `B`) e um seletor de sentido horario/anti-horario (`Prime / Inverso`).
- **Botoes de Acao Rapida:** Botoes dedicados para `Embaralhar (Scramble)` e `Reiniciar (Reset)`.
- **HUD Responsivo:** Ajustar o layout do cronometro, contador de movimentos e status para se adaptar tanto a orientacao retrato (vertical) quanto paisagem (horizontal).

### Fase 3: Configuracao de Build Android
- **Metadados no Cargo.toml:**
  Configurar a secao `[package.metadata.android]` com pacote, versao, orientacao de tela e icone do aplicativo.
- **Dependencias e Ferramental:**
  - `cargo-apk` ou `cargo-ndk`.
  - Android NDK (versao recomendada r26 ou superior).
  - Android SDK (API level 24+ / Android 7.0+).
- **Target Architectures:**
  - `aarch64-linux-android` (dispositivos modernos 64-bit).
  - `armv7-linux-androideabi` (dispositivos legados 32-bit).

### Fase 4: Pipeline de CI/CD (GitHub Actions)
- **Automacao do Build:** Adicionar um job dedicado no `.github/workflows/release.yml` para compilar o APK nos servidores do GitHub.
- **Publicacao Automatica:** Disponibilizar o arquivo `rubiks_sim-android.apk` diretamente na aba de Releases do repositorio a cada nova versao criada.

### Fase 5: Otimizacao e Polimento
- **Gerenciamento de Energia e FPS:** Travar a taxa de quadros em 60 FPS para evitar aquecimento excessivo e consumo desnecessario de bateria.
- **Pausar em Segundo Plano:** Tratamento do ciclo de vida da aplicacao para pausar o cronometro quando o app for minimizado no celular.

---

## 3. Estrutura de Arquivos Proposta

```text
rubiks_sim/
├── .github/
│   └── workflows/
│       └── release.yml          # Pipeline com job de build Android
├── assets/
│   ├── icon.png                 # Icone do aplicativo
│   └── fonts/                   # Fontes embarcadas para a HUD
├── src/
│   ├── main.rs                  # Ponto de entrada, configuracao de janelas e plugins
│   ├── cube.rs                  # Logica do cubo, rotacoes discretas e resolucao
│   ├── touch.rs                 # Modulo de captura de toques e gestos (Novo)
│   └── ui.rs                    # Interface grafica com botoes virtuais para mobile (Novo)
├── Cargo.toml                   # Dependencias e metadados Android
└── PLANO_ANDROID.md             # Este documento de planejamento
```

---

## 4. Cronograma de Execucao

| Etapa | Descricao | Status |
| :--- | :--- | :--- |
| 1 | Criacao do documento de planejamento | Concluido |
| 2 | Implementacao do modulo de Touch e botoes virtuais | Pendente |
| 3 | Configuracao de compilacao local com cargo-apk | Pendente |
| 4 | Integracao no GitHub Actions para geracao de APK | Pendente |
| 5 | Testes em dispositivo fisico e validacao de release | Pendente |
