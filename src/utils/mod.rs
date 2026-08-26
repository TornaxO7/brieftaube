pub trait IntoColor {
    fn into_color(&self) -> ratatui::style::Color;
}

impl IntoColor for material_theme_loader::Rgb {
    fn into_color(&self) -> ratatui::style::Color {
        let material_theme_loader::Rgb { r, g, b } = *self;
        ratatui::style::Color::Rgb(r, g, b)
    }
}
