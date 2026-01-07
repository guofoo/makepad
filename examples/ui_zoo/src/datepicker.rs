use crate::{
    makepad_widgets::*,
    makepad_widgets::makepad_draw::*,
};

live_design!{
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    pub DatePickerBase = {{DatePicker}} {
        width: Fit, height: Fit
        flow: Down
        padding: 10.0
        spacing: 10.0
        
        draw_bg: {
            color: #333
        }

        header: <View> {
            width: Fill, height: 30.0
            flow: Right
            spacing: 10.0
            padding: {bottom: 10.0}
            align: {x: 0.5, y: 0.5}
            
            prev_button = <Button> {
                width: 30.0, height: 30.0
                text: "<"
            }
            
            month_year_label = <Label> {
                width: Fit, height: Fit
                draw_text: {
                    text_style: <THEME_FONT_BOLD> {
                        font_size: 14.0
                    }
                    color: #fff
                }
                text: "Month Year"
            }
            
            next_button = <Button> {
                width: 30.0, height: 30.0
                text: ">"
            }
        }

        weekdays: <View> {
            width: Fit, height: Fit
            flow: Right
            spacing: 5.0
            padding: {bottom: 5.0}
            
            <Label> { width: 30.0, height: 30.0, text: "Su", align: {x: 1.0, y: 0.5}, padding: {right: 4.0}, draw_text: {color: #aaa} }
            <Label> { width: 30.0, height: 30.0, text: "Mo", align: {x: 1.0, y: 0.5}, padding: {right: 4.0}, draw_text: {color: #aaa} }
            <Label> { width: 30.0, height: 30.0, text: "Tu", align: {x: 1.0, y: 0.5}, padding: {right: 4.0}, draw_text: {color: #aaa} }
            <Label> { width: 30.0, height: 30.0, text: "We", align: {x: 1.0, y: 0.5}, padding: {right: 4.0}, draw_text: {color: #aaa} }
            <Label> { width: 30.0, height: 30.0, text: "Th", align: {x: 1.0, y: 0.5}, padding: {right: 4.0}, draw_text: {color: #aaa} }
            <Label> { width: 30.0, height: 30.0, text: "Fr", align: {x: 1.0, y: 0.5}, padding: {right: 4.0}, draw_text: {color: #aaa} }
            <Label> { width: 30.0, height: 30.0, text: "Sa", align: {x: 1.0, y: 0.5}, padding: {right: 4.0}, draw_text: {color: #aaa} }
        }

        days_grid: <View> {
            width: Fit, height: Fit
            flow: Right
            spacing: 5.0
        }
        
        day_button_template: <Button> {
            width: 30.0, height: 30.0
            padding: {right: 4.0}
            align: {x: 1.0, y: 0.5}
            draw_text: {
                text_style: { font_size: 10.0 }
            }
            draw_bg: {
                color: #0000
                border_size: 0.0
                instance color_hover: #FFFFFFEE
            }
        }
    }
    
    pub DatePicker = <DatePickerBase> {}
}

use std::time::Instant;

#[derive(Live, Widget)]
pub struct DatePicker {
    #[deref] view: View,
    
    #[live] #[area] #[redraw] draw_bg: DrawColor,
    
    #[live] header: View,
    #[live] weekdays: View,
    #[live] days_grid: View,
    
    #[rust] displayed_month: i32, // 1-12
    #[rust] displayed_year: i32,
    #[rust] selected_date: Option<(i32, i32, i32)>, // year, month, day
    
    #[live] day_button_template: Option<LivePtr>,
    #[rust] day_items: ComponentMap<LiveId, WidgetRef>,
    #[rust] last_prev_time: Option<Instant>,
    #[rust] last_next_time: Option<Instant>,
}

#[derive(Clone, DefaultNone, Debug, PartialEq)]
pub enum DatePickerAction {
    DateSelected { year: i32, month: i32, day: i32 },
    None,
}

impl Widget for DatePicker {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Initialize defaults if needed (to prevent jumps from 0)
        if self.displayed_year == 0 {
            self.displayed_year = 2023;
            self.displayed_month = 11;
        }

        // Handle prev/next buttons
        let uid = self.widget_uid();
        
        if let Event::Actions(actions) = event {
            let prev_btn = self.header.button(id!(prev_button));
            let next_btn = self.header.button(id!(next_button));
            
            // log!("PrevBtn UID: {:?}, NextBtn UID: {:?}", prev_btn.widget_uid(), next_btn.widget_uid());
            
            if prev_btn.clicked(actions) {
                let now = Instant::now();
                let mut should_process = true;
                if let Some(last) = self.last_prev_time {
                    if now.duration_since(last).as_millis() < 200 {
                        should_process = false;
                    }
                }
                if should_process {
                    self.last_prev_time = Some(now);
                    self.prev_month(cx);
                }
            }
            if next_btn.clicked(actions) {
                let now = Instant::now();
                let mut should_process = true;
                if let Some(last) = self.last_next_time {
                    if now.duration_since(last).as_millis() < 200 {
                        should_process = false;
                    }
                }
                if should_process {
                    self.last_next_time = Some(now);
                    self.next_month(cx);
                }
            }
            
            // Handle day clicks
            let days_in_mo = days_in_month(self.displayed_month, self.displayed_year);
            
            let mut selected_day = None;
            
            for (id, widget) in self.day_items.iter() {
                let day = id.0 as i32;
                if day >= 1 && day <= days_in_mo {
                    if widget.as_button().clicked(actions) {
                        selected_day = Some(day);
                    }
                }
                widget.handle_event(cx, event, scope);
            }
            
            if let Some(day) = selected_day {
                self.selected_date = Some((self.displayed_year, self.displayed_month, day));
                cx.widget_action(uid, &scope.path, DatePickerAction::DateSelected {
                    year: self.displayed_year,
                    month: self.displayed_month,
                    day
                });
                self.redraw(cx);
            }
        }
        
        self.header.handle_event(cx, event, scope);
        self.weekdays.handle_event(cx, event, scope);
        self.days_grid.handle_event(cx, event, scope);
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Initialize defaults if needed
        if self.displayed_year == 0 {
            self.displayed_year = 2023;
            self.displayed_month = 11;
        }
        
        let layout = self.layout;
        self.draw_bg.begin(cx, walk, layout);
        
        // Draw Header
        self.header.draw_walk(cx, scope, Walk::fit()).unwrap();
        
        // Update Label
        let label_text = format!("{} {}", month_name(self.displayed_month), self.displayed_year);
        self.header.label(id!(month_year_label)).set_text(cx, &label_text);
        
        // Draw Weekdays
        self.weekdays.draw_walk(cx, scope, Walk::fit()).unwrap();
        
        // Draw Days Grid
        self.days_grid.draw_walk(cx, scope, Walk::default()).unwrap();
        let grid_layout = self.days_grid.layout;
        self.days_grid.draw_bg.begin(cx, Walk::default(), grid_layout);
        
        let days_in_mo = days_in_month(self.displayed_month, self.displayed_year);
        let start_day = day_of_week(1, self.displayed_month, self.displayed_year); // 0=Sun
        
        // Draw spacer for empty slots at the start
        if start_day > 0 {
            // 30.0 (button width) + 5.0 (spacing)
            let spacer_width = (start_day as f64) * 35.0;
            cx.walk_turtle(Walk {
                width: Size::Fixed(spacer_width),
                height: Size::Fixed(30.0),
                ..Walk::default()
            });
        }
        
        for day in 1..=31 {
            let item_id = LiveId(day as u64);
            let template = self.day_button_template;
            let widget = self.day_items.get_or_insert(cx, item_id, |cx|{
                WidgetRef::new_from_ptr(cx, template)
            });
            
            if day <= days_in_mo {
                let col = (start_day + day - 1) % 7;
                if col == 0 && day > 1 {
                    cx.turtle_new_line();
                }
                
                widget.set_visible(cx, true);
                widget.set_text(cx, &day.to_string());
                
                // Highlight selected date
                if let Some((y, m, d)) = self.selected_date {
                    if y == self.displayed_year && m == self.displayed_month && d == day {
                        widget.apply_over(cx, live!{
                            draw_bg: { color: #x500 }
                            draw_text: { color: #fff }
                        });
                    } else {
                        // Reset style (important if reusing widgets)
                        widget.apply_over(cx, live!{
                            draw_bg: { color: #x0000 }
                            draw_text: { color: #fff }
                        });
                    }
                }
                
                widget.draw_walk(cx, scope, Walk {
                    width: Size::Fixed(30.0),
                    height: Size::Fixed(30.0),
                    ..Walk::default()
                }).unwrap();
            } else {
                widget.set_visible(cx, false);
                widget.draw_walk(cx, scope, Walk {
                    width: Size::Fixed(30.0),
                    height: Size::Fixed(30.0),
                    ..Walk::default()
                }).unwrap();
            }
        }
        
        self.days_grid.draw_bg.end(cx);
        
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl LiveHook for DatePicker {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        if self.displayed_year == 0 {
            self.displayed_year = 2023;
            self.displayed_month = 11;
        }
    }
}

impl DatePicker {
    fn prev_month(&mut self, cx: &mut Cx) {
        self.displayed_month -= 1;
        if self.displayed_month < 1 {
            self.displayed_month = 12;
            self.displayed_year -= 1;
        }
        self.redraw(cx);
    }
    
    fn next_month(&mut self, cx: &mut Cx) {
        self.displayed_month += 1;
        if self.displayed_month > 12 {
            self.displayed_month = 1;
            self.displayed_year += 1;
        }
        self.redraw(cx);
    }
}

// Helpers
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(month: i32, year: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

// Zeller's congruence for day of week (0=Sun, 1=Mon, ..., 6=Sat)
fn day_of_week(day: i32, month: i32, year: i32) -> i32 {
    let mut m = month;
    let mut y = year;
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = y / 100;
    let h = (day + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // h is 0=Sat, 1=Sun, ...
    // Convert to 0=Sun, 1=Mon
    (h + 6) % 7
}

fn month_name(month: i32) -> &'static str {
    match month {
        1 => "January", 2 => "February", 3 => "March", 4 => "April",
        5 => "May", 6 => "June", 7 => "July", 8 => "August",
        9 => "September", 10 => "October", 11 => "November", 12 => "December",
        _ => "Unknown",
    }
}
