use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Widget},
};

#[derive(Default)]
pub struct ProgressBar {
    ratio: f32,
    width: u16,
    height: u16,
    bordered: bool,
}

impl ProgressBar {
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn bordered(mut self) -> Self {
        self.bordered = true;
        self
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Round width to fit all bars nicely
        let rounded_width = if self.bordered {
            ((self.width) / 4) * 4 - 1
        } else {
            ((self.width + 2) / 4) * 4
        };

        let bar_layout = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(rounded_width),
            Constraint::Min(0),
        ])
        .spacing(4)
        .split(area);

        let bar_container =
            Layout::vertical([Constraint::Length(self.height + 2), Constraint::Min(0)])
                .split(bar_layout[1]);

        let widget_area = if self.bordered {
            // TODO: Change this to custom border
            let borderd_block = Block::bordered().border_type(BorderType::Thick);
            let inner_area = borderd_block.inner(bar_container[0]);
            borderd_block.render(bar_container[0], buf);

            inner_area
        } else {
            bar_container[0]
        };

        let total_width = widget_area.width;
        let filled_width = (self.ratio * total_width as f32).round() as u16;

        let vertical_range = if self.height == 0 {
            widget_area.top()..widget_area.bottom()
        } else {
            widget_area.top()..(widget_area.top() + self.height)
        };
        let first_row = vertical_range.start;
        let last_row = vertical_range.end - 1;

        let bar_style = Style::default().fg(Color::White);

        for y in vertical_range {
            for x_offset in 0..total_width {
                let current_x = widget_area.x + x_offset;
                // Leave the first cell empty, then fill the other 3
                let pattern_index = x_offset % 4;
                let is_spot = pattern_index != 0;
                let is_first_row = y == first_row;
                let is_last_row = y == last_row;
                let (symbol, style) = if x_offset < filled_width {
                    if self.height < 3 {
                        if pattern_index % 2 == 0 {
                            ("█", bar_style)
                        } else {
                            (" ", bar_style)
                        }
                    } else if is_spot {
                        if is_first_row && (pattern_index == 1) {
                            ("▟", bar_style)
                        } else if is_first_row && (pattern_index == 3) {
                            ("▙", bar_style)
                        } else if is_last_row && (pattern_index == 1) {
                            ("▜", bar_style)
                        } else if is_last_row && (pattern_index == 3) {
                            ("▛", bar_style)
                        } else {
                            ("█", bar_style)
                        }
                    } else {
                        (" ", Style::default())
                    }
                } else {
                    (" ", Style::default())
                };

                if let Some(cell) = buf.cell_mut((current_x, y)) {
                    cell.set_symbol(symbol);
                    cell.set_style(style);
                }
            }
        }
    }
}
