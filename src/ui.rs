use std::{fmt, fs::File, io::BufReader};

use image::{AnimationDecoder, ImageReader, RgbaImage, codecs::gif::GifDecoder};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget},
};
use tui_big_text::{BigText, PixelSize};

use crate::game::{Choice, Game};

#[derive(Clone, Debug, Default, PartialEq)]
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

#[derive(Debug, Default)]
pub enum RenderStats {
    #[default]
    Stats,
    Discipline,
    Hungry,
    Happiness,
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
    let (titles, selected) = if is_top {
        let tab_titles = vec![
            GameView::Feed.to_string(),
            GameView::Lights.to_string(),
            GameView::Play.to_string(),
            GameView::Medicine.to_string(),
        ];
        let selected_tab = if game.highlighted_tab.index() > 0 {
            Some(game.highlighted_tab.index())
        } else {
            None
        };
        (tab_titles, selected_tab)
    } else {
        let tab_titles = vec![
            GameView::Bath.to_string(),
            GameView::Health.to_string(),
            GameView::Scold.to_string(),
            GameView::Attention.to_string(),
        ];
        let selected_tab = if game.highlighted_tab.index() >= 5 {
            Some(game.highlighted_tab.index() - 4)
        } else {
            None
        };
        (tab_titles, selected_tab)
    };

    let tab_constraints = vec![Constraint::Fill(1); 4];
    let tab_chunks = Layout::horizontal(tab_constraints)
        .flex(Flex::SpaceEvenly)
        .split(area);
    for (i, title) in titles.iter().enumerate() {
        let is_selected = i + 1 == selected.unwrap_or_default();
        let block = Block::new()
            .bg(if is_selected {
                Color::White
            } else {
                Color::default()
            })
            .fg(if is_selected {
                Color::Black
            } else {
                Color::default()
            });
        self::render_image_as_paragraph(
            format!("assets/navigation/{}.png", title),
            None,
            tab_chunks[i],
        )
        .style(if is_selected {
            Style::default().fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        })
        .block(block)
        .render(tab_chunks[i], buf);
    }
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: Here we likely want to differentiate between the Catching-Up and the Running states
        render_view(self, area, buf)
    }
}

fn render_view(game: &Game, area: Rect, buf: &mut Buffer) {
    let title = Line::from(" Ratagotchi ".bold());
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(4),
    ])
    .spacing(1);

    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(get_instructions(game))
        .border_set(border::THICK);

    let [top, main, bottom] = block.inner(area).layout(&layout);

    // Render outer border
    block.render(area, buf);

    // navigation(game, (top, bottom), buf);
    navigation_row(game, top, buf, true);
    navigation_row(game, bottom, buf, false);

    match game.active_view {
        GameView::Main => render_main(game, main, buf),
        GameView::Feed => render_feed(game, main, buf),
        GameView::Lights => todo!(),
        GameView::Play => todo!(),
        GameView::Medicine => todo!(),
        GameView::Bath => todo!(),
        GameView::Health => render_health(game, main, buf),
        GameView::Scold => todo!(),
        GameView::Attention => todo!(),
    }
}

fn render_image_as_paragraph(path: String, frame: Option<usize>, area: Rect) -> Paragraph<'static> {
    let rgba_img = match frame {
        Some(f) => {
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            let decoder = GifDecoder::new(reader).unwrap();
            let mut frames = decoder.into_frames();
            let frame = frames.nth(f);

            frame.unwrap().unwrap().into_buffer()
        }
        None => {
            let image_reader = match ImageReader::open(path.to_lowercase()) {
                Ok(r) => r,
                Err(_) => panic!("Can't find asset: {}", path.to_lowercase()),
            };

            image_reader.decode().unwrap().into_rgba8()
        }
    };

    let lines = img_to_lines(&rgba_img);

    // Vertically center the content
    let area_height = if area.height > 2 {
        area.height - 2
    } else {
        area.height
    } as usize;
    let line_count = lines.len();
    let top_padding = if area_height > line_count {
        (area_height - line_count) / 2
    } else {
        0
    };

    let block = Block::new().padding(Padding::top(top_padding as u16));

    Paragraph::new(lines)
        .block(block)
        .style(Style::new().white())
        .centered()
}

