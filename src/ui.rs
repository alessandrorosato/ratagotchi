use std::{fmt, fs::File, io::BufReader, path::Path};

use chrono::Utc;
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

use crate::{
    config::RATAGOTCHI_ACTIVITY_DURATION,
    game::{Choice, Game},
    widgets::ProgressBar,
};

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

#[derive(Debug)]
enum VerticalPosition {
    Center,
    Bottom,
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
            tab_chunks[i],
            VerticalPosition::Center,
            None,
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
        match self.state {
            crate::game::GameState::CatchingUp => render_catching_up(self, area, buf),
            crate::game::GameState::Running => render_view(self, area, buf),
            crate::game::GameState::Exiting => render_exiting(area, buf),
        }
    }
}

fn render_catching_up(game: &Game, area: Rect, buf: &mut Buffer) {
    let title = Line::from(" Ratagotchi ".bold());
    let vertical_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .spacing(1)
    .split(area);
    let horizontal_layout =
        Layout::horizontal([Constraint::Fill(1), Constraint::Min(5), Constraint::Fill(1)])
            .spacing(1)
            .split(vertical_layout[1]);

    let block = Block::bordered()
        .title(title.centered())
        .border_set(border::THICK);

    let time_delta = (Utc::now() - game.ratagotchi.last_update).num_seconds();
    let human_speech = human_friendly_time_delta(time_delta);
    let line = Line::from(format!("Catching Up: {}", human_speech));

    Paragraph::new(line)
        .block(block)
        .style(Style::new().white())
        .centered()
        .render(horizontal_layout[1], buf);
}

fn human_friendly_time_delta(time_delta: i64) -> String {
    let days = time_delta / 86400;
    let remaining_seconds = time_delta % 86400;
    let hours = remaining_seconds / 3600;
    let remaining_seconds = remaining_seconds % 3600;
    let minutes = remaining_seconds / 60;
    let seconds = remaining_seconds % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days > 1 { "s" } else { "" }));
    }
    if hours > 0 {
        parts.push(format!(
            "{} hour{}",
            hours,
            if hours > 1 { "s" } else { "" }
        ));
    }
    if minutes > 0 {
        parts.push(format!(
            "{} minute{}",
            minutes,
            if minutes > 1 { "s" } else { "" }
        ));
    }
    if seconds > 0 && parts.is_empty() {
        parts.push(format!(
            "{} second{}",
            seconds,
            if seconds > 1 { "s" } else { "" }
        ));
    }

    match parts.len() {
        0 => "0 seconds".to_string(),
        1 => parts[0].clone(),
        _ => {
            let last = parts.pop().unwrap();
            let rest = parts.join(", ");
            format!("{} and {}", rest, last)
        }
    }
}

fn render_exiting(area: Rect, buf: &mut Buffer) {
    let title = Line::from(" Ratagotchi ".bold());
    let vertical_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .spacing(1)
    .split(area);
    let horizontal_layout =
        Layout::horizontal([Constraint::Fill(1), Constraint::Min(5), Constraint::Fill(1)])
            .spacing(1)
            .split(vertical_layout[1]);

    let block = Block::bordered()
        .title(title.centered())
        .border_set(border::THICK);

    let line = Line::from("Saving progress...");

    Paragraph::new(line)
        .block(block)
        .style(Style::new().white())
        .centered()
        .render(horizontal_layout[1], buf);
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
        GameView::Lights => render_main(game, main, buf),
        GameView::Play => todo!(),
        GameView::Medicine => render_medicine(game, main, buf),
        GameView::Bath => render_bath(game, main, buf),
        GameView::Health => render_health(game, main, buf),
        GameView::Scold => render_scold(game, main, buf),
        GameView::Attention => render_attention(game, main, buf),
    }
}

fn render_image_as_paragraph(
    path: String,
    area: Rect,
    vertical_position: VerticalPosition,
    frame: Option<usize>,
) -> Paragraph<'static> {
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

    let padding = match vertical_position {
        VerticalPosition::Center => {
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
            Padding::top(top_padding as u16)
        }
        VerticalPosition::Bottom => {
            let area_height = if area.height > 2 {
                area.height - 2
            } else {
                area.height
            } as usize;
            let line_count = lines.len();
            let top_padding = area_height.saturating_sub(line_count);
            Padding::top(top_padding as u16)
        }
    };

    let block = Block::new().padding(padding);

    Paragraph::new(lines)
        .block(block)
        .style(Style::new().white())
        .centered()
}

