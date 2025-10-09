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

    pub TextFlow2 = {{TextFlow2}} {
        draw_underline: {
            draw_depth: 0.0,

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.hline(
                    self.rect_size.y - 1.0,
                    1.0
                );
                sdf.fill(self.color);
                return sdf.result;
            }
        }

        default_color: (THEME_COLOR_TEXT),
        text_styles: {
            normal: <THEME_FONT_REGULAR> {},
            bold: <THEME_FONT_BOLD> {},
            italic: <THEME_FONT_ITALIC> {},
            bold_italic: <THEME_FONT_BOLD_ITALIC> {},
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TextFlow2 {
    #[live]
    draw_text: DrawText,
    #[live]
    draw_underline: DrawUnderline,

    #[layout]
    layout: Layout,

    #[live]
    default_color: Vec4,
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
        let style = self.styles.flatten(self.default_color);
        self.draw_text.color = style.color;
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
        self.draw_text.draw_walk_resumable_with(cx, text, |cx, rect, _| {
            if style.underline {
                self.draw_underline.color = style.color;
                self.draw_underline.draw_abs(cx, rect);
            }
        });
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
    styles: Vec<Style>,
    colors: Vec<Vec4>,
    counts: StyleCounts,
}

impl StyleStack {
    fn flatten(&self, default_color: Vec4) -> FlattenedStyle {
        FlattenedStyle {
            color: self.colors.last().copied().unwrap_or(default_color),
            bold: self.counts.bold != 0,
            italic: self.counts.italic != 0,
            underline: self.counts.underline != 0,
        }
    }

    fn push(&mut self, style: Style) {
        match style {
            Style::Color(color) => {
                self.colors.push(color);
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
        }
        self.styles.push(style);
    }

    fn pop(&mut self) {
        if let Some(style) = self.styles.pop() {
            match style {
                Style::Color(_) => {
                    self.colors.pop();
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
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Style {
    Color(Vec4),
    Bold,
    Italic,
    Underline,
}

#[derive(Clone, Copy, Debug, Default)]
struct StyleCounts {
    bold: usize,
    italic: usize,
    underline: usize,
}

#[derive(Clone, Copy, Debug)]
struct FlattenedStyle {
    color: Vec4,
    bold: bool,
    italic: bool,
    underline: bool,
}