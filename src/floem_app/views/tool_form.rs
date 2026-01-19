//! Tool Parameter Form View
//!
//! A modal form for collecting tool parameters with validation and type-aware input fields.
//! Supports JSON Schema parsing for automatic form generation.

use floem::prelude::*;
use floem::views::{v_stack, h_stack, label, text_input, container, scroll, Decorators};
use floem::keyboard::{Key, NamedKey};
use floem::text::Weight;
use std::collections::HashMap;
use serde_json::Value as JsonValue;

use crate::floem_app::theme::Theme;

/// Parameter type for form fields
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl ParameterType {
    /// Parse parameter type from JSON Schema type string
    pub fn from_schema_type(type_str: &str) -> Self {
        match type_str {
            "string" => Self::String,
            "number" | "integer" => Self::Number,
            "boolean" => Self::Boolean,
            "array" => Self::Array,
            "object" => Self::Object,
            _ => Self::String, // Default to string
        }
    }
}

/// Form parameter definition
#[derive(Debug, Clone)]
pub struct FormParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
}

impl FormParameter {
    /// Create a new form parameter
    pub fn new(
        name: String,
        description: String,
        param_type: ParameterType,
        required: bool,
        default_value: Option<String>,
    ) -> Self {
        Self {
            name,
            description,
            param_type,
            required,
            default_value,
        }
    }

    /// Create from JSON Schema property
    pub fn from_schema_property(
        name: String,
        property: &JsonValue,
        required: bool,
    ) -> Self {
        let description = property
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let param_type = property
            .get("type")
            .and_then(|v| v.as_str())
            .map(ParameterType::from_schema_type)
            .unwrap_or(ParameterType::String);

        let default_value = property
            .get("default")
            .map(|v| v.to_string());

        Self::new(name, description, param_type, required, default_value)
    }
}

/// Tool form state
#[derive(Clone)]
pub struct ToolFormState {
    /// Whether the form is visible
    pub visible: RwSignal<bool>,
    /// Tool name
    pub tool_name: RwSignal<String>,
    /// Tool description
    pub tool_description: RwSignal<String>,
    /// Form parameters
    pub parameters: RwSignal<Vec<FormParameter>>,
    /// Current form values (parameter name -> value)
    pub values: RwSignal<HashMap<String, String>>,
    /// Validation errors (parameter name -> error message)
    pub errors: RwSignal<HashMap<String, String>>,
}

impl ToolFormState {
    /// Create a new tool form state
    pub fn new() -> Self {
        Self {
            visible: RwSignal::new(false),
            tool_name: RwSignal::new(String::new()),
            tool_description: RwSignal::new(String::new()),
            parameters: RwSignal::new(Vec::new()),
            values: RwSignal::new(HashMap::new()),
            errors: RwSignal::new(HashMap::new()),
        }
    }

    /// Show the form with tool information
    pub fn show(
        &self,
        tool_name: String,
        tool_description: String,
        parameters: Vec<FormParameter>,
    ) {
        self.tool_name.set(tool_name);
        self.tool_description.set(tool_description);
        self.parameters.set(parameters.clone());

        // Initialize values with defaults
        let mut values = HashMap::new();
        for param in parameters {
            if let Some(default) = param.default_value {
                values.insert(param.name.clone(), default);
            }
        }
        self.values.set(values);
        self.errors.set(HashMap::new());

        self.visible.set(true);
    }

    /// Create from JSON Schema
    pub fn from_json_schema(_tool_name: &str, schema: &JsonValue) -> Option<Vec<FormParameter>> {
        let properties = schema.get("properties")?.as_object()?;
        let required_list = schema
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut parameters = Vec::new();
        for (name, property) in properties {
            let is_required = required_list.contains(name);
            let param = FormParameter::from_schema_property(
                name.clone(),
                property,
                is_required,
            );
            parameters.push(param);
        }

        Some(parameters)
    }

    /// Hide the form and reset state
    pub fn hide(&self) {
        self.visible.set(false);
        self.tool_name.set(String::new());
        self.tool_description.set(String::new());
        self.parameters.set(Vec::new());
        self.values.set(HashMap::new());
        self.errors.set(HashMap::new());
    }