fn render_wave_over_image(
    image_path: String,
    wave_path: String,
    area: Rect,
    vertical_position: VerticalPosition,
    frame: Option<usize>,
    activity_tick: usize,
    buf: &mut Buffer,
) {
    let mut line_imgs = Vec::new();
    for path in [image_path, wave_path].iter() {
        let rgba_img = match frame {
            Some(f) => {
                if Path::new(&path).extension().and_then(|ext| ext.to_str()) == Some("gif") {
                    let file = File::open(path).unwrap();
                    let reader = BufReader::new(file);
                    let decoder = GifDecoder::new(reader).unwrap();
                    let mut frames = decoder.into_frames();
                    let frame = frames.nth(f);

                    frame.unwrap().unwrap().into_buffer()
                } else {
                    let image_reader = match ImageReader::open(path.to_lowercase()) {
                        Ok(r) => r,
                        Err(_) => panic!("Can't find asset: {}", path.to_lowercase()),
                    };

                    image_reader.decode().unwrap().into_rgba8()
                }
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

        line_imgs.push(lines);
    }

    let max_line_len = line_imgs.iter().map(|l| l.len()).max().unwrap_or(0);

    let image_padding = match vertical_position {
        VerticalPosition::Center => {
            let area_height = if area.height > 2 {
                area.height - 2
            } else {
                area.height
            } as usize;
            let line_count = max_line_len;
            let top_padding = if area_height > line_count {
                (area_height - line_count) / 2
            } else {
                0
            };
            Padding::top(top_padding as u16)
        }
        VerticalPosition::Bottom => {
            let area_height = if area.height > 2 {
                area.height - 2
            } else {
                area.height
            } as usize;
            let line_count = max_line_len;
            let top_padding = area_height.saturating_sub(line_count);
            Padding::top(top_padding as u16)
        }
    };

    let wave_position_index = activity_tick.abs_diff(4);
    let wave_position_h =
        area.width / (RATAGOTCHI_ACTIVITY_DURATION - 4) as u16 * wave_position_index as u16;
    let wave_padding = if activity_tick > 4 {
        Padding {
            left: wave_position_h,
            top: image_padding.top,
            ..Default::default()
        }
    } else {
        Padding {
            right: wave_position_h,
            top: image_padding.top,
            ..Default::default()
        }
    };

    Paragraph::new(line_imgs[0].clone())
        .block(Block::new().padding(image_padding))
        .style(Style::new().white())
        .centered()
        .render(area, buf);
    Paragraph::new(line_imgs[1].clone())
        .block(Block::new().padding(wave_padding))
        .style(Style::new().white())
        .centered()
        .render(area, buf);
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
        " <Tab> for menu navigation, <Enter> to select, <Esc> to return to main view - tick: {}, activity_ticks: {} ",
        game.tick_progress, game.activity_ticks
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

fn center_and_columns_layout(area: Rect) -> [Rect; 3] {
    let rows = Layout::horizontal([
        Constraint::Min(30),
        Constraint::Fill(1),
        Constraint::Min(30),
    ])
    .spacing(1);
    let block = Block::new();
    let [left, main_area, right] = block.inner(area).layout(&rows);

    [left, main_area, right]
}

// Render the top half with the title and return the bottom one
fn render_stats_title(title: Line<'_>, area: Rect, buf: &mut Buffer) -> Rect {
    let rows =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).spacing(1);
    let block = Block::new();
    let [top_row, bottom_row] = block.inner(area).layout(&rows);

    let num_lines = 1;
    let text_height = num_lines * 8;
    let sub_rows = Layout::vertical([Constraint::Min(0), Constraint::Length(text_height as u16)])
        .split(top_row);

    BigText::builder()
        .pixel_size(PixelSize::Full)
        .style(Style::new().white())
        .centered()
        .lines([title])
        .build()
        .render(sub_rows[1], buf);
    bottom_row
}

fn render_stats_hearts(value: f32, area: Rect, buf: &mut Buffer) {
    let hearts_layout = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Min(0),
    ])
    .spacing(4)
    .split(area);

    let full_hearts = (value / 25.0).ceil() as usize;

    for i in 1..hearts_layout.len() - 1 {
        let which_image = if full_hearts >= i {
            "assets/heart.png"
        } else {
            "assets/heart-empty.png"
        };
        self::render_image_as_paragraph(
            which_image.to_string(),
            area,
            VerticalPosition::Center,
            None,
        )
        .centered()
        .render(hearts_layout[i], buf);
    }
}

