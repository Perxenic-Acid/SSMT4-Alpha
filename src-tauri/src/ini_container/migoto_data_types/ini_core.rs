use crate::utils::ssmt_string_utils::SSMTStringUtils;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IniSectionType {
    Present,
    Constants,
    Key,
    TextureOverrideIB,
    TextureOverrideVB,
    TextureOverrideTexture,
    IBSkip,
    ResourceVB,
    ResourceIB,
    ResourceTexture,
    CreditInfo,
    VSHashCheck,
}

impl Default for IniSectionType {
    fn default() -> Self {
        Self::Present
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExpressionValue {
    pub expression_original_line: String,
    pub expression_list: Vec<String>,
    pub is_pure_value: bool,
}

impl ExpressionValue {
    pub fn new(expression_string_line: impl Into<String>) -> Self {
        let expression_string_line = expression_string_line.into();
        let mut out = Self {
            expression_original_line: expression_string_line.clone(),
            expression_list: Vec::new(),
            is_pure_value: true,
        };

        if expression_string_line.contains('+') || expression_string_line.contains('-') {
            out.is_pure_value = false;
        }

        if !out.is_pure_value {
            out.parse_expression();
        }

        out
    }

    pub fn parse_expression(&mut self) {
        let bytes = self.expression_original_line.as_bytes();
        if bytes.is_empty() {
            return;
        }

        let mut last_split_index: usize = 0;

        for i in 0..bytes.len() {
            let last_ch = if i == 0 { b'L' } else { bytes[i - 1] };
            let next_ch = if i + 1 >= bytes.len() {
                b'N'
            } else {
                bytes[i + 1]
            };
            let ch = bytes[i];

            // Keep C++ precedence-compatible behavior:
            // '+' always splits; '-' splits only when surrounded by spaces.
            if ch == b'+' || (ch == b'-' && last_ch == b' ' && next_ch == b' ') {
                let token = self.expression_original_line[last_split_index..i].trim();
                let token = SSMTStringUtils::remove_dollar_prefix(token);
                if !token.is_empty() {
                    self.expression_list.push(token);
                }

                self.expression_list.push((ch as char).to_string());
                last_split_index = i + 1;
            }
        }

        let tail = self.expression_original_line[last_split_index..].trim();
        let tail = SSMTStringUtils::remove_dollar_prefix(tail);
        if !tail.is_empty() {
            self.expression_list.push(tail);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IniLineObject {
    pub left_str: String,
    pub left_str_trim: String,
    pub right_str: String,
    pub right_str_trim: String,
    pub valid: bool,
}

impl IniLineObject {
    pub fn new(read_line: impl Into<String>) -> Self {
        let read_line = read_line.into();
        let delimiter = if read_line.contains("==") { "==" } else { "=" };
        Self::with_delimiter(read_line, delimiter)
    }

    pub fn with_delimiter(read_line: impl Into<String>, delimiter: &str) -> Self {
        let read_line = read_line.into();
        if let Some((left, right)) = SSMTStringUtils::split_once_owned(&read_line, delimiter) {
            let left_trim = SSMTStringUtils::trim_owned(&left);
            let right_trim = SSMTStringUtils::trim_owned(&right);
            return Self {
                left_str: left,
                left_str_trim: left_trim,
                right_str: right,
                right_str_trim: right_trim,
                valid: true,
            };
        }

        Self::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MigotoAttribute {
    /// Physical directory used to resolve resource filenames.
    pub namespace: String,
    /// 3Dmigoto logical namespace used by cross-file references.
    pub logical_namespace: String,
}

#[derive(Debug, Clone, Default)]
pub struct IniSection {
    pub attr: MigotoAttribute,
    pub section_type: IniSectionType,
    pub section_name: String,
    pub section_line_list: Vec<String>,
}

impl IniSection {
    pub fn new(section_type: IniSectionType) -> Self {
        Self {
            section_type,
            ..Self::default()
        }
    }

    pub fn append(&mut self, input_line: impl Into<String>) {
        self.section_line_list.push(input_line.into());
    }

    pub fn new_line(&mut self) {
        self.section_line_list.push(String::new());
    }
}
