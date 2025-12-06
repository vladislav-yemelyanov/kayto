pub struct Logger {
    indent: usize,
}

impl Logger {
    pub fn new() -> Self {
        Self { indent: 0 }
    }

    pub fn increase_indent(&mut self) {
        self.indent += 1;
    }

    pub fn decrease_indent(&mut self) {
        self.indent -= 1;
    }

    fn print(&mut self, text: &str) {
        println!("{}{}", "  ".repeat(self.indent), text);
    }

    pub fn path(&mut self, path: &str) {
        self.print(&format!("📍 Path: {}", path));
    }

    pub fn method(&mut self, method: &str) {
        self.print(&format!("▶ Method: {}", method));
    }

    pub fn status(&mut self, code: u16) {
        let icon = if (200..300).contains(&code) {
            "🟢"
        } else {
            "🔶"
        };
        self.print(&format!("{} {}", icon, code));
    }

    pub fn params(&mut self) {
        self.print("🔹 Params:");
    }

    pub fn body(&mut self) {
        self.print("🔹 Body:");
    }

    pub fn responses(&mut self) {
        self.print("🔹 Responses:");
    }

    pub fn field(&mut self, name: &str, typ: &str) {
        self.print(&format!("  • {}: {}", name, typ));
    }
}
