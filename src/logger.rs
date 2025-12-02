pub struct Logger {
    indent: usize,
}

impl Logger {
    pub fn new() -> Self {
        Self { indent: 0 }
    }

    fn print(&self, text: &str) {
        println!("{}{}", "  ".repeat(self.indent), text);
    }

    pub fn nested<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.indent += 1;
        f(self);
        self.indent -= 1;
    }

    pub fn path<F>(&mut self, path: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.print(&format!("📍 Path: {}", path));
        self.nested(f);
    }

    pub fn method<F>(&mut self, method: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.print(&format!("▶ Method: {}", method));
        self.nested(f);
    }

    pub fn status<F>(&mut self, code: u16, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let icon = if (200..300).contains(&code) {
            "🟢"
        } else {
            "🔶"
        };
        self.print(&format!("{} {}", icon, code));
        self.nested(f);
    }

    pub fn params<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.print("🔹 Params:");
        self.nested(f);
    }

    pub fn body<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.print("🔹 Body:");
        self.nested(f);
    }

    pub fn response<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.print("🔹 Response:");
        self.nested(f);
    }

    pub fn field(&self, name: &str, typ: &str) {
        self.print(&format!("  • {}: {}", name, typ));
    }
}
