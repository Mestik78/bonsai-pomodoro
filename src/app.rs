pub struct App {
    pub current_tab: usize,
    pub should_quit: bool,
    pub tab_titles: Vec<&'static str>,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            should_quit: false,
            // Asumiendo los nombres por defecto como discutimos
            tab_titles: vec!["Temporizador", "Bosque"],
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tab_titles.len();
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.tab_titles.len() - 1;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
