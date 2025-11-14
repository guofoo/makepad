use {
    crate::widget::*,
    makepad_draw::*,
    makepad_derive_widget::*,
    makepad_draw::shader::draw_text::TextStyle
};

live_design! {
    link widgets;
    
    use link::theme::*;
    use makepad_draw::shader::std::*;

    pub DrawUnderline = {{DrawUnderline}} {}
    pub DrawStrikethrough = {{DrawStrikethrough}} {}

    pub TextFlow2 = {{TextFlow2}} {
        width: Fill,
        height: Fit,
        flow: Right {
            row_align: Bottom,
            wrap: false,
        },

        font_size: (THEME_FONT_SIZE_P),
        font_color: (THEME_COLOR_TEXT),
        text_styles: {
            normal: <THEME_FONT_REGULAR> {},
            bold: <THEME_FONT_BOLD> {},
            italic: <THEME_FONT_ITALIC> {},
            bold_italic: <THEME_FONT_BOLD_ITALIC> {},
        }

        draw_underline: {
            draw_depth: 0.0,

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.hline(
                    self.rect_size.y - 0.5,
                    1.0
                );
                sdf.fill(self.color);
                return sdf.result;
            }
        }

        draw_strikethrough: {
            draw_depth: 2.0,

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.hline(
                    0.65 * self.ascender,
                    1.0
                );
                sdf.fill(self.color);
                return sdf.result;
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TextFlow2 {
    #[live]
    draw_underline: DrawUnderline,
    #[live]
    draw_text: DrawText,
    #[live]
    draw_strikethrough: DrawStrikethrough,

    #[layout]
    layout: Layout,
    #[walk]
    walk: Walk,

    #[live]
    font_size: f32,
    #[live]
    font_color: Vec4,
    #[live]
    text_styles: TextStyles,

    #[redraw]
    #[rust]
    area: Area,
    #[rust]
    styles: StyleStack,
}

impl TextFlow2 {
    pub fn begin(&mut self, cx: &mut Cx2d, walk: Walk) {
        cx.begin_turtle(walk, self.layout);
    }

    pub fn end(&mut self, cx: &mut Cx2d) {
        cx.end_turtle_with_area(&mut self.area);
    }

    pub fn push_style(&mut self, style: Style) {
        self.styles.push(style);
    }

    pub fn pop_style(&mut self) {
        self.styles.pop();
    }

    pub fn draw_text(&mut self, cx: &mut Cx2d, text: &str) {
        let style = self.styles.flatten(
            self.font_size,
            self.font_color,
        );
        self.draw_text.color = style.font_color;
        self.draw_text.text_style = match style {
            FlattenedStyle {
                bold: false,
                italic: false,
                ..
            } => self.text_styles.normal,
            FlattenedStyle {
                bold: true,
                italic: false,
                ..
            } => self.text_styles.bold,
            FlattenedStyle {
                bold: false,
                italic: true,
                ..
            } => self.text_styles.italic,
            FlattenedStyle {
                bold: true,
                italic: true,
                ..
            } => self.text_styles.bold_italic,
        };
        self.draw_text.text_style.font_size = style.font_size;
        self.draw_text.debug = true;
        let laidout_text = self.draw_text.layout(
            cx,
            0.0,
            0.0,
            None,
            false,
            Align::default(),
            text
        );
        self.draw_text.draw_walk_laidout(cx, Walk::fit(), &laidout_text);
        
        /*
            if style.underline {
                self.draw_underline.color = style.font_color;
                self.draw_underline.draw_abs(cx, rect);
            }
            if style.strikethrough {
                self.draw_strikethrough.color = style.font_color;
                self.draw_strikethrough.ascender = ascender;
                self.draw_strikethrough.draw_abs(cx, rect);
            }
        */
    }
}

impl Widget for TextFlow2 {
    fn draw_walk(
        &mut self,
        _cx: &mut Cx2d,
        _scope: &mut Scope,
        _walk: Walk
    ) -> DrawStep {
        unimplemented!()
    }

    fn handle_event(
        &mut self,
        _cx: &mut Cx,
        _event: &Event,
        _scope: &mut Scope)
    {
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
struct DrawUnderline {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    color: Vec4,
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
struct DrawStrikethrough {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    color: Vec4,
    #[live]
    ascender: f32,
}

#[derive(Live, LiveHook, LiveRegister)]
#[live_ignore]
pub struct TextStyles {
    #[live]
    normal: TextStyle,
    #[live]
    bold: TextStyle,
    #[live]
    italic: TextStyle,
    #[live]
    bold_italic: TextStyle,
}

#[derive(Clone, Debug, Default)]
struct StyleStack {
    font_sizes: Vec<f32>,
    font_colors: Vec<Vec4>,
    counts: StyleCounts,
    styles: Vec<Style>,
}

impl StyleStack {
    fn flatten(&self, default_font_size: f32, default_font_color: Vec4) -> FlattenedStyle {
        FlattenedStyle {
            font_size: self.font_sizes.last().copied().unwrap_or(default_font_size),
            font_color: self.font_colors.last().copied().unwrap_or(default_font_color),
            bold: self.counts.bold != 0,
            italic: self.counts.italic != 0,
            underline: self.counts.underline != 0,
            strikethrough: self.counts.strikethrough != 0,
        }
    }

    fn push(&mut self, style: Style) {
        match style {
            Style::FontSize(font_size) => {
                self.font_sizes.push(font_size);
            }
            Style::FontColor(font_color) => {
                self.font_colors.push(font_color);
            }
            Style::Bold => {
                self.counts.bold += 1;
            }
            Style::Italic => {
                self.counts.italic += 1;
            }
            Style::Underline => {
                self.counts.underline += 1;
            }
            Style::Strikethrough => {
                self.counts.strikethrough += 1;
            }
        }
        self.styles.push(style);
    }

    fn pop(&mut self) {
        if let Some(style) = self.styles.pop() {
            match style {
                Style::FontSize(_) => {
                    self.font_sizes.pop();
                }
                Style::FontColor(_) => {
                    self.font_colors.pop();
                },
                Style::Bold => {
                    self.counts.bold -= 1;
                }
                Style::Italic => {
                    self.counts.italic -= 1;
                }
                Style::Underline => {
                    self.counts.underline -= 1;
                }
                Style::Strikethrough => {
                    self.counts.strikethrough -= 1;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Style {
    FontSize(f32),
    FontColor(Vec4),
    Bold,
    Italic,
    Underline,
    Strikethrough,
}

#[derive(Clone, Copy, Debug, Default)]
struct StyleCounts {
    bold: usize,
    italic: usize,
    underline: usize,
    strikethrough: usize,
}

#[derive(Clone, Copy, Debug)]
struct FlattenedStyle {
    font_size: f32,
    font_color: Vec4,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}