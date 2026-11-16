use macroquad::prelude::*;

const ROWS: usize = 5;
const COLS: usize = 9;
const CELL_SIZE: f32 = 80.0;
const GRID_OFFSET_X: f32 = 60.0;
const GRID_OFFSET_Y: f32 = 100.0;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PlantType {
    // Plantas Base
    Peashooter,    // Tecla 1: Atira ervilhas
    Sunflower,     // Tecla 2: Gera sóis
    WallNut,       // Tecla 3: Tanque com muita vida
    CherryBomb,    // Tecla 4: Catalisador explosivo

    // Plantas Fundidas (PvZ Fusion)
    SunPeashooter, // Ervilha + Girassol: Atira rápido e gera sol a cada 7s
    PeaNut,        // Ervilha + Noz: Blindagem pesada + tiros duplos
    ExplodeNut,    // Noz + Cereja: Explode ao ser destruída pelos zumbis!
    GatlingCherry, // Ervilha + Cereja: Dispara rajadas explosivas de 3 ervilhas
}

impl PlantType {
    fn cost(&self) -> i32 {
        match self {
            PlantType::Peashooter => 100,
            PlantType::Sunflower => 50,
            PlantType::WallNut => 50,
            PlantType::CherryBomb => 150,
            _ => 0,
        }
    }

    fn max_hp(&self) -> f32 {
        match self {
            PlantType::WallNut => 450.0,
            PlantType::PeaNut => 550.0,
            PlantType::ExplodeNut => 400.0,
            PlantType::CherryBomb => 80.0,
            _ => 120.0,
        }
    }

    fn color(&self) -> Color {
        match self {
            PlantType::Peashooter => GREEN,
            PlantType::Sunflower => YELLOW,
            PlantType::WallNut => BROWN,
            PlantType::CherryBomb => RED,
            PlantType::SunPeashooter => LIME,
            PlantType::PeaNut => DARKGREEN,
            PlantType::ExplodeNut => ORANGE,
            PlantType::GatlingCherry => MAGENTA,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            PlantType::Peashooter => "Ervilha",
            PlantType::Sunflower => "Girassol",
            PlantType::WallNut => "Noz",
            PlantType::CherryBomb => "Cereja",
            PlantType::SunPeashooter => "Sol+Ervilha",
            PlantType::PeaNut => "Noz+Ervilha",
            PlantType::ExplodeNut => "Noz-Bomba",
            PlantType::GatlingCherry => "Metralha",
        }
    }
}

// Tabela de receitas de fusão (funciona em qualquer ordem!)
fn try_fuse(existing: PlantType, incoming: PlantType) -> Option<PlantType> {
    match (existing, incoming) {
        (PlantType::Peashooter, PlantType::Sunflower)
        | (PlantType::Sunflower, PlantType::Peashooter) => Some(PlantType::SunPeashooter),

        (PlantType::Peashooter, PlantType::WallNut)
        | (PlantType::WallNut, PlantType::Peashooter) => Some(PlantType::PeaNut),

        (PlantType::WallNut, PlantType::CherryBomb)
        | (PlantType::CherryBomb, PlantType::WallNut) => Some(PlantType::ExplodeNut),

        (PlantType::Peashooter, PlantType::CherryBomb)
        | (PlantType::CherryBomb, PlantType::Peashooter) => Some(PlantType::GatlingCherry),

        _ => None,
    }
}

#[derive(Clone, Copy)]
struct Plant {
    kind: PlantType,
    hp: f32,
    attack_timer: f32,
    sun_timer: f32,
}

struct Projectile {
    row: usize,
    x: f32,
    damage: f32,
    explosive: bool,
}

struct Zombie {
    row: usize,
    x: f32,
    hp: f32,
    max_hp: f32,
    speed: f32,
    is_conehead: bool,
}

struct ExplosionEffect {
    x: f32,
    y: f32,
    timer: f32,
}

