use crate::markdown::MarkdownParser;

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub description: String,
}

#[derive(Debug)]
pub struct TemplateFunction {
    pub name: String,
    pub description: String,
}

#[derive(Debug)]
pub struct GlobalFunction {
    pub name: String,
    pub template: String,
    pub description: String,
}

/** Parsed variable and function definitions from `Manual.md`. */
#[derive(Debug)]
pub struct Manual {
    pub template_variables: Vec<Variable>,
    pub template_functions: Vec<TemplateFunction>,
    pub global_variables: Vec<Variable>,
    pub global_functions: Vec<GlobalFunction>,
}

impl Default for Manual {
    fn default() -> Self {
        let manual_parser = MarkdownParser::new(include_str!("../vendored/Manual.md"));

        let global_functions = manual_parser
            .list_bullet_points("### Global functions")
            .unwrap()
            .iter()
            .map(|section| {
                let (header, desc) = section.split_once("\n").unwrap();
                let (command, template) = header.trim().split_once(" ").unwrap();

                GlobalFunction {
                    name: command
                        .replace("*", "")
                        .replace("(", "")
                        .replace(")", "")
                        .trim()
                        .to_string(),
                    template: template.replace("`", "").trim().to_string(),
                    description: desc.trim().to_string(),
                }
            })
            .collect();

        let global_variables = manual_parser
            .list_bullet_points("### Global variables")
            .unwrap()
            .iter()
            .map(|section| {
                let (name, desc) = section.split_once(" ").unwrap();

                Variable {
                    name: name.replace("`", "").trim().to_string(),
                    description: desc.trim().to_string(),
                }
            })
            .collect();

        let template_functions = manual_parser
            .list_bullet_points("### Functions")
            .unwrap()
            .iter()
            .map(|section| {
                let (name, desc) = section.split_once(" ").unwrap();

                TemplateFunction {
                    name: name.replace("`", "").trim().to_string(),
                    description: desc.trim().to_string(),
                }
            })
            .collect();

        let template_variables = [
            "#### Mandatory variables",
            "#### Optional variables",
            "#### Python packages",
            "#### Go packages",
            "#### Haskell packages",
            "#### Font packages",
        ]
        .iter()
        .flat_map(|section_title| {
            manual_parser
                .list_bullet_points(section_title)
                .unwrap()
                .iter()
                .map(|section| {
                    let (name, desc) = section.split_once(" ").unwrap();

                    Variable {
                        name: name
                            .replace("`", "")
                            .trim()
                            .trim_end_matches(':')
                            .to_string(),
                        description: desc.trim().to_string(),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

        Self {
            template_variables,
            template_functions,
            global_variables,
            global_functions,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::manual::Manual;

    #[test]
    fn parse_manual() {
        let manual = Manual::default();
        assert!(!manual.template_variables.is_empty());
        assert!(!manual.template_functions.is_empty());
        assert!(!manual.global_variables.is_empty());
        assert!(!manual.global_functions.is_empty());
    }
}
