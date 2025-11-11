#[derive(Clone, Debug)]
pub struct HomeModel {
    pub options: Vec<String>,
    pub selected_option_index: i8,
}

impl HomeModel {
    pub fn new() -> HomeModel {
        let main_screen_options: Vec<String> = vec![
            "Backup DB",
            "Restore DB",
            "Add DB Connection",
            "Add Storage Provider",
        ]
        .iter()
        .map(|it| it.to_string())
        .collect();

        HomeModel {
            options: main_screen_options,
            selected_option_index: 0,
        }
    }

    pub fn select_next(&mut self) {
        self.selected_option_index = self.selected_option_index + 1;

        if self.selected_option_index >= self.options.len() as i8 {
            self.selected_option_index = 0;
        }
    }

    pub fn select_previous(&mut self) {
        self.selected_option_index = self.selected_option_index - 1;

        if self.selected_option_index < 0 {
            self.selected_option_index = self.options.len() as i8 - 1;
        }
    }
}