// Returns as white monochrome lines
pub fn img_to_lines<'a>(img: &RgbaImage) -> Vec<Line<'a>> {
    let width = img.width();
    let height = img.height();
    let mut lines = Vec::new();

    let transparent_fallback = image::Rgba([0u8, 0u8, 0u8, 0u8]);

    for y in (0..height).step_by(2) {
        let mut row_spans = Vec::new();

        for x in 0..width {
            let top_pixel = img.get_pixel(x, y);
            let bottom_pixel = if y + 1 < height {
                img.get_pixel(x, y + 1)
            } else {
                &transparent_fallback
            };

            // Check alpha channel: true if the pixel is "mostly" transparent
            let top_is_transparent = top_pixel[3] < 128;
            let bottom_is_transparent = bottom_pixel[3] < 128;

            let span = match (top_is_transparent, bottom_is_transparent) {
                (true, true) => Span::raw(" "),
                (true, false) => Span::styled("▄", Style::default()),
                (false, true) => Span::styled("▀", Style::default()),
                (false, false) => Span::styled("█", Style::default()),
            };
            row_spans.push(span);
        }

        lines.push(Line::from(row_spans));
    }

    lines
}

fn get_instructions(game: &Game) -> Line<'static> {
    Line::from(format!(
        " <Tab> for menu navigation, <Enter> to trigger selected item - tick: {}, activity_ticks: {}, hunger: {:.3?} ",
        game.tick_progress, game.activity_ticks, game.ratagotchi.hunger
    ))
    .alignment(Alignment::Center)
}

fn two_by_two_layout(area: Rect) -> [Rect; 4] {
    let rows =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).spacing(1);
    let block = Block::new();
    let [top_row, bottom_row] = block.inner(area).layout(&rows);
    let choice_layout =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let choice_row_top = Block::new();
    let [top_left, top_right] = choice_row_top.inner(top_row).layout(&choice_layout);
    let choice_row_bottom = Block::new();
    let [bottom_left, bottom_right] = choice_row_bottom.inner(bottom_row).layout(&choice_layout);
    [top_left, top_right, bottom_left, bottom_right]
}

fn render_main(game: &Game, area: Rect, buf: &mut Buffer) {
    self::render_image_as_paragraph(
        "assets/ratagotchi-idle.gif".to_string(),
        Some(game.tick_progress % 2),
        area,
    )
    .centered()
    .render(area, buf);
}

fn render_feed(game: &Game, area: Rect, buf: &mut Buffer) {
    if game.activity_ticks > 0 {
        // Use the feeding item to run a short animation
        self::render_image_as_paragraph(
            "assets/ratagotchi-eat.gif".to_string(),
            Some(game.activity_ticks % 2),
            area,
        )
        .centered()
        .render(area, buf);
    } else {
        let [top_left, top_right, bottom_left, bottom_right] = two_by_two_layout(area);
        // Rendering top and bottom arrows
        let active_area = if game.choice == Choice::Top {
            top_left
        } else {
            bottom_left
        };
        self::render_image_as_paragraph("assets/arrow.png".to_string(), None, active_area)
            .centered()
            .render(active_area, buf);

        // Rendering food options
        self::render_image_as_paragraph("assets/cheese.png".to_string(), None, top_right)
            .centered()
            .render(top_right, buf);
        self::render_image_as_paragraph("assets/ratfood.png".to_string(), None, top_right)
            .centered()
            .render(bottom_right, buf);
    }
}

fn render_health(game: &Game, area: Rect, buf: &mut Buffer) {
    let [top_left, top_right, bottom_left, bottom_right] = two_by_two_layout(area);
    match game.stats_screen {
        RenderStats::Stats => {
            self::render_image_as_paragraph(
                "assets/ratagotchi-head.png".to_string(),
                None,
                top_left,
            )
            .centered()
            .render(top_left, buf);
            BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::new().white())
                .centered()
                .lines([format!("{:.0} yr", game.ratagotchi.age).into()])
                .build()
                .render(top_right, buf);

            self::render_image_as_paragraph("assets/scale.png".to_string(), None, bottom_left)
                .centered()
                .render(bottom_left, buf);
            BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::new().white())
                .centered()
                .lines([format!("{:.} kg", game.ratagotchi.weight).into()])
                .build()
                .render(bottom_right, buf);
        }
        RenderStats::Discipline => todo!(),
        RenderStats::Hungry => todo!(),
        RenderStats::Happiness => todo!(),
    }
}
