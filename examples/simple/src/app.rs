
use makepad_widgets::*;

live_design!{
    link widgets;
    
    use link::widgets::*;
    
    MyTextFlow = {{MyTextFlow}} <TextFlow2> {}

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                body = <View> {
                    padding: {
                        top: 32,
                    }
                    <View> {
                        width: Fit,
                        height: Fit,
                        show_bg: true,
                        draw_bg: {
                            fn pixel(self) -> vec4 {
                                return #444;
                            }
                        }
                        <MyTextFlow> {
                            width: 100,
                            height: 700,
                        }
                    }
                }
            }
        }
    }
}  

app_main!(App); 
 
#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
}
 
impl LiveRegister for App {
    fn live_register(cx: &mut Cx) { 
        makepad_widgets::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

#[derive(Live, LiveHook, Widget)]
struct MyTextFlow {
    #[deref]
    text_flow: TextFlow2
}

impl Widget for MyTextFlow {
    fn draw_walk(
        &mut self,
        cx: &mut Cx2d,
        _scope: &mut Scope,
        walk: Walk,
    ) -> DrawStep {
        self.text_flow.begin(cx, walk);
        self.text_flow.push_style(Style::Underline);
        self.text_flow.draw_text(cx, "I don't like 'em putting chemicals ");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::Strikethrough);
        self.text_flow.draw_text(cx, "in the water that turn the freaking ");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::Bold);
        self.text_flow.push_style(Style::FontSize(32.0));
        self.text_flow.draw_text(cx, "frogs");
        self.text_flow.pop_style();
        self.text_flow.pop_style();
        self.text_flow.draw_text(cx, " gay. I'm gonna say it ");
        self.text_flow.push_style(Style::Italic);
        self.text_flow.draw_text(cx, "real");
        self.text_flow.pop_style();
        self.text_flow.draw_text(cx, " slow for you: ");
        self.text_flow.push_style(Style::Bold);
        self.text_flow.push_style(Style::Italic);
        self.text_flow.push_style(Style::FontColor(vec4(1.0, 0.0, 0.0, 1.0)));
        self.text_flow.draw_text(cx, "G");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::FontColor(vec4(1.0, 1.0, 0.0, 1.0)));
        self.text_flow.draw_text(cx, "A");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::FontColor(vec4(0.0, 1.0, 0.0, 1.0)));
        self.text_flow.draw_text(cx, "Y ");
        self.text_flow.pop_style();
        self.text_flow.draw_text(cx,  " ");
        self.text_flow.push_style(Style::FontColor(vec4(0.0, 1.0, 1.0, 1.0)));
        self.text_flow.draw_text(cx, "F");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::FontColor(vec4(0.0, 0.0, 1.0, 1.0)));
        self.text_flow.draw_text(cx, "R");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::FontColor(vec4(1.0, 0.0, 1.0, 1.0)));
        self.text_flow.draw_text(cx, "O");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::FontColor(vec4(1.0, 0.0, 0.0, 1.0)));
        self.text_flow.draw_text(cx, "G");
        self.text_flow.pop_style();
        self.text_flow.push_style(Style::FontColor(vec4(1.0, 1.0, 0.0, 1.0)));
        self.text_flow.draw_text(cx, "S");
        self.text_flow.pop_style();
        self.text_flow.pop_style();
        self.text_flow.pop_style();
        self.text_flow.end(cx);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.text_flow.handle_event(cx, event, scope)
    }
}