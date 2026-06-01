#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Timer,
    Forest,
    Stats,
}

impl Tab {
    pub fn next(&self) -> Self {
        match self {
            Tab::Timer => Tab::Forest,
            Tab::Forest => Tab::Stats,
            Tab::Stats => Tab::Timer,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Tab::Timer => Tab::Stats,
            Tab::Forest => Tab::Timer,
            Tab::Stats => Tab::Forest,
        }
    }
}

pub enum TabEvent {
    Up { is_ctrl: bool },
    Down { is_ctrl: bool },
    Left,
    Right,
    Enter,
    Esc,
}

pub enum EventResult {
    Consumed,
    Ignored,
}
