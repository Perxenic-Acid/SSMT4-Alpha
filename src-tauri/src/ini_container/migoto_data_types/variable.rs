use super::ini_core::{ExpressionValue, MigotoAttribute};

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub attr: MigotoAttribute,
    pub variable_name: String,
    pub initialize_value: String,
    pub namespaced_var_name: String,
    pub var_type: String,
    pub expression_value: ExpressionValue,
}

impl Variable {
    pub fn new(
        namespace: impl Into<String>,
        variable_name: impl Into<String>,
        var_type: impl Into<String>,
    ) -> Self {
        let namespace = namespace.into();
        let variable_name = variable_name.into();
        let namespaced_var_name = format!("{}\\{}", namespace, variable_name);
        Self {
            attr: MigotoAttribute {
                namespace: namespace.clone(),
                logical_namespace: namespace,
            },
            variable_name,
            namespaced_var_name,
            var_type: var_type.into(),
            ..Self::default()
        }
    }

    pub fn with_value(
        namespace: impl Into<String>,
        variable_name: impl Into<String>,
        initialize_value: impl Into<String>,
        var_type: impl Into<String>,
    ) -> Self {
        let mut out = Self::new(namespace, variable_name, var_type);
        out.initialize_value = initialize_value.into();
        out
    }

    pub fn with_expression(
        namespace: impl Into<String>,
        variable_name: impl Into<String>,
        expression_value: ExpressionValue,
        var_type: impl Into<String>,
    ) -> Self {
        let mut out = Self::new(namespace, variable_name, var_type);
        out.expression_value = expression_value;
        out
    }
}
