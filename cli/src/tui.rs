use std::io::{self, stdout, Stdout};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Alignment, Constraint, Direction, Layout},
    style::{
        palette::tailwind::{self, BLACK, GREEN, ORANGE, RED, YELLOW},
        Modifier, Style, Stylize,
    },
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Borders, List, ListItem, ListState,
    },
    Terminal,
};

use crate::{kanji::KanjiEntry, CliError};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct SelectionList {
    pub items: Vec<String>,
    pub state: ListState,
}

impl SelectionList {
    pub fn new(items: Vec<String>) -> SelectionList {
        SelectionList {
            items,
            state: ListState::default(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub trait CanHaveSelection {
    async fn selection_list(
        &mut self,
        options: Vec<String>,
        title: &str,
    ) -> Result<String, CliError>;
}

impl CanHaveSelection for Tui {
    async fn selection_list(
        &mut self,
        options: Vec<String>,
        title: &str,
    ) -> Result<String, CliError> {
        let mut decks = SelectionList::new(options);
        let mut is_exit = false;
        decks.state.select_first();
        loop {
            let outer_block = Block::new()
                .borders(Borders::NONE)
                .title_alignment(Alignment::Center)
                .title(title)
                .fg(tailwind::SLATE.c200)
                .bg(tailwind::BLUE.c950);
            let inner_block = Block::new()
                .borders(Borders::NONE)
                .fg(tailwind::SLATE.c200)
                .bg(tailwind::SLATE.c950);

            self.draw(|f| {
                let inner_area = outer_block.inner(f.size());
                let items: Vec<ListItem> = decks
                    .items
                    .iter()
                    .map(|i| ListItem::new(i.as_str()))
                    .collect();
                let list = List::new(items)
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true)
                    .block(inner_block)
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::REVERSED)
                            .fg(tailwind::BLUE.c300),
                    );
                f.render_widget(outer_block, f.size());
                f.render_stateful_widget(list, inner_area, &mut decks.state)
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Down {
                        decks.next()
                    }
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Up {
                        decks.previous()
                    }
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        is_exit = true;
                        crate::tui::restore()?;
                        break;
                    }
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                        crate::tui::restore()?;
                        break;
                    }
                }
            }
        }
        if !is_exit {
            let selected = decks.state.selected();
            match selected {
                Some(index) => return Ok(decks.items[index].clone()),
                None => return Err(CliError::Custom("Item Selection Menu Failure!".to_string())),
            }
        }
        Err(CliError::Custom("Item Selection Menu Failure!".to_string()))
    }
}

pub struct KanjiList {
    title: String,
    content: StatefulList,
}

pub struct StatefulList {
    state: ListState,
    items: Vec<KanjiEntry>,
    page: usize,
    page_count: u16,
}

impl StatefulList {
    fn with_items(items: Vec<KanjiEntry>) -> StatefulList {
        StatefulList {
            page_count: round_up_to_nearest_10(items.len().try_into().unwrap()) / 450,
            state: ListState::default(),
            items,
            page: 1,
        }
    }

    fn next(&mut self) {
        if self.page * 450 < self.items.len() {
            self.page += 1
        } else {
            self.page = 1
        }
    }

    fn previous(&mut self) {
        if self.page == 1 {
            self.page = self.page_count.into()
        } else {
            self.page -= 1
        }
    }
}

pub trait CanHaveKanjiList {
    async fn kanji_list(&mut self, options: Vec<KanjiEntry>, title: &str) -> Result<(), CliError>;
}

impl CanHaveKanjiList for Tui {
    async fn kanji_list(&mut self, options: Vec<KanjiEntry>, title: &str) -> Result<(), CliError> {
        let mut entries = StatefulList::with_items(options);
        let mut is_exit: bool = false;
        loop {
            let title = Title::from(title.bold());
            let instructions = Title::from(Line::from(vec![
                format!("Page {}/{}", entries.page, entries.page_count).into(),
                " Previous Page ".into(),
                "<Left>".blue().bold(),
                " Next Page ".into(),
                "<Right>".blue().bold(),
                " Quit ".into(),
                "<q>".blue().bold(),
            ]));
            let block = Block::new()
                .title(title.alignment(Alignment::Center))
                .title(
                    instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom),
                );

            let inner_block = Block::new()
                .borders(Borders::NONE)
                .fg(tailwind::SLATE.c200)
                .bg(tailwind::SLATE.c950);

            let start = (entries.page - 1) * 450;
            let end = (start + 450).min(entries.items.len());
            let content = &entries.items[start..end];
            self.draw(|f| {
                let inner_area = block.inner(f.size());
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                    ])
                    .split(inner_area);

                f.render_widget(block, f.size());

                for i in 1..16 {
                    let mut start = (i - 1) * 30;
                    let mut end = (start + 30).min(content.len());
                    if content.len() < start {
                        return;
                    }
                    let list =
                        List::new(content[start..end].iter().map(|item| item.to_list_item()))
                            .highlight_symbol(">>")
                            .repeat_highlight_symbol(true)
                            /* .block(inner_block)*/
                            .highlight_style(
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .add_modifier(Modifier::REVERSED)
                                    .fg(tailwind::BLUE.c300),
                            );
                    f.render_widget(list, layout[i - 1]);
                }
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Right {
                        entries.next()
                    }
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Left {
                        entries.previous()
                    }
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        is_exit = true;
                        crate::tui::restore()?;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

fn round_up_to_nearest_10(n: u16) -> u16 {
    (n + 449) / 450 * 450
}

pub trait IntoListItem<'a> {
    fn to_list_item(&self) -> ListItem<'a>;
}

impl<'a> IntoListItem<'a> for KanjiEntry {
    fn to_list_item(&self) -> ListItem<'a> {
        let bg_color = match self.level {
            0 => RED.c600,
            1 => RED.c400,
            2 => ORANGE.c400,
            3 => YELLOW.c300,
            4 => GREEN.c600,
            _ => RED.c500,
        };
        let line = Line::styled(self.kanji.clone(), (BLACK, bg_color));

        ListItem::new(line).bg(bg_color)
    }
}
