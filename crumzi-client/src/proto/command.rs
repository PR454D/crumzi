#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    name: &'static str,
    args: Vec<String>,
}

impl Command {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl AsRef<str>) -> Self {
        self.args.push(escape_arg(arg.as_ref()));
        self
    }

    pub fn to_line(&self) -> String {
        let mut s = String::from(self.name);
        for a in &self.args {
            s.push(' ');
            s.push_str(a);
        }
        s
    }
}

fn escape_arg(arg: &str) -> String {
    let needs_quotes = arg
        .chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '\\');

    if !needs_quotes {
        return arg.to_string();
    }

    let mut out = String::with_capacity(arg.len() + 2);
    out.push('"');
    for ch in arg.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_line_no_args() {
        assert_eq!(Command::new("status").to_line(), "status");
    }

    #[test]
    fn command_line_with_args_and_escaping() {
        let cmd = Command::new("add").arg("a b").arg(r#"x"y"#).arg(r#"c\d"#);
        assert_eq!(cmd.to_line(), r#"add "a b" "x\"y" "c\\d""#);
    }
}

