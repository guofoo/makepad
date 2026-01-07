use crate::{
    makepad_widgets::*,
};

live_design!{
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::layout_templates::*;
    use crate::accordion::*;

    pub DemoAccordion = <UIZooTabLayout_B> {
        desc = {
            <Markdown> { body: dep("crate://self/resources/accordion.md") }
        }
        demos = {
            <H4> { text: "Standard Accordion" }
            <Accordion> {
                width: Fill, height: Fit
                <AccordionItem> {
                    group: demo_group
                    header: <Button> { 
                        text: "Section 1", 
                        width: Fill,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "Content for section 1\nThis is a standard accordion item.", padding: 10.0 }
                }
                <AccordionItem> {
                    group: demo_group
                    header: <Button> { 
                        text: "Section 2", 
                        width: Fill,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "Content for section 2\nIt opens exclusively.", padding: 10.0 }
                }
                <AccordionItem> {
                    group: demo_group
                    header: <Button> { 
                        text: "Section 3", 
                        width: Fill,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "Content for section 3\nOnly one can be open.", padding: 10.0 }
                }
            }
            
            <Hr> {}
            <H4> { text: "Standard, disabled" }
            <Accordion> {
                width: Fill, height: Fit
                <AccordionItem> {
                    group: disabled_group
                    draw_bg: {
                        color: (THEME_COLOR_OUTSET_DISABLED)
                        border_color: (THEME_COLOR_BEVEL_DISABLED)
                        border_width: 1.0
                        border_radius: 4.0
                    }
                    header: <Button> { 
                        text: "Disabled Section", 
                        width: Fill,
                        enabled: false,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "This section looks disabled.", padding: 10.0 }
                }
            }

            <Hr> {}
            <H4> { text: "GradientX" }
            <Accordion> {
                width: Fill, height: Fit
                <AccordionItem> {
                    group: gradient_x_group
                    draw_bg: {
                        color: (THEME_COLOR_OUTSET_1)
                        color_2: (THEME_COLOR_OUTSET_2)
                        gradient_fill_horizontal: 0.0
                        border_color: (THEME_COLOR_BEVEL_OUTSET_1)
                        border_width: 1.0
                        border_radius: 4.0
                    }
                    header: <Button> { 
                        text: "GradientX Section", 
                        width: Fill,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "Vertical gradient background.", padding: 10.0 }
                }
            }

            <Hr> {}
            <H4> { text: "GradientY" }
            <Accordion> {
                width: Fill, height: Fit
                <AccordionItem> {
                    group: gradient_y_group
                    draw_bg: {
                        color: (THEME_COLOR_OUTSET_1)
                        color_2: (THEME_COLOR_OUTSET_2)
                        gradient_fill_horizontal: 1.0
                        border_color: (THEME_COLOR_BEVEL_OUTSET_1)
                        border_width: 1.0
                        border_radius: 4.0
                    }
                    header: <Button> { 
                        text: "GradientY Section", 
                        width: Fill,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "Horizontal gradient background.", padding: 10.0 }
                }
            }

            <Hr> {}
            <H4> { text: "Flat" }
            <Accordion> {
                width: Fill, height: Fit
                <AccordionItem> {
                    group: flat_group
                    draw_bg: {
                        color: (THEME_COLOR_OUTSET)
                        border_width: 0.0
                        border_radius: 4.0
                    }
                    header: <ButtonFlat> { 
                        text: "Flat Section", 
                        width: Fill,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "Flat background, no border.", padding: 10.0 }
                }
            }

            <Hr> {}
            <H4> { text: "Flatter" }
            <Accordion> {
                width: Fill, height: Fit
                <AccordionItem> {
                    group: flatter_group
                    draw_bg: {
                        color: #0000
                        border_width: 0.0
                    }
                    header: <ButtonFlatter> { 
                        text: "Flatter Section", 
                        width: Fill,
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "Transparent background.", padding: 10.0 }
                }
            }
            
            <Hr> {}
            <H4> { text: "Styled Accordion" }
            <Accordion> {
                width: Fill, height: Fit
                <AccordionItem> {
                    group: styled_group
                    draw_bg: {
                        color: #333
                        border_color: #666
                        border_width: 1.0
                        border_radius: 4.0
                    }
                    header: <Button> { 
                        text: "Styled Section 1", 
                        width: Fill,
                        draw_bg: { color: #444 }
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "This item has a custom background and border.", padding: 10.0, draw_text: { color: #fff } }
                }
                <AccordionItem> {
                    group: styled_group
                    draw_bg: {
                        color: #333
                        border_color: #666
                        border_width: 1.0
                        border_radius: 4.0
                    }
                    header: <Button> { 
                        text: "Styled Section 2", 
                        width: Fill,
                        draw_bg: { color: #444 }
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {
                                font_size: (THEME_FONT_SIZE_P)
                            }
                        }
                    }
                    body: <Label> { text: "You can customize colors and borders.", padding: 10.0, draw_text: { color: #fff } }
                }
            }
        }
    }
}
