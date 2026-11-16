# pvz-fusion-rust-edition
Protótipo jogável do Plants vs. Zombies Fusion reescrito do zero em Rust (com Macroquad)
# 🌻🧟 Plants vs. Zombies Fusion — Rust Edition

Protótipo jogável inspirado em **PvZ Fusion**, reescrito do zero em **Rust** com [Macroquad](https://macroquad.rs) como parte de um desafio de programação!

---

## 🧬 Receitas de Fusão Disponíveis
Plante uma espécie sobre outra compatível no tabuleiro (em qualquer ordem) para criar híbridos:

* **Disparervilha + Girassol** = **Sol+Ervilha** *(Atira mais rápido e gera 25 sóis a cada 7s)*
* **Disparervilha + Noz** = **Noz+Ervilha** *(Tanque blindado com disparo duplo)*
* **Noz + Cereja-Bomba** = **Noz-Bomba** *(Explode em área ao ser destruída pelos zumbis)*
* **Disparervilha + Cereja-Bomba** = **Metralha Explosiva** *(Dispara projéteis explosivos de alto dano)*

---

## 🎮 Controles do Jogo

| Tecla / Ação | Função | Custo |
| :--- | :--- | :--- |
| **Tecla `1`** | Selecionar Disparervilha | 100 Sóis |
| **Tecla `2`** | Selecionar Girassol | 50 Sóis |
| **Tecla `3`** | Selecionar Noz-Obstáculo | 50 Sóis |
| **Tecla `4`** | Selecionar Cereja-Bomba | 150 Sóis |
| **Clique Esquerdo** | Plantar em espaço vazio **ou Fundir** sobre outra planta | — |
| **Tecla `R` ou Clique** | Reiniciar partida após Game Over | — |

---

## 🚀 Como rodar no PC

1. Instale o **Rust** em [rustup.rs](https://rustup.rs).
2. Clone este repositório e execute:
   ```bash
   git clone https://github.com
   cd pvz-fusion-rust-edition
   cargo run --release
   ```
