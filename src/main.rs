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
use ratatui::{style::palette::material::GRAY, symbols::border};
use std::time::{Duration, Instant};
use std::{cmp::Ordering, default}; // Import Instant and Duration

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug, Default, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FightOption {
    Attack,
    Defend,
    Inventory,
    Mercy,
}
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Places {
    #[default]
    Samo,
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

impl Default for FightOption {
    fn default() -> Self {
        FightOption::Attack // Default to Attack
    }
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    pub game_state: GameState,
    pub counter: u32,
    pub player_strength: f64,
    pub player_dmg: f64,
    pub player_health: f64,
    pub player_def: f64,
    pub player_xp: f64,
    pub player_lvl: f64,
    pub player_player_place: String,
    pub player_heal_value: f64,
    pub player_heal_factor: f64,
    pub player_xp_factor: f64,
    pub player_inventory: Vec<String>,
    pub selected_fight_option: FightOption,
    pub enemy_health: f64,
    pub enemy_strength: f64,
    pub enemy_dmg: f64,
    pub enemy_strength_factor: f64,
    pub enemy_heal: f64, // <--- NEW FIELD
}

#[allow(deprecated)]
#[allow(dead_code)]
impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            // Initialize player health
            player_health: 100.0,  // Initialize enemy health
            player_strength: 10.0, // Example initial values
            player_dmg: 15.0,
            player_def: 5.0,
            player_xp: 0.0,
            player_lvl: 1.0,
            player_player_place: "Samo".to_string(),
            player_heal_value: 20.0,
            player_heal_factor: 1.0,
            player_xp_factor: 1.0,

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
            GameState::Fight => self.render_fight(frame),
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

        let attack_text = "hi".to_string();
        let option_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(frame.area());

        // Helper to get styled text based on selection
        let get_option_style = |option: FightOption, current_selection: FightOption| {
            if option == current_selection {
                Style::default().yellow().bold() // Highlight selected option
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
            parent_area.x + parent_area.width / 4 - popup_width + 5,
            parent_area.y + parent_area.height / 2 - popup_height + 10,
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
            parent_area.x + parent_area.width / 4 - popup_width + 24,
            parent_area.y + parent_area.height / 2 - popup_height + 10,
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

    fn render_fight(&mut self, frame: &mut Frame) {
        let title = Line::from(format!(" Oh No! Hai incontrato il boss "))
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

    fn render_shop(&mut self, frame: &mut Frame) {
        let title = Line::from(format!(" Negozio di {} ", self.player_player_place))
            .bold()
            .blue()
            .centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        let text2 = "(E) Esci (C) Continua  ";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        );

        frame.render_widget(
            Paragraph::new(text2).block(Block::bordered()).centered(),
            frame.area(),
        );
    }
    fn render_mercy(&mut self, frame: &mut Frame) {
        let title = Line::from(format!(" Hai ")).bold().blue().centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        let text2 = "(E) Esci (C) Continua  ";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        );

        frame.render_widget(
            Paragraph::new(text2).block(Block::bordered()).centered(),
            frame.area(),
        );
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
                    KeyCode::Char('I') => self.game_state = GameState::Inventory,
                    KeyCode::Char('W') => self.game_state = GameState::Shop,

                    _ => {}
                },

                GameState::Battle => match key.code {
                    KeyCode::Char('G') => self.game_state = GameState::Heal,
                    KeyCode::Char('I') => self.game_state = GameState::Inventory,
                    KeyCode::Char('W') => self.game_state = GameState::Shop,

                    _ => {}
                },
                // Making the values for Story
                GameState::Story => match key.code {
                    KeyCode::Char('H') => self.game_state = GameState::Heal,
                    KeyCode::Char('V') => self.game_state = GameState::GameOver,
                    // KeyCode::Char('C') => self.game_state = GameState::Continue,
                    KeyCode::Char('F') => self.game_state = GameState::Fight,
                    KeyCode::Char('M') => self.game_state = GameState::MainMenu,

                    _ => {}
                },
                GameState::Test => match key.code {
                    _ => {}
                },

                GameState::Fight => match key.code {
                    KeyCode::Enter => {}

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
                            FightOption::Inventory => self.game_state = GameState::Inventory,
                            FightOption::Mercy => {
                                // Implement run logic
                                // For now, just go back to story. You might add a success/fail chance.
                                self.game_state = GameState::Story;
                            }
                        }
                    }

                    _ => {}
                },
                GameState::Minigame => match key.code {
                    KeyCode::Char('M') => self.game_state = GameState::MainMenu,

                    _ => {}
                },

                GameState::Mercy => match key.code {
                    KeyCode::Char('G') => self.logic_mercy(),

                    _ => {}
                },

                GameState::GameOver => match key.code {
                    KeyCode::Char('E') | KeyCode::Char('e') => self.logic_quit(),
                    _ => {}
                },

                GameState::Heal => match key.code {
                    KeyCode::Char('M') => self.game_state = GameState::MainMenu,
                    _ => {}
                },

                GameState::Shop => match key.code {
                    KeyCode::Char('M') => self.game_state = GameState::MainMenu,
                    KeyCode::Enter => self.logic_shop(),

                    _ => {}
                },

                GameState::Inventory => match key.code {
                    KeyCode::Up => self.game_state = GameState::Fight,
                    _ => {}
                },
            },
        }
    }

    fn logic_attack(&mut self) {}

    fn logic_defend(&mut self) {}

    fn logic_heal(&mut self) {}

    fn logic_story(&mut self) {}

    fn logic_fight(&mut self) {}

    fn logic_hook(&mut self) {}

    fn logic_jab(&mut self) {}

    fn logic_montante(&mut self) {}

    fn logic_minigame(&mut self) {}

    fn logic_game_over(&mut self) {
        self.running = false;
    }

    fn logic_shop(&mut self) {}

    fn logic_mercy(&mut self) {
        let secret_number: i32 = rand::thread_rng().gen_range(1..=10);

        let five: i32 = 7;

        match five.cmp(&secret_number) {
            Ordering::Less => self.game_state = GameState::GameOver,
            Ordering::Greater => self.game_state = GameState::Fight,
            Ordering::Equal => self.game_state = GameState::Story,
        }
    }

    fn logic_quit(&mut self) {
        self.running = false;
    }
}
