#[allow(unused)]
use color_eyre::{
    Result,
    owo_colors::{OwoColorize, colors::css::AliceBlue},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rand::Rng;
#[allow(unused)]
use ratatui::{
    DefaultTerminal, Frame,
    layout::Constraint::{self, Fill, Length, Max, Min, Percentage, Ratio},
    prelude::*,
    style::Stylize,
    symbols::{border::FULL, scrollbar::VERTICAL},
    text::Line,
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
};
#[allow(unused)]
use ratatui::{style::palette::material::GRAY, symbols::border};
#[allow(unused)]
use serde::{Deserialize, Serialize};
#[allow(unused)]
use std::fs;
#[allow(unused)]
use std::io;
#[allow(unused)]
use std::time::{Duration, Instant};
use std::{cmp::Ordering, default};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItem {
    pub name: String,
    pub quantity: u32,
    pub description: String,
    pub usable: bool,
    pub value: u32,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    #[default]
    MainMenu,
    Story,
    Fight,
    Minigame,
    GameOver,
    Heal,
    Shop,
    Inventory,
    Test,
    Mercy,
    Battle,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub enum BossType {
    #[default]
    None,
    SamosGuardian,  // Early game boss
    TyrantOfTyre,   // Mid game boss
    BabylonianSage, // Late game boss
    FinalBoss,      // End game boss
}

#[derive(Debug, Clone, Default)]
pub struct Boss {
    pub boss_type: BossType,
    pub name: String,
    pub max_health: f64,
    pub current_health: f64,
    pub damage: f64,
    pub defense: f64,
    pub special_attack_cooldown: u32,
    pub current_cooldown: u32,
    pub phase: u32,
    pub defeated: bool,
    pub description: String,
    pub special_ability: String,
}

impl Boss {
    pub fn new(boss_type: BossType) -> Self {
        match boss_type {
            BossType::SamosGuardian => Boss {
                boss_type: BossType::SamosGuardian,
                name: "Guardiano di Samos".to_string(),
                max_health: 200.0,
                current_health: 200.0,
                damage: 25.0,
                defense: 10.0,
                special_attack_cooldown: 3,
                current_cooldown: 0,
                phase: 1,
                defeated: false,
                description: "Un antico guardiano che protegge i segreti di Samos".to_string(),
                special_ability: "Scudo Geometrico - Riduce il danno del 50% per 2 turni"
                    .to_string(),
            },
            BossType::TyrantOfTyre => Boss {
                boss_type: BossType::TyrantOfTyre,
                name: "Tiranno di Tiro".to_string(),
                max_health: 350.0,
                current_health: 350.0,
                damage: 35.0,
                defense: 15.0,
                special_attack_cooldown: 4,
                current_cooldown: 0,
                phase: 1,
                defeated: false,
                description: "Un tiranno crudele che governa Tiro con pugno di ferro".to_string(),
                special_ability: "Ira del Tiranno - Attacco devastante che ignora la difesa"
                    .to_string(),
            },
            BossType::BabylonianSage => Boss {
                boss_type: BossType::BabylonianSage,
                name: "Saggio Babilonese".to_string(),
                max_health: 500.0,
                current_health: 500.0,
                damage: 45.0,
                defense: 20.0,
                special_attack_cooldown: 5,
                current_cooldown: 0,
                phase: 1,
                defeated: false,
                description: "Un antico saggio che custodisce i misteri della matematica"
                    .to_string(),
                special_ability: "Teorema Antico - Si cura e potenzia i suoi attacchi".to_string(),
            },
            BossType::FinalBoss => Boss {
                boss_type: BossType::FinalBoss,
                name: "Ombra del Caos".to_string(),
                max_health: 750.0,
                current_health: 750.0,
                damage: 60.0,
                defense: 25.0,
                special_attack_cooldown: 3,
                current_cooldown: 0,
                phase: 1,
                defeated: false,
                description: "L'antitesi di tutto ciò che Pitagora rappresenta".to_string(),
                special_ability: "Caos Numerico - Confonde e danneggia gravemente".to_string(),
            },
            BossType::None => Boss {
                boss_type: BossType::None,
                name: "Nessun Boss".to_string(),
                max_health: 0.0,
                current_health: 0.0,
                damage: 0.0,
                defense: 0.0,
                special_attack_cooldown: 0,
                current_cooldown: 0,
                phase: 1,
                defeated: true,
                description: "".to_string(),
                special_ability: "".to_string(),
            },
        }
    }

    pub fn get_health_percentage(&self) -> f64 {
        (self.current_health / self.max_health) * 100.0
    }

    pub fn is_special_ready(&self) -> bool {
        self.current_cooldown == 0
    }

    pub fn use_special_attack(&mut self) {
        self.current_cooldown = self.special_attack_cooldown;
    }

    pub fn tick_cooldown(&mut self) {
        if self.current_cooldown > 0 {
            self.current_cooldown -= 1;
        }
    }

    pub fn enter_next_phase(&mut self) {
        self.phase += 1;
        match self.boss_type {
            BossType::SamosGuardian => {
                if self.phase == 2 {
                    self.damage *= 1.2;
                    self.special_attack_cooldown = 2; // More frequent specials
                }
            }
            BossType::TyrantOfTyre => {
                if self.phase == 2 {
                    self.damage *= 1.3;
                    self.defense *= 0.8; // Less defense but more damage
                }
            }
            BossType::BabylonianSage => {
                if self.phase == 2 {
                    self.current_health += 100.0; // Heals when entering phase 2
                    self.damage *= 1.4;
                }
            }
            BossType::FinalBoss => {
                if self.phase == 2 {
                    self.damage *= 1.5;
                    self.special_attack_cooldown = 2;
                } else if self.phase == 3 {
                    self.damage *= 1.8;
                    self.special_attack_cooldown = 1; // Very frequent specials
                }
            }
            BossType::None => {}
        }
    }

    pub fn should_enter_next_phase(&self) -> bool {
        let health_percentage = self.get_health_percentage();
        match self.boss_type {
            BossType::SamosGuardian => self.phase == 1 && health_percentage <= 50.0,
            BossType::TyrantOfTyre => self.phase == 1 && health_percentage <= 40.0,
            BossType::BabylonianSage => self.phase == 1 && health_percentage <= 30.0,
            BossType::FinalBoss => {
                (self.phase == 1 && health_percentage <= 60.0)
                    || (self.phase == 2 && health_percentage <= 25.0)
            }
            BossType::None => false,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum FightOption {
    #[default]
    Attack,
    Defend,
    Inventory,
    Mercy,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ShopOption {
    #[default]
    Buy,
    Sell,
    Inventory,
    Exit,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Places {
    #[default]
    Samos,
    SabbiaSamos,
    Tiro,
    ColonneTiro,
    Crotone,
    ScuolaCrotone,
    Babilonia,
    BabiloniaBoss,
    Olimpia,
    Syros,
    Mileto,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum StoryState {
    #[default]
    First,
    Second,
    Third,
}

#[derive(Debug, Default, Clone)]
pub struct App {
    running: bool,
    pub previous_game_state: Option<GameState>,
    pub saved_state_before_inventory: Option<Box<App>>,
    pub game_state: GameState,
    pub story_state: StoryState,
    pub counter: u32,
    pub player_strength: f64,
    pub player_dmg: f64,
    pub player_health: f64,
    pub player_def: f64,
    pub player_xp: f64,
    pub player_lvl: f64,
    pub player_player_place: Places,
    pub player_heal_value: f64,
    pub player_heal_factor: f64,
    pub player_xp_factor: f64,
    pub player_inventory: Vec<String>,
    pub selected_fight_option: FightOption,
    pub selected_shop_option: ShopOption,
    pub enemy_health: f64,
    pub enemy_strength: f64,
    pub enemy_dmg: f64,
    pub enemy_strength_factor: f64,
    pub enemy_heal: f64,
    pub enemy_is_alive: bool,
    pub mercy_outcome: Option<bool>,
    pub message_log: Vec<String>,
    pub boss_dialogue_index: usize,
    pub boss_dialogue: Vec<String>,
    pub is_boss_battle: bool,
    pub current_boss: Boss, // New field for message history
}

#[allow(deprecated)]
#[allow(dead_code)]
impl App {
    pub fn new() -> Self {
        Self {
            player_health: 1000.0,
            player_strength: 1.0,
            player_dmg: 15.0,
            player_def: 5.0,
            player_xp: 0.0,
            player_lvl: 1.0,
            player_heal_value: 20.0,
            player_heal_factor: 1.0,
            player_xp_factor: 1.0,
            enemy_health: 150.0,
            enemy_strength: 1.1,
            enemy_dmg: 10.0,
            enemy_heal: 15.0,
            saved_state_before_inventory: None,
            mercy_outcome: None,
            message_log: vec!["Benvenuto nel mondo di Pitagora!".to_string()],
            current_boss: Boss::new(BossType::FinalBoss),
            is_boss_battle: true,
            boss_dialogue: vec![],
            boss_dialogue_index: 0,
            ..Self::default()
        }
    }

    pub fn start_boss_battle(&mut self, boss_type: BossType) {
        self.current_boss = Boss::new(boss_type);
        self.is_boss_battle = true;
        self.game_state = GameState::Battle;
        self.boss_dialogue_index = 0;

        // Set boss-specific dialogue
        self.boss_dialogue = match boss_type {
            BossType::SamosGuardian => vec![
                "Chi osa disturbare l'antica saggezza di Samos?".to_string(),
                "I segreti geometrici non sono per i deboli!".to_string(),
                "Dimostra la tua conoscenza in battaglia!".to_string(),
            ],
            BossType::TyrantOfTyre => vec![
                "Un altro sfidante si presenta davanti al mio trono!".to_string(),
                "Nessuno può sfidare il mio potere a Tiro!".to_string(),
                "Preparati a cadere davanti alla mia ira!".to_string(),
            ],
            BossType::BabylonianSage => vec![
                "Ah, un giovane studioso cerca la conoscenza antica...".to_string(),
                "Ma prima devi dimostrare di essere degno!".to_string(),
                "I misteri di Babilonia non si rivelano facilmente!".to_string(),
            ],
            BossType::FinalBoss => vec![
                "Così... hai raggiunto la fine del tuo viaggio...".to_string(),
                "Io sono tutto ciò che si oppone all'ordine e alla ragione!".to_string(),
                "Preparati ad affrontare il CAOS ASSOLUTO!".to_string(),
            ],
            BossType::None => vec![],
        };

        let boss_name = self.current_boss.name.clone();
        self.add_message(format!("Boss battle iniziata: {}!", boss_name));
    }

    // Enhanced battle rendering for bosses
    fn render_boss_battle(&mut self, frame: &mut Frame, area: Rect) {
        let battle_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Boss info and health bar
                Constraint::Percentage(30), // Boss dialogue/status
                Constraint::Percentage(20), // Battle options
            ])
            .split(area);

        // Render boss info with health bar
        self.render_boss_info(frame, battle_layout[0]);

        // Render boss dialogue or status
        self.render_boss_dialogue(frame, battle_layout[1]);

        // Render battle options
        self.render_battle_options(frame, battle_layout[2]);
    }

    fn render_boss_info(&mut self, frame: &mut Frame, area: Rect) {
        let boss = &self.current_boss;
        let health_percentage = boss.get_health_percentage();

        // Health bar color based on percentage
        let health_color = if health_percentage > 60.0 {
            Color::Green
        } else if health_percentage > 30.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Create health bar
        let health_bar_width = (area.width as f64 * (health_percentage / 100.0)) as u16;
        let health_bar = "█".repeat(health_bar_width.min(area.width.saturating_sub(4)) as usize);

        let boss_info = vec![
            Line::from(boss.name.clone()).bold().red().centered(),
            Line::from(""),
            Line::from(format!("Fase: {}", boss.phase))
                .yellow()
                .centered(),
            Line::from(""),
            Line::from(format!(
                "HP: {:.0}/{:.0} ({:.1}%)",
                boss.current_health, boss.max_health, health_percentage
            ))
            .style(Style::default().fg(health_color))
            .centered(),
            Line::from(health_bar)
                .style(Style::default().fg(health_color))
                .centered(),
            Line::from(""),
            Line::from(if boss.is_special_ready() {
                format!("⚡ {} - PRONTO!", boss.special_ability)
            } else {
                format!("⏳ Abilità speciale: {} turni", boss.current_cooldown)
            })
            .cyan()
            .centered(),
        ];

        let boss_block = Block::bordered()
            .title(Line::from(" BOSS BATTLE ").bold().red())
            .border_style(Style::default().red());

        frame.render_widget(
            Paragraph::new(boss_info)
                .block(boss_block)
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_boss_dialogue(&mut self, frame: &mut Frame, area: Rect) {
        let dialogue_text = if self.boss_dialogue_index < self.boss_dialogue.len() {
            vec![
                Line::from(""),
                Line::from(format!(
                    "\"{}\"",
                    self.boss_dialogue[self.boss_dialogue_index]
                ))
                .italic()
                .white()
                .centered(),
                Line::from(""),
                Line::from("Premi Spazio per continuare...")
                    .dark_gray()
                    .centered(),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from("La battaglia è iniziata!")
                    .bold()
                    .red()
                    .centered(),
                Line::from(""),
                Line::from("Scegli la tua azione...").dark_gray().centered(),
            ]
        };

        let dialogue_block = Block::bordered()
            .title(" Dialogo ")
            .border_style(Style::default().yellow());

        frame.render_widget(
            Paragraph::new(dialogue_text)
                .block(dialogue_block)
                .alignment(Alignment::Center),
            area,
        );
    }

    // Enhanced attack logic for boss battles
    fn logic_boss_attack(&mut self) {
        let mut damage_dealt = self.player_dmg;
        let boss_defense = self.current_boss.defense;

        // Apply boss defense
        damage_dealt = (damage_dealt - boss_defense).max(1.0);

        self.current_boss.current_health -= damage_dealt;
        self.add_message(format!("Hai inflitto {:.0} danni al boss!", damage_dealt));

        // Check if boss should enter next phase
        if self.current_boss.should_enter_next_phase() {
            let old_phase = self.current_boss.phase;
            self.current_boss.enter_next_phase();
            self.add_message(format!(
                "{} entra nella fase {}!",
                self.current_boss.name, self.current_boss.phase
            ));
        }

        // Check if boss is defeated
        if self.current_boss.current_health <= 0.0 {
            self.current_boss.defeated = true;
            self.is_boss_battle = false;
            self.logic_boss_victory();
            return;
        }

        // Boss counterattack
        self.logic_boss_counterattack();
    }

    fn logic_boss_counterattack(&mut self) {
        // Extract the values we need first, before any mutable borrowing
        let boss_name = self.current_boss.name.clone();
        let boss_damage = self.current_boss.damage;
        let is_special_ready = self.current_boss.is_special_ready();

        // Decide if boss uses special attack
        let use_special = is_special_ready && rand::thread_rng().gen_bool(0.6);

        if use_special {
            self.logic_boss_special_attack();
        } else {
            // Normal attack
            let mut damage = boss_damage;
            // Add some randomness
            damage *= rand::thread_rng().gen_range(0.8..1.2);
            self.player_health -= damage;
            self.add_message(format!("{} ti attacca per {:.0} danni!", boss_name, damage));
        }

        // Check if player is defeated
        if self.player_health <= 0.0 {
            self.game_state = GameState::GameOver;
        }
    }

    fn logic_boss_special_attack(&mut self) {
        // Clone/copy the values we need before borrowing mutably
        let boss_name = self.current_boss.name.clone();
        let boss_type = self.current_boss.boss_type.clone(); // Assuming BossType implements Clone
        let boss_damage = self.current_boss.damage;
        let boss_max_health = self.current_boss.max_health;

        match boss_type {
            BossType::SamosGuardian => {
                // Scudo Geometrico - reduces incoming damage
                let damage = boss_damage * 1.5;
                self.player_health -= damage;
                self.add_message(format!(
                    "{} usa Scudo Geometrico! {:.0} danni!",
                    boss_name, damage
                ));
            }
            BossType::TyrantOfTyre => {
                // Ira del Tiranno - ignores player defense
                let damage = boss_damage * 2.0;
                self.player_health -= damage;
                self.add_message(format!(
                    "{} scatena la sua Ira! {:.0} danni devastanti!",
                    boss_name, damage
                ));
            }
            BossType::BabylonianSage => {
                // Teorema Antico - heals and buffs
                self.current_boss.current_health += 50.0;
                self.current_boss.current_health =
                    self.current_boss.current_health.min(boss_max_health);
                self.current_boss.damage *= 1.1;
                self.add_message(format!("{} usa un Teorema Antico! Si rafforza!", boss_name));
            }
            BossType::FinalBoss => {
                // Caos Numerico - massive damage and debuff
                let damage = boss_damage * 2.5;
                self.player_health -= damage;
                self.player_dmg *= 0.9; // Temporary debuff
                self.add_message(format!(
                    "{} scatena il Caos Numerico! {:.0} danni! Sei indebolito!",
                    boss_name, damage
                ));
            }
            BossType::None => {}
        }
        self.current_boss.use_special_attack();
    }

    fn logic_boss_victory(&mut self) {
        let boss_type = self.current_boss.boss_type.clone();
        let boss_name = self.current_boss.name.clone();

        // Give rewards based on boss type
        match boss_type {
            BossType::SamosGuardian => {
                self.player_xp += 100.0;
                self.player_inventory
                    .push("Frammento Geometrico di Samos".to_string());
                self.add_message("Hai sconfitto il Guardiano di Samos!".to_string());
            }
            BossType::TyrantOfTyre => {
                self.player_xp += 200.0;
                self.player_dmg += 5.0;
                self.player_inventory.push("Corona del Tiranno".to_string());
                self.add_message("Hai liberato Tiro dal tiranno!".to_string());
            }
            BossType::BabylonianSage => {
                self.player_xp += 300.0;
                self.player_heal_factor += 0.5;
                self.player_inventory
                    .push("Tavoletta Babilonese Antica".to_string());
                self.add_message("Hai ottenuto la saggezza babilonese!".to_string());
            }
            BossType::FinalBoss => {
                self.player_xp += 500.0;
                self.player_lvl += 1.0;
                self.player_inventory
                    .push("Cristallo dell'Ordine".to_string());
                self.add_message(
                    "Hai sconfitto il Caos! Sei un vero seguace di Pitagora!".to_string(),
                );
            }
            BossType::None => {}
        }

        self.game_state = GameState::Story;
    }

    // Method to trigger boss battles based on location
    pub fn check_for_boss_encounter(&mut self) {
        let boss_type = match self.player_player_place {
            Places::SabbiaSamos => BossType::SamosGuardian,
            Places::ColonneTiro => BossType::TyrantOfTyre,
            Places::BabiloniaBoss => BossType::BabylonianSage,
            Places::Olimpia => BossType::FinalBoss,
            _ => BossType::None,
        };

        if boss_type != BossType::None {
            self.start_boss_battle(boss_type);
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn add_message(&mut self, message: String) {
        self.message_log.push(message);
        // Keep only the last 10 messages to prevent overflow
        if self.message_log.len() > 10 {
            self.message_log.remove(0);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        // Create the main three-panel layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Map area
                Constraint::Percentage(20), // Player stats
                Constraint::Percentage(20), // Message log
            ])
            .split(frame.area());

        // Render the map/main content area
        self.render_main_content(frame, main_layout[0]);

        // Render player stats
        self.render_player_stats(frame, main_layout[1]);

        // Render message log
        self.render_message_log(frame, main_layout[2]);
    }

    fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        match self.game_state {
            GameState::MainMenu => self.render_main_menu(frame, area),
            GameState::Story => self.render_story(frame, area),
            GameState::Fight => self.render_battle(frame, area),
            GameState::Shop => self.render_shop(frame, area),
            GameState::Inventory => self.render_inventory(frame, area),
            GameState::Mercy => self.render_mercy(frame, area),
            GameState::GameOver => self.render_game_over(frame, area),
            GameState::Heal => self.render_heal(frame, area),
            GameState::Minigame => self.render_minigame(frame, area),
            GameState::Test => self.render_test(frame, area),
            GameState::Battle => {
                if self.is_boss_battle {
                    self.render_boss_battle(frame, area);
                } else {
                    self.render_battle(frame, area);
                }
            }
        }
    }

    fn render_player_stats(&mut self, frame: &mut Frame, area: Rect) {
        let max_hp = 100.0 + (self.player_lvl - 1.0) * 20.0;
        let hp_percentage = (self.player_health / max_hp * 100.0) as u8;

        let stats_text = vec![
            Line::from(vec![
                "HP: ".into(),
                format!("{:.0}/{:.0}", self.player_health, max_hp)
                    .red()
                    .bold(),
                format!(" ({}%)", hp_percentage).dark_gray(),
            ]),
            Line::from(vec![
                "LVL: ".into(),
                format!("{:.0}", self.player_lvl).yellow().bold(),
                " | XP: ".into(),
                format!("{:.0}", self.player_xp).cyan(),
            ]),
            Line::from(vec![
                "ATK: ".into(),
                format!("{:.0}", self.player_dmg).red(),
                " | DEF: ".into(),
                format!("{:.0}", self.player_def).blue(),
            ]),
            Line::from(vec!["Luogo: ".into(), self.get_place_name().green().bold()]),
        ];

        let stats_block = Block::bordered()
            .title(" Statistiche Giocatore ")
            .title_style(Style::default().yellow().bold())
            .border_style(Style::default().cyan());

        frame.render_widget(
            Paragraph::new(stats_text)
                .block(stats_block)
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn render_message_log(&mut self, frame: &mut Frame, area: Rect) {
        let message_lines: Vec<Line> = self
            .message_log
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                if i == self.message_log.len() - 1 {
                    Line::from(format!("> {}", msg)).white().bold()
                } else {
                    Line::from(format!("  {}", msg)).dark_gray()
                }
            })
            .collect();

        let log_block = Block::bordered()
            .title(" Registro Messaggi ")
            .title_style(Style::default().magenta().bold())
            .border_style(Style::default().gray());

        frame.render_widget(
            Paragraph::new(message_lines)
                .block(log_block)
                .wrap(Wrap { trim: false })
                .scroll((0, 0)),
            area,
        );
    }

    fn get_place_name(&self) -> &str {
        match self.player_player_place {
            Places::Samos => "Samos",
            Places::SabbiaSamos => "Sabbia di Samos",
            Places::Tiro => "Tiro",
            Places::ColonneTiro => "Colonne di Tiro",
            Places::Crotone => "Crotone",
            Places::ScuolaCrotone => "Scuola di Crotone",
            Places::Babilonia => "Babilonia",
            Places::BabiloniaBoss => "Palazzo di Babilonia",
            Places::Olimpia => "Olimpia",
            Places::Syros => "Syros",
            Places::Mileto => "Mileto",
        }
    }

    fn render_main_menu(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from(" Menù Principale ").bold().blue().centered();

        let menu_text = vec![
            Line::from(""),
            Line::from(" Salve, Avventuriero! ").centered(),
            Line::from(""),
            Line::from(" Ti trovi nel menù principale.")
                .blue()
                .centered(),
            Line::from(""),
            Line::from("Cosa desideri fare?").bold().centered(),
            Line::from(""),
            Line::from("(S) Inizia Storia | (H) Guarigione | (W) Negozio").centered(),
            Line::from("(I) Inventario | (T) Test | (E) Esci").centered(),
        ];

        frame.render_widget(
            Paragraph::new(menu_text)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_battle(&mut self, frame: &mut Frame, area: Rect) {
        // Split battle area into enemy info and options
        let battle_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Enemy info
                Constraint::Percentage(40), // Battle options
            ])
            .split(area);

        // Render enemy information
        let enemy_title = Line::from("Battaglia Imminente!").bold().red().centered();
        let enemy_text = vec![
            Line::from(""),
            Line::from("Un Brigante selvaggio appare!").centered(),
            Line::from(""),
            Line::from(format!("Salute del Brigante: {:.0}", self.enemy_health))
                .red()
                .centered(),
            Line::from(""),
            Line::from("Prepara la tua mossa...").dark_gray().centered(),
        ];

        frame.render_widget(
            Paragraph::new(enemy_text)
                .block(Block::bordered().title(enemy_title))
                .alignment(Alignment::Center),
            battle_layout[0],
        );

        // Render battle options
        self.render_battle_options(frame, battle_layout[1]);
    }

    fn render_battle_options(&mut self, frame: &mut Frame, area: Rect) {
        let options_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        let get_option_style = |option: FightOption, current_selection: FightOption| {
            if option == current_selection {
                Style::default().bg(Color::Yellow).black().bold()
            } else {
                Style::default().white()
            }
        };

        // Attack option
        let attack_text = Line::from("ATTACCA")
            .style(get_option_style(
                FightOption::Attack,
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(attack_text)
                .block(Block::bordered())
                .alignment(Alignment::Center),
            options_layout[0],
        );

        // Defend option
        let defend_text = Line::from("DIFENDI")
            .style(get_option_style(
                FightOption::Defend,
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(defend_text)
                .block(Block::bordered())
                .alignment(Alignment::Center),
            options_layout[1],
        );

        // Inventory option
        let inventory_text = Line::from("INVENTARIO")
            .style(get_option_style(
                FightOption::Inventory,
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(inventory_text)
                .block(Block::bordered())
                .alignment(Alignment::Center),
            options_layout[2],
        );

        // Mercy option
        let mercy_text = Line::from("PIETA'")
            .style(get_option_style(
                FightOption::Mercy,
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(mercy_text)
                .block(Block::bordered())
                .alignment(Alignment::Center),
            options_layout[3],
        );
    }

    fn render_story(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from("La Storia di Pitagora").bold().blue().centered();

        let (story_text, controls) = match self.story_state {
            StoryState::First => (
                vec![
                    Line::from(""),
                    Line::from("Nel VI secolo a.C., il grande matematico Pitagora").centered(),
                    Line::from("viaggiava attraverso il mondo mediterraneo,").centered(),
                    Line::from("alla ricerca della conoscenza universale...").centered(),
                    Line::from(""),
                    Line::from("Nato a Samos, iniziò il suo viaggio verso la saggezza").centered(),
                    Line::from("studiando presso i saggi dell'Oriente.").centered(),
                ],
                Line::from("Premi (C) per continuare | (B) per Battaglia | (M) per Menu")
                    .dark_gray()
                    .centered(),
            ),
            StoryState::Second => (
                vec![
                    Line::from(""),
                    Line::from("Dopo anni di studio in Egitto e Babilonia,").centered(),
                    Line::from("Pitagora apprese i segreti della geometria").centered(),
                    Line::from("e scoprì le relazioni mistiche tra i numeri.").centered(),
                    Line::from(""),
                    Line::from("Ma il suo destino lo chiamava in Magna Grecia,").centered(),
                    Line::from("dove avrebbe fondato la sua famosa scuola.").centered(),
                ],
                Line::from("Premi (C) per continuare | (B) per Battaglia | (M) per Menu")
                    .dark_gray()
                    .centered(),
            ),
            StoryState::Third => (
                vec![
                    Line::from(""),
                    Line::from("A Crotone, Pitagora fondò una comunità di studiosi").centered(),
                    Line::from("dove matematica, filosofia e musica si univano").centered(),
                    Line::from("in un'armonia perfetta.").centered(),
                    Line::from(""),
                    Line::from("Il teorema che porta il suo nome sarebbe diventato").centered(),
                    Line::from("una delle scoperte più importanti dell'umanità.").centered(),
                    Line::from(""),
                    Line::from("La tua avventura nelle terre di Pitagora inizia ora...")
                        .yellow()
                        .bold()
                        .centered(),
                ],
                Line::from("Storia completata! | (B) per Battaglia | (M) per Menu")
                    .dark_gray()
                    .centered(),
            ),
        };

        let mut full_story = story_text;
        full_story.push(Line::from(""));
        full_story.push(controls);

        frame.render_widget(
            Paragraph::new(full_story)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_shop(&mut self, frame: &mut Frame, area: Rect) {
        let shop_name = self.get_shop_name();
        let title = Line::from(format!(" {} ", shop_name))
            .bold()
            .blue()
            .centered();

        // Split shop area
        let shop_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Shop info
                Constraint::Percentage(40), // Shop options
            ])
            .split(area);

        let shop_text = vec![
            Line::from(""),
            Line::from("Benvenuto nel negozio!").centered(),
            Line::from("Cosa desideri fare?").centered(),
            Line::from(""),
            Line::from("Usa le frecce per navigare, Invio per selezionare")
                .dark_gray()
                .centered(),
        ];

        frame.render_widget(
            Paragraph::new(shop_text)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Center),
            shop_layout[0],
        );

        self.render_shop_options(frame, shop_layout[1]);
    }

    fn render_shop_options(&mut self, frame: &mut Frame, area: Rect) {
        let options_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        let get_option_style = |option: ShopOption, current_selection: ShopOption| {
            if option == current_selection {
                Style::default().bg(Color::Yellow).black().bold()
            } else {
                Style::default().white()
            }
        };

        let options = [
            ("COMPRA", ShopOption::Buy),
            ("VENDI", ShopOption::Sell),
            ("INVENTARIO", ShopOption::Inventory),
            ("ESCI", ShopOption::Exit),
        ];

        for (i, (text, option)) in options.iter().enumerate() {
            let option_text = Line::from(*text)
                .style(get_option_style(*option, self.selected_shop_option))
                .centered();
            frame.render_widget(
                Paragraph::new(option_text)
                    .block(Block::bordered())
                    .alignment(Alignment::Center),
                options_layout[i],
            );
        }
    }

    fn get_shop_name(&self) -> &str {
        match self.player_player_place {
            Places::Samos => "Negozio di Samos",
            Places::SabbiaSamos => "Bancarella nella Sabbia di Samos",
            Places::Tiro => "Emporio di Tiro",
            Places::ColonneTiro => "Mercato delle Colonne di Tiro",
            Places::Crotone => "Bottega di Crotone",
            Places::ScuolaCrotone => "Spaccio della Scuola di Crotone",
            Places::Babilonia => "Mercante Babilonese",
            Places::BabiloniaBoss => "Tesori del Boss di Babilonia",
            Places::Olimpia => "Bazar Olimpico",
            Places::Syros => "Mercato di Syros",
            Places::Mileto => "Bottega di Mileto",
        }
    }

    fn render_inventory(&mut self, frame: &mut Frame, area: Rect) {
        let inventory_block = Block::bordered()
            .title(Line::from(" Inventario ").bold().yellow())
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let mut inventory_lines: Vec<Line> = vec![Line::from("")];

        if self.player_inventory.is_empty() {
            inventory_lines.push(Line::from("Il tuo inventario è vuoto.").centered());
        } else {
            for (i, item) in self.player_inventory.iter().enumerate() {
                inventory_lines.push(Line::from(format!("• {}", item)).white());
            }
        }

        inventory_lines.push(Line::from(""));
        inventory_lines.push(
            Line::from("Premi (B) per tornare indietro.")
                .dark_gray()
                .centered(),
        );

        frame.render_widget(
            Paragraph::new(inventory_lines)
                .block(inventory_block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn render_mercy(&mut self, frame: &mut Frame, area: Rect) {
        let title_text = match self.mercy_outcome {
            Some(true) => Line::from("Pietà Accettata!").bold().green().centered(),
            Some(false) => Line::from("Pietà Rifiutata!").bold().red().centered(),
            None => Line::from("Tenta la Pietà...").bold().blue().centered(),
        };

        let message_text = match self.mercy_outcome {
            Some(true) => vec![
                Line::from(""),
                Line::from("Il brigante si ritira.").green().centered(),
                Line::from("La battaglia è terminata pacificamente.")
                    .green()
                    .centered(),
                Line::from(""),
                Line::from("Premi Invio per continuare")
                    .dark_gray()
                    .centered(),
            ],
            Some(false) => vec![
                Line::from(""),
                Line::from("Il brigante rifiuta la tua pietà!")
                    .red()
                    .centered(),
                Line::from("La battaglia continua.").red().centered(),
                Line::from(""),
                Line::from("Premi Invio per continuare")
                    .dark_gray()
                    .centered(),
            ],
            None => vec![
                Line::from(""),
                Line::from("Offri pietà al tuo nemico?").centered(),
                Line::from(""),
                Line::from("Premi Invio per offrire pietà")
                    .dark_gray()
                    .centered(),
                Line::from("Premi (B) per tornare alla battaglia")
                    .dark_gray()
                    .centered(),
            ],
        };

        frame.render_widget(
            Paragraph::new(message_text)
                .block(Block::bordered().title(title_text))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_game_over(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from("Game Over!").bold().red().centered();
        let game_over_text = vec![
            Line::from(""),
            Line::from("Hai perso!").red().bold().centered(),
            Line::from(""),
            Line::from("La tua avventura è giunta al termine...").centered(),
            Line::from(""),
            Line::from("Premi (E) per uscire").dark_gray().centered(),
        ];

        frame.render_widget(
            Paragraph::new(game_over_text)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_heal(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from("Centro di Guarigione").bold().green().centered();
        let heal_text = vec![
            Line::from(""),
            Line::from("Ti trovi in un luogo sacro di guarigione.").centered(),
            Line::from(""),
            Line::from(format!("Salute attuale: {:.0}", self.player_health))
                .yellow()
                .centered(),
            Line::from(""),
            Line::from("Premi (H) per guarire | (M) per Menu")
                .dark_gray()
                .centered(),
        ];

        frame.render_widget(
            Paragraph::new(heal_text)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_minigame(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from("Minigame Matematico")
            .bold()
            .magenta()
            .centered();
        let minigame_text = vec![
            Line::from(""),
            Line::from("Risolvi il teorema di Pitagora:").centered(),
            Line::from(""),
            Line::from("a² + b² = c²").yellow().bold().centered(),
            Line::from(""),
            Line::from("Se a = 3 e b = 4, quanto vale c?").centered(),
            Line::from(""),
            Line::from("Premi (M) per Menu").dark_gray().centered(),
        ];

        frame.render_widget(
            Paragraph::new(minigame_text)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_test(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from("Modalità Test").bold().cyan().centered();
        let test_text = vec![
            Line::from(""),
            Line::from("Questa è la modalità test").centered(),
            Line::from(""),
            Line::from("Qui puoi provare le funzionalità del gioco").centered(),
            Line::from(""),
            Line::from("Premi Esc per uscire").dark_gray().centered(),
        ];

        frame.render_widget(
            Paragraph::new(test_text)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.logic_quit(),
            _ => match self.game_state {
                GameState::Minigame => match key.code {
                    KeyCode::Char('M') | KeyCode::Char('m') => {
                        self.game_state = GameState::MainMenu
                    }

                    _ => {}
                },

                GameState::Test => match key.code {
                    KeyCode::Char('M') | KeyCode::Char('m') => {
                        self.game_state = GameState::MainMenu
                    }

                    _ => {}
                },

                GameState::Fight => match key.code {
                    // Changed to GameState::Battle
                    KeyCode::Left => {
                        self.selected_fight_option = match self.selected_fight_option {
                            FightOption::Attack => FightOption::Mercy, // Loop from Attack to Run
                            FightOption::Defend => FightOption::Attack,
                            FightOption::Inventory => FightOption::Defend,
                            FightOption::Mercy => FightOption::Inventory,
                        };
                    }
                    KeyCode::Right => {
                        self.selected_fight_option = match self.selected_fight_option {
                            FightOption::Attack => FightOption::Defend,
                            FightOption::Defend => FightOption::Inventory,
                            FightOption::Inventory => FightOption::Mercy,
                            FightOption::Mercy => FightOption::Attack, // Loop from Run to Attack
                        };
                    }
                    KeyCode::Char('G') => self.logic_hook(),
                    KeyCode::Char('J') => self.logic_jab(),
                    KeyCode::Char('M') => self.logic_montante(),

                    _ => {}
                },

                GameState::Mercy => match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        // If mercy was successful, go to Story, otherwise return to Battle
                        match self.mercy_outcome {
                            Some(true) => self.game_state = GameState::Story,
                            Some(false) => self.game_state = GameState::Battle,
                            None => {} // Should not happen, but no action if outcome not determined
                        }
                        self.mercy_outcome = None; // Reset mercy outcome after action
                    }
                    KeyCode::Char('B') | KeyCode::Char('b') => {
                        self.game_state = GameState::Battle;
                        self.mercy_outcome = None; // Reset mercy outcome if returning to battle
                    }
                    _ => {}
                },
                GameState::GameOver => match key.code {
                    KeyCode::Char('E') | KeyCode::Char('e') => self.logic_quit(),
                    _ => {}
                },

                GameState::Heal => match key.code {
                    KeyCode::Char('M') | KeyCode::Char('m') => {
                        self.game_state = GameState::MainMenu
                    }
                    KeyCode::Char('H') | KeyCode::Char('h') => {
                        self.logic_heal();
                    }
                    _ => {}
                },

                GameState::MainMenu => match key.code {
                    KeyCode::Char('S') | KeyCode::Char('s') => {
                        self.game_state = GameState::Story;
                        self.add_message("Iniziata la storia di Pitagora!".to_string());
                    }
                    KeyCode::Char('E') | KeyCode::Char('e') => {
                        self.game_state = GameState::GameOver
                    }
                    KeyCode::Char('T') | KeyCode::Char('t') => self.game_state = GameState::Test,
                    KeyCode::Char('H') | KeyCode::Char('h') => {
                        self.game_state = GameState::Heal;
                        self.add_message("Sei entrato nel centro di guarigione.".to_string());
                    }
                    KeyCode::Char('I') | KeyCode::Char('i') => {
                        self.previous_game_state = Some(self.game_state);
                        self.game_state = GameState::Inventory;
                    }
                    KeyCode::Char('W') | KeyCode::Char('w') => {
                        self.game_state = GameState::Shop;
                        self.add_message("Benvenuto nel negozio!".to_string());
                    }
                    _ => {}
                },
                GameState::Story => match key.code {
                    KeyCode::Char('C') | KeyCode::Char('c') => {
                        match self.story_state {
                            StoryState::First => {
                                self.story_state = StoryState::Second;
                                self.add_message("Continui il viaggio di Pitagora...".to_string());
                                self.player_player_place = Places::Samos;
                            }
                            StoryState::Second => {
                                self.story_state = StoryState::Third;
                                self.add_message("Il destino di Pitagora si compie...".to_string());
                                self.player_player_place = Places::Babilonia;
                            }
                            StoryState::Third => {
                                // Story is complete, maybe unlock something or provide different options

                                self.add_message(
                                    "La storia è completa! Ora puoi esplorare liberamente."
                                        .to_string(),
                                );

                                self.player_player_place = Places::Crotone;
                            }
                        }
                    }
                    KeyCode::Char('H') | KeyCode::Char('h') => self.game_state = GameState::Heal,
                    KeyCode::Char('V') | KeyCode::Char('v') => {
                        self.game_state = GameState::GameOver
                    }
                    KeyCode::Char('B') | KeyCode::Char('b') => {
                        self.game_state = GameState::Battle;
                        self.add_message("Una battaglia sta per iniziare!".to_string());
                    }
                    KeyCode::Char('M') | KeyCode::Char('m') => {
                        self.game_state = GameState::MainMenu
                    }
                    _ => {}
                },

                GameState::Battle => match key.code {
                    KeyCode::Left => {
                        self.selected_fight_option = match self.selected_fight_option {
                            FightOption::Attack => FightOption::Mercy,
                            FightOption::Defend => FightOption::Attack,
                            FightOption::Inventory => FightOption::Defend,
                            FightOption::Mercy => FightOption::Inventory,
                        };
                    }
                    KeyCode::Right => {
                        self.selected_fight_option = match self.selected_fight_option {
                            FightOption::Attack => FightOption::Defend,
                            FightOption::Defend => FightOption::Inventory,
                            FightOption::Inventory => FightOption::Mercy,
                            FightOption::Mercy => FightOption::Attack,
                        };
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => match self.selected_fight_option {
                        FightOption::Attack => self.logic_attack(),
                        FightOption::Defend => self.logic_defend(),
                        FightOption::Inventory => {
                            self.previous_game_state = Some(self.game_state);
                            self.game_state = GameState::Inventory
                        }
                        FightOption::Mercy => {
                            self.game_state = GameState::Mercy;
                            self.logic_mercy();
                        }
                    },

                    KeyCode::Char(' ') => {
                        if self.is_boss_battle
                            && self.boss_dialogue_index < self.boss_dialogue.len()
                        {
                            self.boss_dialogue_index += 1;
                        } else {
                            // Normal battle action handling
                            match self.selected_fight_option {
                                FightOption::Attack => {
                                    if self.is_boss_battle {
                                        self.logic_boss_attack();
                                    } else {
                                        self.logic_attack();
                                    }
                                } //

                                _ => {}
                            }
                        }
                    }

                    _ => {}
                },
                GameState::Shop => match key.code {
                    KeyCode::Left => {
                        self.selected_shop_option = match self.selected_shop_option {
                            ShopOption::Buy => ShopOption::Exit,
                            ShopOption::Sell => ShopOption::Buy,
                            ShopOption::Inventory => ShopOption::Sell,
                            ShopOption::Exit => ShopOption::Inventory,
                        };
                    }
                    KeyCode::Right => {
                        self.selected_shop_option = match self.selected_shop_option {
                            ShopOption::Buy => ShopOption::Sell,
                            ShopOption::Sell => ShopOption::Inventory,
                            ShopOption::Inventory => ShopOption::Exit,
                            ShopOption::Exit => ShopOption::Buy,
                        };
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        match self.selected_shop_option {
                            ShopOption::Buy => self.logic_buy(),
                            ShopOption::Sell => self.logic_sell(),
                            ShopOption::Inventory => {
                                self.previous_game_state = Some(self.game_state); // Save current state
                                self.game_state = GameState::Inventory
                            }
                            ShopOption::Exit => self.logic_quit(),
                        }
                    }
                    KeyCode::Char('M') | KeyCode::Char('m') => {
                        self.game_state = GameState::MainMenu
                    }
                    _ => {}
                },
                GameState::Inventory => match key.code {
                    KeyCode::Char('B') | KeyCode::Char('b') => {
                        // Restore the previous game state if available, otherwise go to MainMenu
                        if let Some(prev_state) = self.previous_game_state.take() {
                            self.game_state = prev_state;
                        } else {
                            self.game_state = GameState::MainMenu;
                        }
                    }
                    _ => {}
                },
            },
        }
    }
    fn logic_attack(&mut self) {
        self.enemy_health -= self.player_dmg;
        if self.enemy_health <= 0.0 {
            self.enemy_is_alive = false;
            self.game_state = GameState::Story; // Example: go back to story after defeating enemy
        } else {
            self.player_health -= self.enemy_dmg;
            if self.player_health <= 0.0 {
                self.game_state = GameState::GameOver;
            }
        }
    }
    fn logic_defend(&mut self) {
        self.player_health -= self.enemy_dmg * 0.5; // Example: 50% damage reduction when defending
        if self.player_health <= 0.0 {
            self.game_state = GameState::GameOver;
        }
    }
    fn logic_heal(&mut self) {
        self.player_health = self.player_health * 1.2;

        if self.player_health > 100.0 {
            self.player_health = 100.0
        }
    }
    fn logic_story(&mut self) {}
    fn logic_fight(&mut self) {}
    fn logic_hook(&mut self) {}
    fn logic_jab(&mut self) {
        self.enemy_health = self.enemy_health - self.player_dmg * (self.player_strength / 2.0)
    }
    fn logic_montante(&mut self) {}
    fn logic_minigame(&mut self) {}
    fn logic_buy(&mut self) {
        self.player_inventory
            .push("Pozione della Salute".to_string());
    }
    fn logic_sell(&mut self) {
        if !self.player_inventory.is_empty() {
            self.player_inventory.pop(); // Remove the last item as a simple example
        }
    }
    fn logic_game_over(&mut self) {
        self.running = false;
    }
    fn logic_shop(&mut self) {}
    fn logic_mercy(&mut self) {
        let secret_number: i32 = rand::thread_rng().gen_range(1..=10);
        let player_mercy_chance: i32 = 7; // This could be based on player stats or charisma
        self.mercy_outcome = Some(player_mercy_chance >= secret_number);
    }
    fn logic_quit(&mut self) {
        self.running = false;
    }
}