fn render_main(game: &Game, area: Rect, buf: &mut Buffer) {
    if game.is_dark {
        // TODO: Add something animated "in the dark"
    } else {
        let [left, center, right] = center_and_columns_layout(area);
        self::render_image_as_paragraph(
            "assets/ratagotchi-idle.gif".to_string(),
            center,
            VerticalPosition::Center,
            Some(game.tick_progress % 2),
        )
        .centered()
        .render(center, buf);

        // Define the poop situation
        if game.ratagotchi.poop >= 50. {
            self::render_image_as_paragraph(
                "assets/poop.gif".to_string(),
                left,
                VerticalPosition::Bottom,
                Some(game.tick_progress % 2),
            )
            .centered()
            .render(left, buf);
        }

        if game.ratagotchi.poop >= 95. {
            self::render_image_as_paragraph(
                "assets/poop.gif".to_string(),
                right,
                VerticalPosition::Center,
                Some(game.tick_progress % 2),
            )
            .centered()
            .render(right, buf);
        }
    }
}

fn render_feed(game: &Game, area: Rect, buf: &mut Buffer) {
    if game.activity_ticks > 0 {
        let view_layout =
            Layout::horizontal([Constraint::Min(16), Constraint::Fill(1)]).split(area);
        let left_column =
            Layout::vertical([Constraint::Min(16), Constraint::Fill(1)]).split(view_layout[0]);

        // Use the feeding item to run a short animation
        self::render_image_as_paragraph(
            "assets/ratagotchi-eat.gif".to_string(),
            view_layout[1],
            VerticalPosition::Center,
            Some(game.activity_ticks % 2),
        )
        .centered()
        .render(view_layout[1], buf);

        // Food being chopped away
        let food_asset = if game.choice == Choice::Top {
            "assets/cheese-eat.gif"
        } else {
            "assets/ratfood-eat.gif"
        };
        self::render_image_as_paragraph(
            food_asset.to_string(),
            left_column[0],
            VerticalPosition::Center,
            Some(4 - (game.activity_ticks / 2)),
        )
        .centered()
        .render(left_column[0], buf);
    } else {
        let [top_left, top_right, bottom_left, bottom_right] = two_by_two_layout(area);
        // Rendering top and bottom arrows
        let active_area = if game.choice == Choice::Top {
            top_left
        } else {
            bottom_left
        };
        self::render_image_as_paragraph(
            "assets/arrow.png".to_string(),
            active_area,
            VerticalPosition::Center,
            None,
        )
        .centered()
        .render(active_area, buf);

        // Rendering food options
        self::render_image_as_paragraph(
            "assets/cheese.png".to_string(),
            top_right,
            VerticalPosition::Center,
            None,
        )
        .centered()
        .render(top_right, buf);
        self::render_image_as_paragraph(
            "assets/ratfood.png".to_string(),
            top_right,
            VerticalPosition::Center,
            None,
        )
        .centered()
        .render(bottom_right, buf);
    }
}

fn render_medicine(game: &Game, area: Rect, buf: &mut Buffer) {
    let frame = if game.activity_ticks > 5 { 0 } else { 1 };
    self::render_image_as_paragraph(
        "assets/ratagotchi-medicine.gif".to_string(),
        area,
        VerticalPosition::Center,
        Some(frame),
    )
    .centered()
    .render(area, buf);
}

