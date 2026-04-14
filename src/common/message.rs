use druid::Color;

#[derive(Clone, Copy)]
pub enum MessageType {
    Info,
    Error,
}

impl MessageType {
    pub fn color(self) -> Color {
        match self {
            MessageType::Info => Color::GREEN,
            MessageType::Error => Color::RED,
        }
    }
}
