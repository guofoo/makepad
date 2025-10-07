use {
    crate::widget::*,
    makepad_draw::*,
    makepad_derive_widget::*,
    makepad_draw::shader::draw_text::TextStyle
};

live_design! {
    link widgets;
    
    use link::theme::*;

    pub TextFlow2 = {{TextFlow2}} {
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
    #[redraw]
    area: Area,

    #[layout]
    layout: Layout,

    #[live]
    default_color: Vec4,
    #[live]
    text_styles: TextStyles,
    #[live]
    draw_text: DrawText,

    #[rust]
    styles: StyleStack,
}

impl TextFlow2 {
    pub fn begin(&mut self, cx: &mut Cx2d, walk: Walk) {
        cx.begin_turtle(walk, self.layout);
    }

    pub fn end(&mut self, cx: &mut Cx2d) {
        cx.end_turtle();
    }

    pub fn push_style(&mut self, style: Style) {
        self.styles.push(style);
    }

    pub fn pop_style(&mut self) {
        self.styles.pop();
    }

    pub fn draw_text(&mut self, cx: &mut Cx2d, text: &str) {
        let style = self.styles.flatten();
        self.draw_text.color = style.color.unwrap_or(self.default_color);
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
        self.draw_text.draw_walk_resumable(cx, text);
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
    fn flatten(&self) -> FlattenedStyle {
        FlattenedStyle {
            color: self.colors.last().copied(),
            bold: self.counts.bold != 0,
            italic: self.counts.italic != 0,
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
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Style {
    Color(Vec4),
    Bold,
    Italic,
}

#[derive(Clone, Copy, Debug, Default)]
struct StyleCounts {
    bold: usize,
    italic: usize,
}

#[derive(Clone, Copy, Debug)]
struct FlattenedStyle {
    color: Option<Vec4>,
    bold: bool,
    italic: bool,
}