fn render_bath(game: &Game, area: Rect, buf: &mut Buffer) {
    let frame = if game.activity_ticks.is_multiple_of(2) {
        0
    } else {
        1
    };
    self::render_wave_over_image(
        "assets/ratagotchi-idle.gif".to_string(),
        "assets/screen-wave.png".to_string(),
        area,
        VerticalPosition::Center,
        Some(frame),
        game.activity_ticks,
        buf,
    );
}

fn render_health(game: &Game, area: Rect, buf: &mut Buffer) {
    let [top_left, top_right, bottom_left, bottom_right] = two_by_two_layout(area);
    match game.stats_screen {
        RenderStats::Stats => {
            self::render_image_as_paragraph(
                "assets/ratagotchi-head.png".to_string(),
                top_left,
                VerticalPosition::Center,
                None,
            )
            .centered()
            .render(top_left, buf);
            BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::new().white())
                .centered()
                .lines([game.ratagotchi.print_age().into()])
                .build()
                .render(top_right, buf);

            self::render_image_as_paragraph(
                "assets/scale.png".to_string(),
                bottom_left,
                VerticalPosition::Center,
                None,
            )
            .centered()
            .render(bottom_left, buf);
            let two_rows = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]);
            let weight_block = Block::new();
            let [upper, lower] = weight_block.inner(bottom_right).layout(&two_rows);
            BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::new().white())
                .centered()
                .lines([game.ratagotchi.print_weight().into()])
                .build()
                .render(upper, buf);
            BigText::builder()
                .pixel_size(PixelSize::Quadrant)
                .style(Style::new().white())
                .centered()
                .lines([game.ratagotchi.print_fitness().into()])
                .build()
                .render(lower, buf);
        }
        RenderStats::Discipline => {
            let bottom_row = render_stats_title(Line::from("Discipline"), area, buf);
            ProgressBar::default()
                .ratio(game.ratagotchi.discipline / 100.)
                .width(bottom_row.width / 2)
                .height(2)
                .bordered()
                .render(bottom_row, buf);
        }
        RenderStats::Hungry => {
            let bottom_row = render_stats_title(Line::from("Hungry"), area, buf);
            render_stats_hearts(game.ratagotchi.hunger, bottom_row, buf);
        }
        RenderStats::Happiness => {
            let bottom_row = render_stats_title(Line::from("Happiness"), area, buf);
            render_stats_hearts(game.ratagotchi.happiness, bottom_row, buf);
        }
    }
}

fn render_scold(game: &Game, area: Rect, buf: &mut Buffer) {
    let frame = if game.activity_ticks.is_multiple_of(2) {
        0
    } else {
        1
    };
    self::render_image_as_paragraph(
        "assets/ratagotchi-scold.gif".to_string(),
        area,
        VerticalPosition::Center,
        Some(frame),
    )
    .centered()
    .render(area, buf);
}

fn render_attention(game: &Game, area: Rect, buf: &mut Buffer) {
    // Flash content
    let should_render = game.tick_progress.is_multiple_of(2);

    if should_render {
        let mut low_values = vec![];

        // Get low values
        if game.ratagotchi.hunger < 20.0 {
            low_values.push(Span::from("Low hunger"));
        }
        if game.ratagotchi.health < 20.0 {
            low_values.push(Span::from("Low health"));
        }
        if game.ratagotchi.discipline < 20.0 {
            low_values.push(Span::from("Low discipline"));
        }
        if game.ratagotchi.happiness < 20.0 {
            low_values.push(Span::from("Low happiness"));
        }

        let (layout, text_size) = if low_values.len() < 3 {
            // We need to stick to [Rect; 4], so we add two small rows at the bottom
            let two_rows = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Percentage(0),
                Constraint::Percentage(0),
            ]);
            let block = Block::new();
            (block.inner(area).layout(&two_rows), PixelSize::Full)
        } else {
            (two_by_two_layout(area), PixelSize::Quadrant)
        };

        for (i, value) in low_values.into_iter().enumerate() {
            let inner_layout =
                Layout::vertical([Constraint::Min(1), Constraint::Fill(1)]).split(layout[i]);

            BigText::builder()
                .pixel_size(text_size)
                .style(Style::new().white())
                .centered()
                .lines([value.into()])
                .build()
                .render(inner_layout[1], buf);
        }
    }
}
