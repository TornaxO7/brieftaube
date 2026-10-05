use material_theme_loader::Scheme;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::Block,
};

use crate::utils::IntoColor;

pub fn view(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Vec<crate::ui::Message> {
    let area = area.centered(Constraint::Percentage(50), Constraint::Length(3));

    state.input.set_block(
        Block::bordered()
            .title(state.description.clone())
            .title_style(scheme.primary.into_color())
            .border_style(scheme.outline.into_color()),
    );

    frame.render_widget(&state.input, area);

    vec![]
}