#[macroquad::main("PvZ Fusion - Rust Edition")]
async fn main() {
    let mut grid: [[Option<Plant>; COLS]; ROWS] = [[None; COLS]; ROWS];
    let mut projectiles: Vec<Projectile> = Vec::new();
    let mut zombies: Vec<Zombie> = Vec::new();
    let mut explosions: Vec<ExplosionEffect> = Vec::new();

    let mut sun: i32 = 250;
    let mut selected_seed = PlantType::Peashooter;
    let mut sky_sun_timer: f32 = 0.0;
    let mut zombie_spawn_timer: f32 = 2.0;
    let mut score: i32 = 0;
    let mut wave: i32 = 1;
    let mut status_msg = String::from("Clique no gramado para plantar. Clique sobre outra planta para FUNDIR!");
    let mut game_over = false;

    loop {
        let dt = get_frame_time().min(0.1); // Trava delta-time para evitar saltos ao arrastar janela
        clear_background(Color::from_rgba(30, 120, 30, 255));

        if game_over {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 180));
            draw_text("GAME OVER! Os zumbis invadiram a casa!", 140.0, 250.0, 34.0, RED);
            draw_text(&format!("Pontuacao Final: {} | Onda: {}", score, wave), 240.0, 300.0, 26.0, YELLOW);
            draw_text("Pressione R ou clique na tela para jogar novamente", 180.0, 350.0, 22.0, WHITE);

            if is_key_pressed(KeyCode::R) || is_mouse_button_pressed(MouseButton::Left) {
                grid = [[None; COLS]; ROWS];
                projectiles.clear();
                zombies.clear();
                explosions.clear();
                sun = 250;
                score = 0;
                wave = 1;
                zombie_spawn_timer = 2.0;
                status_msg = String::from("Partida reiniciada! Boa sorte!");
                game_over = false;
            }
            next_frame().await;
            continue;
        }

        // Seleção de sementes pelo teclado (1, 2, 3, 4)
        if is_key_pressed(KeyCode::Key1) {
            selected_seed = PlantType::Peashooter;
        }
        if is_key_pressed(KeyCode::Key2) {
            selected_seed = PlantType::Sunflower;
        }
        if is_key_pressed(KeyCode::Key3) {
            selected_seed = PlantType::WallNut;
        }
        if is_key_pressed(KeyCode::Key4) {
            selected_seed = PlantType::CherryBomb;
        }

        // Sol caindo do céu a cada 5 segundos
        sky_sun_timer += dt;
        if sky_sun_timer >= 5.0 {
            sun += 25;
            sky_sun_timer = 0.0;
        }

        // Aumenta dificuldade por pontuação
        wave = 1 + (score / 100);

        // Spawn de Zumbis
        zombie_spawn_timer -= dt;
        if zombie_spawn_timer <= 0.0 {
            let random_row = rand::gen_range(0, ROWS);
            let is_cone = wave >= 2 && rand::gen_range(0, 100) < (wave * 15).min(60);
            let hp = if is_cone { 220.0 } else { 100.0 };

            zombies.push(Zombie {
                row: random_row,
                x: GRID_OFFSET_X + (COLS as f32 * CELL_SIZE) + 30.0,
                hp,
                max_hp: hp,
                speed: if is_cone { 16.0 } else { 20.0 },
                is_conehead: is_cone,
            });

            let min_spawn = (4.5 - (wave as f32 * 0.3)).max(1.5);
            zombie_spawn_timer = rand::gen_range(min_spawn, min_spawn + 2.0);
        }

        // Clique no tabuleiro para plantar ou FUNDIR
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= GRID_OFFSET_X && my >= GRID_OFFSET_Y {
                let col = ((mx - GRID_OFFSET_X) / CELL_SIZE) as usize;
                let row = ((my - GRID_OFFSET_Y) / CELL_SIZE) as usize;

                if row < ROWS && col < COLS {
                    let cost = selected_seed.cost();
                    match grid[row][col] {
                        None => {
                            if sun >= cost {
                                sun -= cost;
                                grid[row][col] = Some(Plant {
                                    kind: selected_seed,
                                    hp: selected_seed.max_hp(),
                                    attack_timer: 0.0,
                                    sun_timer: 0.0,
                                });
                                status_msg = format!("Plantado: {}", selected_seed.label());
                            } else {
                                status_msg = String::from("Sois insuficientes!");
                            }
                        }
                        Some(existing_plant) => {
                            if let Some(fused_kind) = try_fuse(existing_plant.kind, selected_seed) {
                                if sun >= cost {
                                    sun -= cost;
                                    grid[row][col] = Some(Plant {
                                        kind: fused_kind,
                                        hp: fused_kind.max_hp(),
                                        attack_timer: 0.0,
                                        sun_timer: 0.0,
                                    });
                                    status_msg = format!("FUSAO REALIZADA: {}!", fused_kind.label());
                                } else {
                                    status_msg = String::from("Sois insuficientes para fundir!");
                                }
                            } else {
                                status_msg = format!(
                                    "Sem receita entre {} + {}",
                                    existing_plant.kind.label(),
                                    selected_seed.label()
                                );
                            }
                        }
                    }
                }
            }
        }

        // Atualização das Plantas
        for r in 0..ROWS {
            for c in 0..COLS {
                if let Some(ref mut plant) = grid[r][c] {
                    plant.attack_timer += dt;
                    plant.sun_timer += dt;
                    let plant_x = GRID_OFFSET_X + (c as f32 * CELL_SIZE) + CELL_SIZE / 2.0;
                    let zombie_in_lane = zombies.iter().any(|z| z.row == r && z.x > plant_x - 10.0);

                    match plant.kind {
                        PlantType::Peashooter => {
                            if zombie_in_lane && plant.attack_timer >= 1.5 {
                                projectiles.push(Projectile {
                                    row: r,
                                    x: plant_x + 20.0,
                                    damage: 25.0,
                                    explosive: false,
                                });
                                plant.attack_timer = 0.0;
                            }
                        }
                        PlantType::Sunflower => {
                            if plant.sun_timer >= 6.5 {
                                sun += 25;
                                plant.sun_timer = 0.0;
                            }
                        }
                        PlantType::SunPeashooter => {
                            // Atira mais rápido E gera sol com timer fixo (sem bug de FPS)
                            if zombie_in_lane && plant.attack_timer >= 1.1 {
                                projectiles.push(Projectile {
                                    row: r,
                                    x: plant_x + 20.0,
                                    damage: 30.0,
                                    explosive: false,
                                });
                                plant.attack_timer = 0.0;
                            }
                            if plant.sun_timer >= 7.0 {
                                sun += 25;
                                plant.sun_timer = 0.0;
                            }
                        }
                        PlantType::PeaNut => {
                            // Dispara rajada dupla de ervilhas
                            if zombie_in_lane && plant.attack_timer >= 1.4 {
                                projectiles.push(Projectile {
                                    row: r,
                                    x: plant_x + 15.0,
                                    damage: 25.0,
                                    explosive: false,
                                });
                                projectiles.push(Projectile {
                                    row: r,
                                    x: plant_x + 35.0,
                                    damage: 25.0,
                                    explosive: false,
                                });
                                plant.attack_timer = 0.0;
                            }
                        }
                        PlantType::GatlingCherry => {
                            // Dispara tiros explosivos poderosos
                            if zombie_in_lane && plant.attack_timer >= 1.3 {
                                projectiles.push(Projectile {
                                    row: r,
                                    x: plant_x + 25.0,
                                    damage: 65.0,
                                    explosive: true,
                                });
                                plant.attack_timer = 0.0;
                            }
                        }
                        PlantType::WallNut | PlantType::ExplodeNut | PlantType::CherryBomb => {}
                    }
                }
            }
        }

        // Atualização dos Projéteis
        for proj in &mut projectiles {
            proj.x += 280.0 * dt;
        }

        // Colisão Projétil x Zumbi
        projectiles.retain(|proj| {
            let mut hit = false;
            for z in &mut zombies {
                if z.row == proj.row && (z.x - proj.x).abs() < 22.0 {
                    z.hp -= proj.damage;
                    hit = true;
                    if proj.explosive {
                        let y = GRID_OFFSET_Y + proj.row as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                        explosions.push(ExplosionEffect { x: proj.x, y, timer: 0.25 });
                    }
                    break;
                }
            }
            !hit && proj.x < 860.0
        });

        // Atualização dos Zumbis e Mordidas
        for z in &mut zombies {
            let col = ((z.x - GRID_OFFSET_X) / CELL_SIZE) as isize;
            let mut eating = false;

            if col >= 0 && (col as usize) < COLS {
                let c = col as usize;
                if let Some(ref mut plant) = grid[z.row][c] {
                    eating = true;
                    plant.hp -= 40.0 * dt;

                    if plant.hp <= 0.0 {
                        let dead_kind = plant.kind;
                        grid[z.row][c] = None;

                        // Efeito especial: Noz-Bomba explode ao morrer!
                        if dead_kind == PlantType::ExplodeNut {
                            let ex_x = GRID_OFFSET_X + c as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                            let ex_y = GRID_OFFSET_Y + z.row as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                            explosions.push(ExplosionEffect { x: ex_x, y: ex_y, timer: 0.5 });

                            for target in &mut zombies {
                                if (target.row as isize - z.row as isize).abs() <= 1
                                    && (target.x - ex_x).abs() < 130.0
                                {
                                    target.hp -= 250.0;
                                }
                            }
                        }
                    }
                }
            }

            if !eating {
                z.x -= z.speed * dt;
            }

            if z.x < GRID_OFFSET_X - 10.0 {
                game_over = true;
            }
        }

        // Remove zumbis mortos e soma pontos
        zombies.retain(|z| {
            if z.hp <= 0.0 {
                score += if z.is_conehead { 25 } else { 10 };
                false
            } else {
                true
            }
        });

        // Atualiza efeitos visuais de explosão
        for ex in &mut explosions {
            ex.timer -= dt;
        }
        explosions.retain(|ex| ex.timer > 0.0);

        // --- RENDERIZAÇÃO ---
        // HUD Superior
        draw_rectangle(0.0, 0.0, 900.0, 85.0, DARKBROWN);
        draw_text(&format!("SOL: {}", sun), 18.0, 34.0, 30.0, YELLOW);
        draw_text(&format!("PTS: {} | ONDA {}", score, wave), 18.0, 64.0, 20.0, WHITE);

        draw_text(
            " Ervilha(100)  Girassol(50)  Noz(50)  Cereja(150)",
            210.0,
            32.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Semente: {} | {}", selected_seed.label(), status_msg),
            210.0,
            64.0,
            18.0,
            GOLD,
        );

        // Desenhar Grid e Plantas
        for r in 0..ROWS {
            for c in 0..COLS {
                let x = GRID_OFFSET_X + c as f32 * CELL_SIZE;
                let y = GRID_OFFSET_Y + r as f32 * CELL_SIZE;
                let tile_color = if (r + c) % 2 == 0 {
                    Color::from_rgba(46, 160, 46, 255)
                } else {
                    Color::from_rgba(38, 145, 38, 255)
                };
                draw_rectangle(x, y, CELL_SIZE - 2.0, CELL_SIZE - 2.0, tile_color);

                if let Some(plant) = grid[r][c] {
                    draw_circle(x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0, 25.0, plant.kind.color());
                    draw_text(plant.kind.label(), x + 6.0, y + 73.0, 13.0, WHITE);

                    // Barra de vida da planta
                    let hp_ratio = (plant.hp / plant.kind.max_hp()).clamp(0.0, 1.0);
                    draw_rectangle(x + 10.0, y + 6.0, 60.0 * hp_ratio, 5.0, SKYBLUE);
                }
            }
        }

        // Desenhar Projéteis
        for proj in &projectiles {
            let y = GRID_OFFSET_Y + proj.row as f32 * CELL_SIZE + CELL_SIZE / 2.0;
            let color = if proj.explosive { RED } else { LIME };
            draw_circle(proj.x, y, if proj.explosive { 10.0 } else { 8.0 }, color);
        }

        // Desenhar Zumbis
        for z in &zombies {
            let y = GRID_OFFSET_Y + z.row as f32 * CELL_SIZE + 16.0;
            let body_color = if z.is_conehead { MAROON } else { PURPLE };
            draw_rectangle(z.x - 15.0, y, 30.0, 48.0, body_color);

            if z.is_conehead {
                // Desenha cone laranja na cabeça do zumbi
                draw_triangle(
                    vec2(z.x, y - 14.0),
                    vec2(z.x - 12.0, y),
                    vec2(z.x + 12.0, y),
                    ORANGE,
                );
            }

            let hp_ratio = (z.hp / z.max_hp).clamp(0.0, 1.0);
            draw_rectangle(z.x - 15.0, y - 20.0, 30.0 * hp_ratio, 4.0, RED);
        }

        // Desenhar Explosões
        for ex in &explosions {
            draw_circle(ex.x, ex.y, 45.0, Color::from_rgba(255, 140, 0, 160));
        }

        next_frame().await;
    }
}
