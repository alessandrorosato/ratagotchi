use std::fmt;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Tabs, Widget},
};

use crate::game::Game;

pub struct MainWidget;
pub struct FeedWidget;
pub struct LightsWidget;
pub struct PlayWidget;
pub struct MedicineWidget;
pub struct BathWidget;
pub struct HealthWidget;
pub struct ScoldWidget;
pub struct AttentionWidget;

#[derive(Clone, Debug, Default)]
pub enum GameView {
    #[default]
    Main,
    Feed,
    Lights,
    Play,
    Medicine,
    Bath,
    Health,
    Scold,
    Attention,
}

impl fmt::Display for GameView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl GameView {
    fn index(&self) -> usize {
        match self {
            GameView::Main => 0,
            GameView::Feed => 1,
            GameView::Lights => 2,
            GameView::Play => 3,
            GameView::Medicine => 4,
            GameView::Bath => 5,
            GameView::Health => 6,
            GameView::Scold => 7,
            GameView::Attention => 8,
        }
    }
}

fn navigation_row(game: &Game, area: Rect, buf: &mut Buffer, is_top: bool) {
    let (titles, selected, padding) = if is_top {
        let tab_titles = vec![
            GameView::Feed.to_string(),
            GameView::Lights.to_string(),
            GameView::Play.to_string(),
            GameView::Medicine.to_string(),
        ];
        let selected_tab = if game.highlighted_tab.index() > 0 {
            Some(game.highlighted_tab.index() - 1)
        } else {
            None
        };
        let total_text_len: usize = tab_titles.iter().map(|s| s.len()).sum();
        let total_dividers_len: usize = tab_titles.len() - 1; // Assuming default "|" divider
        let total_content_width = total_text_len + total_dividers_len;
        let screen_width = area.width as usize;
        let dynamic_padding = if screen_width > total_content_width {
            let remaining_space = screen_width - total_content_width;
            let padding_per_side = remaining_space / (tab_titles.len() * 2);
            " ".repeat(padding_per_side.max(1)) // At least 1 space
        } else {
            " ".to_string()
        };
        (tab_titles, selected_tab, dynamic_padding)
    } else {
        let tab_titles = vec![
            GameView::Bath.to_string(),
            GameView::Health.to_string(),
            GameView::Scold.to_string(),
            GameView::Attention.to_string(),
        ];
        let selected_tab = if game.highlighted_tab.index() >= 5 {
            Some(game.highlighted_tab.index() - 5)
        } else {
            None
        };
        let total_text_len: usize = tab_titles.iter().map(|s| s.len()).sum();
        let total_dividers_len: usize = tab_titles.len() - 1; // Assuming default "|" divider
        let total_content_width = total_text_len + total_dividers_len;
        let screen_width = area.width as usize;
        let dynamic_padding = if screen_width > total_content_width {
            let remaining_space = screen_width - total_content_width;
            let padding_per_side = remaining_space / (tab_titles.len() * 2);
            " ".repeat(padding_per_side.max(1)) // At least 1 space
        } else {
            " ".to_string()
        };
        (tab_titles, selected_tab, dynamic_padding)
    };

    Tabs::new(titles)
        .select(selected)
        .padding(padding.as_str(), padding.as_str())
        .render(area, buf);
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());
        match self.active_view {
            GameView::Main => render_main(self, area, buf, title),
            GameView::Feed => todo!(),
            GameView::Lights => todo!(),
            GameView::Play => todo!(),
            GameView::Medicine => todo!(),
            GameView::Bath => todo!(),
            GameView::Health => todo!(),
            GameView::Scold => todo!(),
            GameView::Attention => todo!(),
        }
    }
}

fn render_main(game: &Game, area: Rect, buf: &mut Buffer, title: Line) {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .spacing(1);

    let [top, main, bottom] = area.layout(&layout);

    let block = Block::bordered()
        .title(title.centered())
        .title_bottom("Stuff at the bottom")
        .border_set(border::THICK);

    // navigation(game, (top, bottom), buf);
    navigation_row(game, top, buf, true);
    navigation_row(game, bottom, buf, false);

    Paragraph::new(format!("main {:#?} {}", game, game.highlighted_tab.index()))
        .centered()
        .block(block)
        .render(main, buf);
}

impl Widget for FeedWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("feed")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for LightsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("lights")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for PlayWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("play")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for MedicineWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("medicine")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for BathWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("bath")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for HealthWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("health")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for ScoldWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("scold")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for AttentionWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Ratagotchi ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom("Stuff at the bottom")
            .border_set(border::THICK);

        Paragraph::new("attention")
            .centered()
            .block(block)
            .render(area, buf);
    }
}