    /// Validate current form values
    pub fn validate(&self) -> bool {
        let parameters = self.parameters.get();
        let values = self.values.get();
        let mut errors = HashMap::new();

        for param in parameters {
            let value = values.get(&param.name);

            // Check required fields
            if param.required && (value.is_none() || value.unwrap().trim().is_empty()) {
                errors.insert(param.name.clone(), "This field is required".to_string());
                continue;
            }

            // Type validation
            if let Some(val) = value {
                if !val.trim().is_empty() {
                    match param.param_type {
                        ParameterType::Number => {
                            if val.parse::<f64>().is_err() {
                                errors.insert(param.name.clone(), "Must be a valid number".to_string());
                            }
                        }
                        ParameterType::Boolean => {
                            let lower = val.to_lowercase();
                            if lower != "true" && lower != "false" {
                                errors.insert(param.name.clone(), "Must be true or false".to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        self.errors.set(errors.clone());
        errors.is_empty()
    }

    /// Get all form values as a HashMap
    pub fn get_values(&self) -> HashMap<String, String> {
        self.values.get()
    }
}

impl Default for ToolFormState {
    fn default() -> Self {
        Self::new()
    }
}

/// Main tool form view
pub fn tool_form<F1, F2>(
    state: &ToolFormState,
    theme: RwSignal<Theme>,
    on_submit: F1,
    on_cancel: F2,
) -> impl IntoView
where
    F1: Fn(HashMap<String, String>) + Clone + 'static,
    F2: Fn() + Clone + 'static,
{
    let state_modal = state.clone();
    let state_close = state.clone();
    let state_submit = state.clone();
    let state_cancel = state.clone();
    let state_params = state.clone();

    let tool_name = state.tool_name;
    let tool_description = state.tool_description;

    // Clones for submit handler
    let on_submit_clone = on_submit.clone();

    container(
        v_stack((
            // Header
            h_stack((
                label(move || {
                    let name = tool_name.get();
                    format!("🔧 {}", name)
                })
                .style(move |s| {
                    let colors = theme.get().colors();
                    s.font_size(18.0)
                        .font_weight(Weight::BOLD)
                        .color(colors.text_primary)
                }),
            ))
            .style(move |s| {
                let colors = theme.get().colors();
                s.width_full()
                    .padding(16.0)
                    .border_bottom(1.0)
                    .border_color(colors.border)
            }),

            // Description (if exists)
            {
                let desc = tool_description.get();
                if !desc.is_empty() {
                    v_stack((
                        label(move || tool_description.get())
                            .style(move |s| {
                                let colors = theme.get().colors();
                                s.font_size(13.0)
                                    .color(colors.text_secondary)
                            }),
                    ))
                    .style(move |s| {
                        let colors = theme.get().colors();
                        s.width_full()
                            .padding(16.0)
                            .padding_top(8.0)
                            .border_bottom(1.0)
                            .border_color(colors.border)
                    })
                    .into_any()
                } else {
                    floem::views::empty()
                        .style(|s| s.display(floem::style::Display::None))
                        .into_any()
                }
            },

            // Form fields (scrollable)
            scroll(
                dyn_stack(
                    move || state_params.parameters.get(),
                    move |param| param.name.clone(),
                    move |param| parameter_field(param, &state_params, theme),
                )
                .style(|s| s.width_full().flex_col().gap(16.0))
            )
            .style(|s| {
                s.width_full()
                    .flex_grow(1.0)
                    .padding(16.0)
            }),

            // Footer with buttons
            h_stack((
                // Cancel button
                container(
                    label(|| "Cancel".to_string())
                        .style(move |s| {
                            let colors = theme.get().colors();
                            s.font_size(13.0)
                                .color(colors.text_primary)
                        })
                )
                .style(move |s| {
                    let colors = theme.get().colors();
                    s.padding(10.0)
                        .padding_horiz(24.0)
                        .border_radius(6.0)
                        .background(colors.bg_secondary)
                        .border(1.0)
                        .border_color(colors.border)
                        .cursor(floem::style::CursorStyle::Pointer)
                        .hover(move |s| {
                            s.background(colors.bg_tab_hover)
                                .border_color(colors.border.multiply_alpha(1.5))
                        })
                })
                .on_click_stop({
                    let on_cancel = on_cancel.clone();
                    let state = state_cancel.clone();
                    move |_| {
                        state.hide();
                        on_cancel();
                    }
                }),

                // Execute button
                container(
                    label(|| "Execute".to_string())
                        .style(move |s| {
                            let colors = theme.get().colors();
                            s.font_size(13.0)
                                .font_weight(Weight::SEMIBOLD)
                                .color(colors.bg_primary)
                        })
                )
                .style(move |s| {
                    let colors = theme.get().colors();
                    s.padding(10.0)
                        .padding_horiz(24.0)
                        .border_radius(6.0)
                        .background(colors.accent_blue)
                        .cursor(floem::style::CursorStyle::Pointer)
                        .hover(move |s| {
                            s.background(colors.accent_blue.multiply_alpha(1.2))
                        })
                })
                .on_click_stop({
                    let state = state_submit.clone();
                    let on_submit = on_submit_clone.clone();
                    move |_| {
                        if state.validate() {
                            let values = state.get_values();
                            state.hide();
                            on_submit(values);
                        }
                    }
                }),
            ))
            .style(move |s| {
                let colors = theme.get().colors();
                s.width_full()
                    .padding(16.0)
                    .justify_end()
                    .gap(12.0)
                    .border_top(1.0)
                    .border_color(colors.border)
            }),
        ))
        .style(move |s| {
            let colors = theme.get().colors();
            s.width(600.0)
                .max_height_pct(80.0)
                .background(colors.bg_primary)
                .border_radius(8.0)
                .border(1.0)
                .border_color(colors.border)
                .box_shadow_blur(20.0)
        })
    )
    .style(move |s| {
        s.width_full()
            .height_full()
            .items_center()
            .justify_center()
            .background(floem::peniko::Color::rgba8(0, 0, 0, 128))
    })
    .on_event_stop(floem::event::EventListener::KeyDown, move |event| {
        if let floem::event::Event::KeyDown(key_event) = event {
            match &key_event.key.logical_key {
                Key::Named(NamedKey::Escape) => {
                    state_close.hide();
                }
                _ => {}
            }
        }
    })
    .on_click_stop(move |_| {
        // Close when clicking outside the modal
        state_modal.hide();
    })
}

/// Create a form field for a parameter
fn parameter_field(
    param: FormParameter,
    state: &ToolFormState,
    theme: RwSignal<Theme>,
) -> impl IntoView {
    let param_name = param.name.clone();
    let state_error = state.clone();

    // Get or create signal for this parameter's value
    let value_signal = RwSignal::new(
        state.values.get().get(&param_name).cloned().unwrap_or_default()
    );

    // Sync changes back to state
    {
        let state = state.clone();
        let param_name = param_name.clone();
        floem::reactive::create_effect(move |_| {
            let val = value_signal.get();
            state.values.update(|values| {
                values.insert(param_name.clone(), val);
            });
        });
    }

    v_stack((
        // Label with required indicator
        label({
            let param_name = param.name.clone();
            let required = param.required;
            move || {
                if required {
                    format!("{} *", param_name)
                } else {
                    param_name.clone()
                }
            }
        })
        .style(move |s| {
            let colors = theme.get().colors();
            let required = param.required;
            s.font_size(13.0)
                .font_weight(Weight::SEMIBOLD)
                .color(if required { colors.text_primary } else { colors.text_secondary })
                .margin_bottom(4.0)
        }),

        // Description
        if !param.description.is_empty() {
            label({
                let desc = param.description.clone();
                move || desc.clone()
            })
            .style(move |s| {
                let colors = theme.get().colors();
                s.font_size(11.0)
                    .color(colors.text_muted)
                    .margin_bottom(6.0)
            })
            .into_any()
        } else {
            floem::views::empty()
                .style(|s| s.display(floem::style::Display::None))
                .into_any()
        },

        // Input field based on type
        match param.param_type {
            ParameterType::String => {
                string_input(value_signal, theme).into_any()
            }
            ParameterType::Number => {
                number_input(value_signal, theme).into_any()
            }
            ParameterType::Boolean => {
                boolean_input(value_signal, theme).into_any()
            }
            ParameterType::Array | ParameterType::Object => {
                // For complex types, use textarea-style input
                text_area_input(value_signal, theme).into_any()
            }
        },

        // Error message
        {
            let param_name_label = param.name.clone();
            let param_name_style = param.name.clone();
            label(move || {
                state_error.errors.get()
                    .get(&param_name_label)
                    .cloned()
                    .unwrap_or_default()
            })
            .style(move |s| {
                let colors = theme.get().colors();
                let has_error = state_error.errors.get().contains_key(&param_name_style);
                s.font_size(11.0)
                    .color(colors.accent_red)
                    .margin_top(4.0)
                    .display(if has_error {
                        floem::style::Display::Flex
                    } else {
                        floem::style::Display::None
                    })
            })
        },
    ))
    .style(|s| s.width_full().flex_col())
}

/// String input field
fn string_input(value: RwSignal<String>, theme: RwSignal<Theme>) -> impl IntoView {
    text_input(value)
        .style(move |s| {
            let colors = theme.get().colors();
            s.width_full()
                .padding(10.0)
                .border(1.0)
                .border_color(colors.border)
                .border_radius(6.0)
                .background(colors.bg_secondary)
                .color(colors.text_primary)
                .font_size(13.0)
                .focus(move |s| {
                    s.border_color(colors.accent_blue)
                        .border(2.0)
                })
        })
}

/// Number input field
fn number_input(value: RwSignal<String>, theme: RwSignal<Theme>) -> impl IntoView {
    text_input(value)
        .style(move |s| {
            let colors = theme.get().colors();
            s.width_full()
                .padding(10.0)
                .border(1.0)
                .border_color(colors.border)
                .border_radius(6.0)
                .background(colors.bg_secondary)
                .color(colors.text_primary)
                .font_size(13.0)
                .focus(move |s| {
                    s.border_color(colors.accent_blue)
                        .border(2.0)
                })
        })
}

/// Boolean input field (dropdown-style)
fn boolean_input(value: RwSignal<String>, theme: RwSignal<Theme>) -> impl IntoView {
    let value_for_true = value;
    let value_for_false = value;

    h_stack((
        // True button
        container(
            label(|| "True".to_string())
                .style(move |s| {
                    let colors = theme.get().colors();
                    let is_selected = value_for_true.get().to_lowercase() == "true";
                    s.font_size(13.0)
                        .color(if is_selected { colors.accent_blue } else { colors.text_secondary })
                })
        )
        .style(move |s| {
            let colors = theme.get().colors();
            let is_selected = value_for_true.get().to_lowercase() == "true";
            s.padding(8.0)
                .padding_horiz(20.0)
                .border_radius(6.0)
                .background(if is_selected { colors.accent_blue.multiply_alpha(0.2) } else { colors.bg_secondary })
                .border(1.0)
                .border_color(if is_selected { colors.accent_blue } else { colors.border })
                .cursor(floem::style::CursorStyle::Pointer)
                .hover(move |s| {
                    if !is_selected {
                        s.background(colors.bg_tab_hover)
                            .border_color(colors.border.multiply_alpha(1.5))
                    } else {
                        s
                    }
                })
        })
        .on_click_stop(move |_| {
            value_for_true.set("true".to_string());
        }),

        // False button
        container(
            label(|| "False".to_string())
                .style(move |s| {
                    let colors = theme.get().colors();
                    let is_selected = value_for_false.get().to_lowercase() == "false" || value_for_false.get().is_empty();
                    s.font_size(13.0)
                        .color(if is_selected { colors.accent_blue } else { colors.text_secondary })
                })
        )
        .style(move |s| {
            let colors = theme.get().colors();
            let is_selected = value_for_false.get().to_lowercase() == "false" || value_for_false.get().is_empty();
            s.padding(8.0)
                .padding_horiz(20.0)
                .border_radius(6.0)
                .background(if is_selected { colors.accent_blue.multiply_alpha(0.2) } else { colors.bg_secondary })
                .border(1.0)
                .border_color(if is_selected { colors.accent_blue } else { colors.border })
                .cursor(floem::style::CursorStyle::Pointer)
                .hover(move |s| {
                    if !is_selected {
                        s.background(colors.bg_tab_hover)
                            .border_color(colors.border.multiply_alpha(1.5))
                    } else {
                        s
                    }
                })
        })
        .on_click_stop(move |_| {
            value_for_false.set("false".to_string());
        }),
    ))
    .style(|s| s.gap(12.0))
}

/// Text area input for complex types (arrays, objects)
fn text_area_input(value: RwSignal<String>, theme: RwSignal<Theme>) -> impl IntoView {
    text_input(value)
        .style(move |s| {
            let colors = theme.get().colors();
            s.width_full()
                .padding(10.0)
                .border(1.0)
                .border_color(colors.border)
                .border_radius(6.0)
                .background(colors.bg_secondary)
                .color(colors.text_primary)
                .font_size(12.0)
                .font_family("monospace".to_string())
                .min_height(80.0)
                .focus(move |s| {
                    s.border_color(colors.accent_blue)
                        .border(2.0)
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_type_from_schema() {
        assert_eq!(ParameterType::from_schema_type("string"), ParameterType::String);
        assert_eq!(ParameterType::from_schema_type("number"), ParameterType::Number);
        assert_eq!(ParameterType::from_schema_type("integer"), ParameterType::Number);
        assert_eq!(ParameterType::from_schema_type("boolean"), ParameterType::Boolean);
        assert_eq!(ParameterType::from_schema_type("array"), ParameterType::Array);
        assert_eq!(ParameterType::from_schema_type("object"), ParameterType::Object);
        assert_eq!(ParameterType::from_schema_type("unknown"), ParameterType::String);
    }

    #[test]
    fn test_form_parameter_creation() {
        let param = FormParameter::new(
            "test".to_string(),
            "Test parameter".to_string(),
            ParameterType::String,
            true,
            Some("default".to_string()),
        );

        assert_eq!(param.name, "test");
        assert_eq!(param.description, "Test parameter");
        assert_eq!(param.param_type, ParameterType::String);
        assert!(param.required);
        assert_eq!(param.default_value, Some("default".to_string()));
    }

    #[test]
    fn test_tool_form_state_validation() {
        let state = ToolFormState::new();

        let params = vec![
            FormParameter::new(
                "command".to_string(),
                "Command to run".to_string(),
                ParameterType::String,
                true,
                None,
            ),
            FormParameter::new(
                "timeout".to_string(),
                "Timeout in seconds".to_string(),
                ParameterType::Number,
                false,
                Some("30".to_string()),
            ),
        ];

        state.parameters.set(params);

        // Should fail validation - required field is empty
        assert!(!state.validate());

        // Add value for required field
        state.values.update(|v| {
            v.insert("command".to_string(), "npm run build".to_string());
        });

        // Should pass validation now
        assert!(state.validate());

        // Add invalid number
        state.values.update(|v| {
            v.insert("timeout".to_string(), "not a number".to_string());
        });

        // Should fail validation - invalid number
        assert!(!state.validate());

        // Fix the number
        state.values.update(|v| {
            v.insert("timeout".to_string(), "60".to_string());
        });

        // Should pass validation
        assert!(state.validate());
    }

    #[test]
    fn test_json_schema_parsing() {
        let schema_json = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory"
                },
                "timeout": {
                    "type": "number",
                    "description": "Timeout in seconds",
                    "default": 30
                }
            },
            "required": ["command"]
        });

        let params = ToolFormState::from_json_schema("run_command", &schema_json)
            .expect("Should parse schema");

        assert_eq!(params.len(), 3);

        // Find command parameter
        let command_param = params.iter().find(|p| p.name == "command").unwrap();
        assert_eq!(command_param.param_type, ParameterType::String);
        assert!(command_param.required);

        // Find timeout parameter
        let timeout_param = params.iter().find(|p| p.name == "timeout").unwrap();
        assert_eq!(timeout_param.param_type, ParameterType::Number);
        assert!(!timeout_param.required);
    }
}
