#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Timer,
    Forest,
    Stats,
    Plants,
}

impl Tab {
    pub fn next(&self, is_production: bool) -> Self {
        match self {
            Tab::Timer => Tab::Forest,
            Tab::Forest => Tab::Stats,
            Tab::Stats => {
                if is_production {
                    Tab::Timer
                } else {
                    Tab::Plants
                }
            },
            Tab::Plants => Tab::Timer,
        }
    }

    pub fn previous(&self, is_production: bool) -> Self {
        match self {
            Tab::Timer => {
                if is_production {
                    Tab::Stats
                } else {
                    Tab::Plants
                }
            },
            Tab::Forest => Tab::Timer,
            Tab::Stats => Tab::Forest,
            Tab::Plants => Tab::Stats,
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
    Tab,
}

pub enum EventResult {
    Consumed,
    Ignored,
}
