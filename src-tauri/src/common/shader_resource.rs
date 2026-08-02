#[derive(Debug, Default, Clone)]
pub struct ShaderResource {
    pub index: String,
    pub resource: String,
    pub view: String,
    pub hash: String,
}

impl ShaderResource {
    pub fn new(log_line: &str) -> Self {
        let mut res = ShaderResource::default();

        let trimmed = log_line.trim();
        if trimmed.is_empty() {
            return res;
        }

        let mut parts = trimmed.splitn(2, ':');
        if let Some(idx) = parts.next() {
            res.index = idx.to_string();
        }

        let arguments = parts.next().map(str::trim).unwrap_or("");
        if arguments.is_empty() {
            return res;
        }

        for key_value_str in arguments.split_whitespace() {
            let mut kv = key_value_str.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim().to_lowercase();
            let value = kv.next().unwrap_or("").trim().to_string();

            match key.as_str() {
                "resource" => res.resource = value,
                "hash" => res.hash = value,
                "view" => res.view = value,
                _ => {}
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_line() {
        let line = "12: resource=myres hash=abc123 view=main";
        let s = ShaderResource::new(line);
        assert_eq!(s.index, "12");
        assert_eq!(s.resource, "myres");
        assert_eq!(s.hash, "abc123");
        assert_eq!(s.view, "main");
    }

    #[test]
    fn parse_missing_args() {
        let line = "7:";
        let s = ShaderResource::new(line);
        assert_eq!(s.index, "7");
        assert!(s.resource.is_empty());
        assert!(s.hash.is_empty());
        assert!(s.view.is_empty());
    }
}
