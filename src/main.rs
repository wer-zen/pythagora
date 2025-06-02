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
use std::cmp::Ordering;
#[allow(unused)]
use std::fs;
#[allow(unused)]
use std::io;
#[allow(unused)]
use std::time::{Duration, Instant};

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
    Samos,
    SabbiaSamos,
    Tiro,
    ColonneTiro,
    Crotone,
    ScuolaCrotone,
    Babilonia,
    #[default]
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
    pub message_log: Vec<String>, // New field for message history
}

#[allow(deprecated)]
#[allow(dead_code)]
impl App {
    pub fn new() -> Self {
        Self {
            player_health: 100.0,
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
            ..Self::default()
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
            GameState::Battle => self.render_battle(frame, area),
            GameState::Fight => self.render_battle(frame, area),
            GameState::Shop => self.render_shop(frame, area),
            GameState::Inventory => self.render_inventory(frame, area),
            GameState::Mercy => self.render_mercy(frame, area),
            GameState::GameOver => self.render_game_over(frame, area),
            GameState::Heal => self.render_heal(frame, area),
            GameState::Minigame => self.render_minigame(frame, area),
            GameState::Test => self.render_test(frame, area),
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
        let story_text = vec![
            Line::from(""),
            Line::from("Nel VI secolo a.C., il grande matematico Pitagora").centered(),
            Line::from("viaggiava attraverso il mondo mediterraneo,").centered(),
            Line::from("alla ricerca della conoscenza universale...").centered(),
            Line::from(""),
            Line::from("Premi (C) per continuare | (B) per Battaglia | (M) per Menu")
                .dark_gray()
                .centered(),
        ];

        frame.render_widget(
            Paragraph::new(story_text)
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
    fn logic_heal(&mut self) {}
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
