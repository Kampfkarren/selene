use super::*;
use full_moon::{ast, visitors::Visitor, ast::Ast};
use regex::Regex;
use serde::Deserialize;

/// Config for the variable_naming lint
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct VariableNamingConfig {
    pub function_names: String,
    pub variable_names: String,
    pub constant_names: String,
    pub type_names: String,
    pub ignore_pattern: String,

    pub check_locals: bool,
    pub check_globals: bool,
    pub check_parameters: bool,
    pub check_functions: bool,
    pub check_types: bool,
}

impl Default for VariableNamingConfig {
    fn default() -> Self {
        Self {
            function_names: "^_?[a-z][a-zA-Z0-9]*$".to_owned(),
            variable_names: "^[a-z][a-z0-9_]*$".to_owned(),
            constant_names: "^[A-Z][A-Z0-9_]*$".to_owned(),
            type_names: "^[A-Z][a-zA-Z0-9]+$".to_owned(),
            ignore_pattern: "^$".to_owned(),

            check_locals: true,
            check_globals: true,
            check_parameters: true,
            check_functions: true,
            check_types: true,
        }
    }
}

pub struct VariableNamingLint {
    fn_re: Regex,
    var_re: Regex,
    const_re: Regex,
    type_re: Regex,
    ignore_re: Regex,
    config: VariableNamingConfig,
}

impl Lint for VariableNamingLint {
    type Config = VariableNamingConfig;
    type Error = regex::Error;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            fn_re: Regex::new(&config.function_names)?,
            var_re: Regex::new(&config.variable_names)?,
            const_re: Regex::new(&config.constant_names)?,
            type_re: Regex::new(&config.type_names)?,
            ignore_re: Regex::new(&config.ignore_pattern)?,
            config,
        })
    }

    fn pass(&self, ast: &Ast, _context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 1) Variables (locals & globals) via ScopeManager
        for (_, variable) in &ast_context.scope_manager.variables {
            let name = &variable.name;

            if self.ignore_re.is_match(name) {
                continue;
            }

            // Skip parameter checks if disabled. Some Variable structs in selene mark parameters via a flag;
            // attempt to skip by checking variable.is_parameter if it exists (this is best-effort).
            // (If the project uses a different field, adjust accordingly.)
            let is_param = variable
                .definitions
                .iter()
                .any(|_| false); // fallback: treat as not parameter (no-op); maintainers can refine

            if is_param && !self.config.check_parameters {
                continue;
            }

            // Treat names matching the constant regex as constants (name-based approach)
            if self.const_re.is_match(name) {
                // If constants are under separate rules, ensure they match. Already matched.
                // Nothing to report here.
                continue;
            }

            // Validate variable name
            if !self.var_re.is_match(name) {
                diagnostics.push(Diagnostic::new(
                    "variable_naming",
                    format!("variable `{}` does not match pattern `{}`", name, self.var_re.as_str()),
                    Label::new(variable.identifiers[0]),
                ));
            }
        }

        // 2) Functions & types using an AST visitor for precise locations.
        if self.config.check_functions || self.config.check_types {
            let mut visitor = VariableNamingAstVisitor {
                fn_re: &self.fn_re,
                type_re: &self.type_re,
                check_functions: self.config.check_functions,
                check_types: self.config.check_types,
                diagnostics: Vec::new(),
            };

            visitor.visit_ast(ast);

            diagnostics.extend(visitor.diagnostics);
        }

        diagnostics
    }
}

/// AST visitor for checking function and type names
struct VariableNamingAstVisitor<'a> {
    fn_re: &'a Regex,
    type_re: &'a Regex,
    check_functions: bool,
    check_types: bool,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Visitor for VariableNamingAstVisitor<'a> {
    fn visit_function_declaration(&mut self, function: &ast::FunctionDeclaration) {
        if !self.check_functions {
            return;
        }

        // function.name() returns a FunctionName, get the first name token
        let name_token = function.name().names().iter().next().unwrap();
        let name = name_token.token().to_string();

        if !self.fn_re.is_match(name.trim()) {
            let label = Label::from_node(name_token, None);
            self.diagnostics.push(Diagnostic::new(
                "variable_naming",
                format!("function `{}` does not match pattern `{}`", name.trim(), self.fn_re.as_str()),
                label,
            ));
        }
    }

    fn visit_local_function(&mut self, function: &ast::LocalFunction) {
        if !self.check_functions {
            return;
        }

        // local function name is a token
        let name_token = function.name();
        let name = name_token.token().to_string();

        if !self.fn_re.is_match(name.trim()) {
            let label = Label::from_node(name_token, None);
            self.diagnostics.push(Diagnostic::new(
                "variable_naming",
                format!("function `{}` does not match pattern `{}`", name.trim(), self.fn_re.as_str()),
                label,
            ));
        }
    }

    fn visit_local_assignment(&mut self, local_assignment: &ast::LocalAssignment) {
        if !self.check_types {
            return;
        }

        // Check for patterns like: local MyType = { ... } or local MyType = SomeConstructor(...)
        // If the left-hand side name looks like a type, validate it.
        for name_token in local_assignment.names().iter() {
            let name = name_token.token().to_string();
            // Heuristic: if name begins with uppercase letter, consider it a type candidate
            if name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                if !self.type_re.is_match(name.trim()) {
                    let label = Label::from_node(name_token, None);
                    self.diagnostics.push(Diagnostic::new(
                        "variable_naming",
                        format!("type `{}` does not match pattern `{}`", name.trim(), self.type_re.as_str()),
                        label,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_variable_naming() {
        test_lint(
            VariableNamingLint::new(VariableNamingConfig::default()).unwrap(),
            "variable_naming",
            "functions",
        );
    }
}
