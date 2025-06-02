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
use serde::{Deserialize, Serialize}; // Import Serialize and Deserialize
use std::cmp::Ordering;
#[allow(unused)]
use std::fs; // Import file system module for saving/loading
#[allow(unused)]
use std::io;
#[allow(unused)]
use std::time::{Duration, Instant}; // Import io module for file operations

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    #[default]
    MainMenu,
    Story,
    // Fight, // Renamed to Battle
    Minigame,
    GameOver,
    Heal,
    Shop,
    Inventory,
    Test,
    Mercy,
    Battle, // New name for Fight
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

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default, Clone)] // Add Clone derive to App
pub struct App {
    /// Is the application running?
    running: bool,
    pub previous_game_state: Option<GameState>,
    pub saved_state_before_inventory: Option<Box<App>>, // Use Box<App> to avoid recursive type issues
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
    pub mercy_outcome: Option<bool>, // true for success, false for failure
}

#[allow(deprecated)]
#[allow(dead_code)]
impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            // Initialize player health
            player_health: 100.0, // Initialize enemy health
            player_strength: 1.0, // Example initial values
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
            saved_state_before_inventory: None, // Initialize the new field
            mercy_outcome: None,                // Initialize new field

            ..Self::default()
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    /// This is where you add new widgets. See the following resources for more information:

    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(1), // <-- This takes up all available space
                Constraint::Length(1),
            ])
            .split(frame.area());

        match self.game_state {
            GameState::MainMenu => self.render_main_menu(frame),
            GameState::Story => self.render_story(frame),
            // GameState::Fight => self.render_fight(frame), // Removed
            GameState::Minigame => self.render_minigame(frame),
            GameState::GameOver => self.render_game_over(frame),
            GameState::Heal => self.render_heal(frame),
            GameState::Shop => self.render_shop(frame),
            GameState::Inventory => self.render_inventory(frame),
            GameState::Test => self.render_popup_options(frame),
            GameState::Mercy => self.render_mercy(frame),
            GameState::Battle => self.render_battle(frame),
        }

        // let main_content_area = layout[0];
        let footer_area = layout[1];

        let footer_text = Line::from(format!(
            " HP: {:.0}/{:.0} | LVL: {:.0} | XP: {:.0} ",
            self.player_health,
            100.0 + (self.player_lvl - 1.0) * 20.0, // Example max HP
            self.player_lvl,
            self.player_xp
        ))
        .centered()
        .dark_gray();
        frame.render_widget(Paragraph::new(footer_text), footer_area);
    }

    fn render_battle(&mut self, frame: &mut Frame) {
        // 1. Render Enemy/Message Area
        let enemy_title = Line::from("Battaglia Imminente!").bold().red().centered();
        let enemy_text = format!(
            "Un Brigante selvaggio appare! \n\n\
                    Salute del Brigante: {:.0}\n\n\
                    La tua salute: {:.0}",
            self.enemy_health, self.player_health
        );
        frame.render_widget(
            Paragraph::new(enemy_text)
                .block(Block::bordered().title(enemy_title))
                .centered(),
            frame.area(),
        );

        // let attack_text = "hi".to_string();
        // let option_chunks = Layout::default()
        //    .direction(Direction::Horizontal)
        //    .constraints([
        //        Constraint::Percentage(33),
        //        Constraint::Percentage(33),
        //        Constraint::Percentage(34),
        //    ])
        //    .split(frame.area());

        // Helper to get styled text based on selection
        let get_option_style = |option: FightOption, current_selection: FightOption| {
            if option == current_selection {
                Style::default().bg(Color::Yellow).black().bold() // Highlight selected option
            } else {
                Style::default().white()
            }
        };

        let parent_area = frame.area();

        let popup_width = 17;
        let popup_height = 3; // Increased height to comfortably fit message + prompt + padding

        // ATTACK
        // ATTACK POPUP
        let popup_area_attack = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 12 * popup_width / 40,
            parent_area.y + parent_area.height / 2 - popup_height + 14 * popup_height / 4,
            popup_width,
            popup_height,
        );
        let attack_text = Line::from("ATTACCA")
            .style(get_option_style(
                FightOption::Attack,
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(attack_text)
                .block(Block::bordered())
                .centered(),
            popup_area_attack,
        );

        // DEFEND POPUP
        let popup_area_defend = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 4 * popup_width / 230 * 100,
            parent_area.y + parent_area.height / 2 - popup_height + 14 * popup_height / 4,
            popup_width,
            popup_height,
        );
        let defend_text = Line::from("DIFENDI")
            .style(get_option_style(
                FightOption::Defend,
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(defend_text)
                .block(Block::bordered())
                .centered(),
            popup_area_defend,
        );

        // INVENTORY POPUP (You had a commented out inventory render previously, so adapting this)
        let popup_area_inventory = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 43,
            parent_area.y + parent_area.height / 2 - popup_height + 10,
            popup_width,
            popup_height,
        );
        let inventory_text = Line::from("INVENTARIO")
            .style(get_option_style(
                // Assuming you want to associate Inventory with an existing FightOption or add a new one.
                // For now, I'll link it to Defend as a placeholder or you might need a new enum variant.
                // If "Inventory" isn't a FightOption, you'll need to adjust `get_option_style` or its usage.
                FightOption::Inventory, // Placeholder, adjust as needed. Maybe add FightOption::Inventory
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(inventory_text)
                .block(Block::bordered())
                .centered(),
            popup_area_inventory,
        );

        // MERCY / PIETA' POPUP (Renamed Run to Mercy for consistency with FightOption::Run)
        let popup_area_mercy = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 62,
            parent_area.y + parent_area.height / 2 - popup_height + 10,
            popup_width,
            popup_height,
        );
        let mercy_text = Line::from("PIETA'")
            .style(get_option_style(
                FightOption::Mercy, // Corresponds to FightOption::Mercy(Mercy)
                self.selected_fight_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(mercy_text)
                .block(Block::bordered())
                .centered(),
            popup_area_mercy,
        );
    }

    fn render_popup_options(&mut self, frame: &mut Frame) {
        let parent_area = frame.size(); // Or main_content_area if you prefer

        // Calculate a smaller Rect in the middle
        let popup_width = 60; // Example width
        let popup_height = 5; // Example height

        let popup_area = Rect::new(
            parent_area.width / 2 - popup_width / 2,
            parent_area.height / 2 - popup_height / 2,
            popup_width,
            popup_height,
        );

        // You would then render your options into this popup_area,
        // potentially using another Layout to split it for the boxes.
        // For a simple example, just a block:
        frame.render_widget(
            Block::bordered().title("Choose Your Action").magenta(),
            popup_area,
        );
    }

    fn render_defend(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    fn render_minigame(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    fn render_inventory(&mut self, frame: &mut Frame) {
        let parent_area = frame.size();

        // Calculate a smaller Rect in the middle for the inventory popup
        let popup_width = 40;
        let popup_height = 10; // Adjust height based on how many items you expect to display

        let popup_area = Rect::new(
            parent_area.width / 2 - popup_width / 2,
            parent_area.height / 2 - popup_height / 2,
            popup_width,
            popup_height,
        );

        // Create the block for the inventory popup
        let inventory_block = Block::bordered()
            .title(Line::from(" Inventory ").bold().yellow())
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        // Prepare the text for inventory items
        let mut inventory_lines: Vec<Line> = Vec::new();
        if self.player_inventory.is_empty() {
            inventory_lines.push(Line::from("Your inventory is empty."));
        } else {
            for (i, item) in self.player_inventory.iter().enumerate() {
                // You can add logic here to highlight selected item if you implement selection within inventory
                inventory_lines.push(Line::from(format!("- {}", item)).white());
            }
        }
        inventory_lines.push(Line::from("").white()); // Add an empty line for spacing
        inventory_lines.push(Line::from("Press (B) to go Back.").dark_gray()); // Changed M to B for consistency

        let inventory_paragraph = Paragraph::new(inventory_lines)
            .block(inventory_block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(inventory_paragraph, popup_area);
    }

    fn render_game_over(&mut self, frame: &mut Frame) {
        let title = Line::from("Game Over!").bold().blue().centered();
        let text = "Hai perso! Premi (E) per uscire.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        );
    }

    fn render_heal(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    fn render_story(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let text = "La Storia di Pitagora\n\n\
            Premi (C) per continuare.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    fn render_main_menu(&mut self, frame: &mut Frame) {
        let title = Line::from(format!(" Menù Principale "))
            .bold()
            .blue()
            .centered();
        let text1 = Line::from("\n Salve, Avventuriero! \n ");

        let text2 = Line::from(format!("\n  Ti trovi nel menù principale.")).blue();
        let text3 = Line::from(format!("\nCosa desideri fare?")).bold();

        frame.render_widget(
            Paragraph::new(text1 + text2 + text3)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    // Removed render_fight as it's now render_battle
    // fn render_fight(&mut self, frame: &mut Frame) {
    //     let title = Line::from(format!(
    //         " Hai deciso di combattere il boss! Cosa intendi fare?"
    //     ))
    //     .bold()
    //     .blue()
    //     .centered();
    // }

    fn render_shop(&mut self, frame: &mut Frame) {
        // Determine the shop name based on the player's current place
        let shop_name = match self.player_player_place {
            Places::Samos => "Negozio di Samos",
            Places::SabbiaSamos => "Bancarella nella Sabbia di Samos",
            Places::Tiro => "Emporio di Tiro",
            Places::ColonneTiro => "Mercato delle Colonne di Tiro",
            Places::Crotone => "Bottega di Crotone",
            Places::ScuolaCrotone => "Spaccio della Scuola di Crotone",
            Places::Babilonia => "Mercante Babilonese",
            Places::BabiloniaBoss => "Tesori del Boss di Babilonia", // Maybe a special shop after defeating the boss?
            Places::Olimpia => "Bazar Olimpico",
            Places::Syros => "Mercato di Syros",
            Places::Mileto => "Bottega di Mileto",
            // Add more cases for each `Places` variant as needed
            _ => "Negozio Errante", // Default or fallback name if you add new places later
        };

        let title = Line::from(format!(" {} ", shop_name))
            .bold()
            .blue()
            .centered();
        let text = "Benvenuto! Cosa desideri acquistare?\n\n\
                    (W) Compra Pozione (A) Compra Arma (D) Compra Armatura\n\
                    Premi `Esc` o `q` per uscire."; // Updated text to be more shop-like
        let text2 = "(M) Torna al Menù Principale "; // Moved the return to main menu here for clarity

        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        );

        // You might want to adjust the area for text2 so it doesn't overlap the main shop text
        // For now, it's rendering over the same area, which might look cluttered.
        // Consider adding a layout to split the area if you want separate sections for text and controls.
        let get_option_style = |option: ShopOption, current_selection: ShopOption| {
            if option == current_selection {
                Style::default().bg(Color::Yellow).black().bold() // Highlight selected option
            } else {
                Style::default().white()
            }
        };

        let parent_area = frame.area();

        let popup_width = 17;
        let popup_height = 3; // Increased height to comfortably fit message + prompt + padding

        // ATTACK
        // ATTACK POPUP
        let popup_area_buy = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 12 * popup_width / 40,
            parent_area.y + parent_area.height / 2 - popup_height + 14 * popup_height / 4,
            popup_width,
            popup_height,
        );
        let buy_text = Line::from("COMPRA")
            .style(get_option_style(ShopOption::Buy, self.selected_shop_option))
            .centered();
        frame.render_widget(
            Paragraph::new(buy_text).block(Block::bordered()).centered(),
            popup_area_buy,
        );

        // DEFEND POPUP
        let popup_area_sell = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 4 * popup_width / 2,
            parent_area.y + parent_area.height / 2 - popup_height + 14 * popup_height / 4,
            popup_width,
            popup_height,
        );
        let defend_sell = Line::from("VENDI")
            .style(get_option_style(
                ShopOption::Sell,
                self.selected_shop_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(defend_sell)
                .block(Block::bordered())
                .centered(),
            popup_area_sell,
        );

        // INVENTORY POPUP (You had a commented out inventory render previously, so adapting this)
        let popup_area_inventory = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 43,
            parent_area.y + parent_area.height / 2 - popup_height + 10,
            popup_width,
            popup_height,
        );
        let inventory_text = Line::from("INVENTARIO")
            .style(get_option_style(
                // Assuming you want to associate Inventory with an existing FightOption or add a new one.
                // For now, I'll link it to Defend as a placeholder or you might need a new enum variant.
                // If "Inventory" isn't a FightOption, you'll need to adjust `get_option_style` or its usage.
                ShopOption::Inventory, // Placeholder, adjust as needed. Maybe add FightOption::Inventory
                self.selected_shop_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(inventory_text)
                .block(Block::bordered())
                .centered(),
            popup_area_inventory,
        );

        let popup_area_exit = Rect::new(
            parent_area.x + parent_area.width / 4 - popup_width + 20,
            parent_area.y + parent_area.height / 2 - popup_height + 10,
            popup_width,
            popup_height,
        );

        let exit_text = Line::from("ESCI")
            .style(get_option_style(
                ShopOption::Exit,
                self.selected_shop_option,
            ))
            .centered();
        frame.render_widget(
            Paragraph::new(exit_text)
                .block(Block::bordered())
                .centered(),
            popup_area_exit,
        );
    }
    fn render_mercy(&mut self, frame: &mut Frame) {
        let title_text = match self.mercy_outcome {
            Some(true) => Line::from("Pietà Accettata!").bold().green().centered(),
            Some(false) => Line::from("Pietà Rifiutata!").bold().red().centered(),
            None => Line::from("Tenta la Pietà...").bold().blue().centered(), // Initial state
        };

        let message_text = match self.mercy_outcome {
            Some(true) => {
                Line::from("Il brigante si ritira. La battaglia è terminata pacificamente.").white()
            }
            Some(false) => {
                Line::from("Il brigante rifiuta la tua pietà! La battaglia continua.").white()
            }
            None => Line::from("Premi (Invio) per offrire pietà o (B) per tornare alla battaglia.")
                .dark_gray(),
        };

        let mercy_block = Block::bordered().title(title_text);

        let paragraph = Paragraph::new(message_text)
            .block(mercy_block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, frame.area());
    }

    /// Reads the crossterm events and updates the state of [`App`].
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.logic_quit(),
            // Add other key handlers here.
            _ => match self.game_state {
                GameState::MainMenu => match key.code {
                    KeyCode::Char('S') => self.game_state = GameState::Story,
                    KeyCode::Char('E') | KeyCode::Char('e') => {
                        self.game_state = GameState::GameOver
                    }
                    KeyCode::Char('T') => self.game_state = GameState::Test,
                    // Travelling
                    //KeyCode::Char('V') => self.game_state = GameState::GameOver,
                    KeyCode::Char('H') => self.game_state = GameState::Heal,
                    KeyCode::Char('I') => {
                        self.previous_game_state = Some(self.game_state); // Save current state
                        self.game_state = GameState::Inventory;
                    }
                    KeyCode::Char('W') => self.game_state = GameState::Shop,

                    _ => {}
                },

                // GameState::Fight => match key.code { // Changed to GameState::Battle
                //     KeyCode::Left => {
                //         self.selected_fight_option = match self.selected_fight_option {
                //             FightOption::Attack => FightOption::Mercy, // Loop from Attack to Run
                //             FightOption::Defend => FightOption::Attack,
                //             FightOption::Inventory => FightOption::Defend,
                //             FightOption::Mercy => FightOption::Inventory,
                //         };
                //     }
                //     KeyCode::Right => {
                //         self.selected_fight_option = match self.selected_fight_option {
                //             FightOption::Attack => FightOption::Defend,
                //             FightOption::Defend => FightOption::Inventory,
                //             FightOption::Inventory => FightOption::Mercy,
                //             FightOption::Mercy => FightOption::Attack, // Loop from Run to Attack
                //         };
                //     }
                //     KeyCode::Char('G') => self.logic_hook(),
                //     KeyCode::Char('J') => self.logic_jab(),
                //     KeyCode::Char('M') => self.logic_montante(),

                //     _ => {}
                // },
                // Making the values for Story
                GameState::Story => match key.code {
                    KeyCode::Char('H') => self.game_state = GameState::Heal,
                    KeyCode::Char('V') => self.game_state = GameState::GameOver,
                    // KeyCode::Char('C') => self.game_state = GameState::Continue,
                    KeyCode::Char('B') => self.game_state = GameState::Battle,
                    KeyCode::Char('M') => self.game_state = GameState::MainMenu,

                    _ => {}
                },
                GameState::Test => match key.code {
                    _ => {}
                },

                GameState::Battle => match key.code {
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
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        // User pressed Enter or Space to select
                        match self.selected_fight_option {
                            FightOption::Attack => self.logic_attack(),
                            FightOption::Defend => self.logic_defend(),
                            FightOption::Inventory => {
                                self.previous_game_state = Some(self.game_state); // Save current state
                                self.game_state = GameState::Inventory
                            }
                            FightOption::Mercy => {
                                // Implement run logic
                                // For now, just go back to story. You might add a success/fail chance.
                                self.game_state = GameState::Mercy; // Go to Mercy state to display outcome
                                self.logic_mercy();
                            }
                        }
                    }

                    _ => {}
                },
                GameState::Minigame => match key.code {
                    KeyCode::Char('M') | KeyCode::Char('m') => {
                        self.game_state = GameState::MainMenu
                    }

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

                GameState::Shop => match key.code {
                    KeyCode::Left => {
                        self.selected_shop_option = match self.selected_shop_option {
                            ShopOption::Buy => ShopOption::Exit, // Loop from Attack to Run
                            ShopOption::Sell => ShopOption::Buy,
                            ShopOption::Inventory => ShopOption::Sell,
                            ShopOption::Exit => ShopOption::Inventory,
                        };
                    }
                    KeyCode::Right => {
                        self.selected_shop_option = match self.selected_shop_option {
                            ShopOption::Buy => ShopOption::Sell, // Loop from Attack to Run
                            ShopOption::Sell => ShopOption::Inventory,
                            ShopOption::Inventory => ShopOption::Exit,
                            ShopOption::Exit => ShopOption::Buy,
                        };
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        // User pressed Enter or Space to select
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
        // Example attack logic: Player attacks enemy
        self.enemy_health -= self.player_dmg;
        if self.enemy_health <= 0.0 {
            self.enemy_is_alive = false;
            // Transition to a victory state or back to story/main menu
            self.game_state = GameState::Story; // Example: go back to story after defeating enemy
        } else {
            // Enemy attacks back
            self.player_health -= self.enemy_dmg;
            if self.player_health <= 0.0 {
                self.game_state = GameState::GameOver;
            }
        }
    }

    fn logic_defend(&mut self) {
        // Example defend logic: Reduce incoming damage for one turn
        // This is a simplified example. You might want to implement a temporary defense boost.
        // For now, let's just say defending passes a turn.
        self.player_health -= self.enemy_dmg * 0.5; // Example: 50% damage reduction when defending
        if self.player_health <= 0.0 {
            self.game_state = GameState::GameOver;
        }
    }

    fn logic_heal(&mut self) {}

    fn logic_story(&mut self) {}

    // Removed logic_fight as it's now logic_battle
    // fn logic_fight(&mut self) {}

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

        // The state transition will now happen in `on_key_event` after the user acknowledges the outcome
        // No direct state change here, just set the outcome.
    }

    fn logic_quit(&mut self) {
        self.running = false;
    }
}
