use crate::{
    makepad_widgets::*,
};

live_design!{
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::layout_templates::*;
    use crate::layout_templates::*;
    use crate::datepicker::*;
    use crate::datepicker::*;

    pub DemoDatePicker = <UIZooTabLayout_B> {
        desc = {
            <Markdown> { body: dep("crate://self/resources/datepicker.md") }
        }
        demos = {
            <H4> { text: "Standard DatePicker" }
            <UIZooRowH> {
                <DatePicker> {
                    width: Fit, height: Fit
                }
            }
            
        }
    }
}
