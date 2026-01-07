use makepad_widgets::*;

live_design!{
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    

    pub AccordionItemBase = {{AccordionItem}} {}
    pub AccordionBase = {{Accordion}} {}

    pub AccordionItem = <AccordionItemBase> {
        width: Fill, height: Fit,
        flow: Down,
        
        draw_bg: {
            uniform border_radius: 0.0
            uniform border_width: 0.0
            uniform border_color: #0000
            uniform color: #0000
            
            uniform color_2: #0000
            uniform gradient_fill_horizontal: 0.0
            
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.border_radius
                )
                
                let gradient_fill_dir = self.pos.y;
                if (self.gradient_fill_horizontal > 0.5) {
                    gradient_fill_dir = self.pos.x;
                }
                
                let fill_color = mix(self.color, self.color_2, gradient_fill_dir);
                if (self.color_2.a == 0.0) {
                    fill_color = self.color;
                }
                
                sdf.fill_keep(fill_color)
                sdf.stroke(self.border_color, self.border_width)
                return sdf.result
            }
        }
        animator: {
            active = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.2}}
                    ease: ExpDecay {d1: 0.96, d2: 0.97}
                    redraw: true
                    apply: {
                        opened: [{time: 0.0, value: 1.0}, {time: 1.0, value: 0.0}]
                    }
                }
                on = {
                    from: {all: Forward {duration: 0.2}}
                    ease: ExpDecay {d1: 0.98, d2: 0.95}
                    redraw: true
                    apply: {
                        opened: [{time: 0.0, value: 0.0}, {time: 1.0, value: 1.0}]
                    }
                }
            }
        }
    }

    pub Accordion = <AccordionBase> {
        width: Fill, height: Fit,
        flow: Down,
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct AccordionItem {
    #[rust] draw_state: DrawStateWrap<DrawState>,
    #[rust] rect_size: f64,
    #[rust] area: Area,
    
    #[live] draw_bg: DrawQuad,
    
    #[find] #[redraw] #[live] header: WidgetRef,
    #[find] #[redraw] #[live] body: WidgetRef,
    
    #[animator] animator: Animator,

    #[live] opened: f64,
    #[live] group: LiveId,
    #[layout] layout: Layout,
    #[walk] walk: Walk,
}

#[derive(Clone)]
enum DrawState {
    DrawHeader,
    DrawBody
}

#[derive(Clone, DefaultNone, Debug, PartialEq)]
pub enum AccordionItemAction {
    Opening { group: LiveId, uid: WidgetUid },
    CloseOthers { group: LiveId, keeping: WidgetUid },
    Closing,
    None,
}

impl Widget for AccordionItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            if self.animator.is_track_animating(cx, id!(active)) {
                self.area.redraw(cx);
            }
        };
        
        self.header.handle_event(cx, event, scope);
        self.body.handle_event(cx, event, scope);
        
        if let Event::Actions(actions) = event {
            // Handle actions from OTHER items (via Accordion mediator)
            for action in actions {
                if let Some(AccordionItemAction::CloseOthers { group, keeping }) = action.cast() {
                    log!("Item {:?} (group {:?}) received CloseOthers(keeping={:?}, group={:?})", self.widget_uid(), self.group, keeping, group);
                    if group == self.group && keeping != self.widget_uid() {
                        log!("Item {:?} closing because {:?} opened in group {:?}", self.widget_uid(), keeping, group);
                        self.close(cx);
                    }
                }
            }
            
            if let Some(item) = actions.find_widget_action(self.header.widget_uid()) {
                if let ButtonAction::Clicked(_) = item.cast() {
                    if self.opened > 0.5 {
                        self.animator_play(cx, id!(active.off));
                        cx.widget_action(self.widget_uid(), &scope.path, AccordionItemAction::Closing);
                    } else {
                        self.animator_play(cx, id!(active.on));
                        log!("Item {:?} opening in group {:?} -> Emitting Opening", self.widget_uid(), self.group);
                        cx.widget_action(self.widget_uid(), &scope.path, AccordionItemAction::Opening {
                            group: self.group,
                            uid: self.widget_uid()
                        });
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // log!("AccordionItem draw_walk: uid={:?}, group={:?}", self.widget_uid(), self.group);
        if self.draw_state.begin(cx, DrawState::DrawHeader) {
            self.draw_bg.begin(cx, walk, self.layout);
        }
        
        if let Some(DrawState::DrawHeader) = self.draw_state.get() {
            let walk = self.header.walk(cx);
            self.header.draw_walk(cx, scope, walk)?;
            
            // Draw body with clipping/scrolling based on opened state
            // We use the previously measured rect_size to determine the height
            let current_height = self.rect_size * self.opened;
            
            // Start a turtle for the body container with the animated height and clipping
            cx.begin_turtle(
                Walk {
                    width: Size::fill(),
                    height: Size::Fixed(current_height),
                    margin: Margin::default(),
                    abs_pos: None,
                },
                Layout {
                    clip_x: true,
                    clip_y: true,
                    ..Layout::flow_down()
                }
            );
            
            self.draw_state.set(DrawState::DrawBody);
        }
        
        if let Some(DrawState::DrawBody) = self.draw_state.get() {
            // Draw the actual body content
            let body_walk = self.body.walk(cx);
            self.body.draw_walk(cx, scope, body_walk)?;
            
            // Update rect_size with the actual height of the body content
            self.rect_size = cx.turtle().used().y;
            
            // End the body container turtle
            cx.end_turtle();
            
            // End the main turtle (draw_bg)
            self.draw_bg.end(cx);
            self.area = self.draw_bg.area();
            self.draw_state.end();
        }
        
        DrawStep::done()
    }
}

impl AccordionItem {
    pub fn close(&mut self, cx: &mut Cx) {
        self.animator_play(cx, id!(active.off));
        self.redraw(cx);
    }
    
    pub fn open(&mut self, cx: &mut Cx) {
        self.animator_play(cx, id!(active.on));
    }
    
    pub fn group(&self) -> LiveId {
        self.group
    }
}

impl AccordionItemRef {
    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
        }
    }
    
    pub fn open(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open(cx);
        }
    }
    
    pub fn group(&self) -> Option<LiveId> {
        if let Some(inner) = self.borrow() {
            Some(inner.group())
        } else {
            None
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Accordion {
    #[deref] view: View,
}

impl Widget for Accordion {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Event::Actions(actions) = event {
            for action in actions {
                if let Some(AccordionItemAction::Opening { group, uid }) = action.cast() {
                    log!("Accordion saw Opening from {:?} (group {:?}) -> Dispatching CloseOthers", uid, group);
                    cx.widget_action(self.widget_uid(), &scope.path, AccordionItemAction::CloseOthers { group, keeping: uid });
                }
            }
        }
        self.view.handle_event(cx, event, scope);
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
