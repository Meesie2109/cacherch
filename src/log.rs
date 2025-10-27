use console::Style;

pub struct LogStyle;

impl LogStyle {
    pub fn info(msg: &str) -> String {
        let tag = Style::new().blue().bold();
        let text = Style::new();
        format!("{} {}", tag.apply_to("[Info]"), text.apply_to(msg))
    }

    pub fn success(msg: &str) -> String {
        let tag = Style::new().green().bold();
        let text = Style::new();
        format!("{} {}", tag.apply_to("[Success]"), text.apply_to(msg))
    }

    pub fn warning(msg: &str) -> String {
        let tag = Style::new().yellow().bold();
        let text = Style::new();
        format!("{} {}", tag.apply_to("[Warning]"), text.apply_to(msg))
    }

    pub fn error(msg: &str) -> String {
        let tag = Style::new().red().bold();
        let text = Style::new();
        format!("{} {}", tag.apply_to("[Error]"), text.apply_to(msg))
    }
}
