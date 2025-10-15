use crate::errors::Result;
use rhai::{Dynamic, Engine};
use std::collections::HashMap;

/// Configuration for parametric design
#[derive(Clone, Debug)]
pub struct ParametricConfig {
    pub variables: HashMap<String, f64>,
    pub steps: usize,
    pub script_template: String,
}

impl Default for ParametricConfig {
    fn default() -> Self {
        let mut variables = HashMap::new();
        variables.insert("width".to_string(), 100.0);
        variables.insert("height".to_string(), 100.0);
        variables.insert("radius".to_string(), 50.0);
        variables.insert("angle".to_string(), 0.0);

        Self {
            variables,
            steps: 100,
            script_template: r#"
// Parametric shape script
// Available variables: {t} (0-1), {width}, {height}, {radius}, {angle}
// Set x and y coordinates for each point

let x = {width} * cos({t} * 2.0 * PI);
let y = {height} * sin({t} * 2.0 * PI);
"#
            .to_string(),
        }
    }
}

/// Advanced parametric design system
pub struct ParametricDesigner;

impl ParametricDesigner {
    /// Create a new Rhai engine with custom functions
    pub fn create_engine() -> Engine {
        let mut engine = Engine::new();

        // Register custom functions
        engine.register_fn("lerp", |a: f64, b: f64, t: f64| a + (b - a) * t);
        engine.register_fn("clamp", |value: f64, min: f64, max: f64| {
            value.max(min).min(max)
        });
        engine.register_fn("smoothstep", |edge0: f64, edge1: f64, x: f64| {
            let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        });

        engine
    }

    /// Validate a parametric script
    pub fn validate_script(script: &str) -> Result<()> {
        let engine = Self::create_engine();

        match engine.compile(script) {
            Ok(_) => Ok(()),
            Err(e) => Err(crate::errors::GcodeKitError::Script(format!(
                "Script compilation error: {}",
                e
            ))),
        }
    }

    /// Evaluate a parametric script and generate points
    pub fn evaluate_script(
        script: &str,
        config: &ParametricConfig,
        bounds: (f32, f32, f32, f32),
    ) -> Result<Vec<(f32, f32)>> {
        let engine = Self::create_engine();

        let mut points = Vec::new();

        // Calculate dimensions
        let width = bounds.2 as f64 - bounds.0 as f64;
        let height = bounds.3 as f64 - bounds.1 as f64;
        let center_x = (bounds.0 + bounds.2) as f64 / 2.0;
        let center_y = (bounds.1 + bounds.3) as f64 / 2.0;

        // Scale factor (assuming shapes are designed for -1 to 1 range)
        let scale = (width.min(height) / 2.0) as f32;

        // Evaluate for each step
        for i in 0..config.steps {
            let t = i as f64 / (config.steps - 1) as f64;

            // Create script with variables replaced
            let mut script_for_step = script.to_string() + "\nreturn #{\"x\": x, \"y\": y};";

            // Replace built-in variables
            script_for_step = script_for_step.replace("{t}", &t.to_string());
            script_for_step = script_for_step.replace("{width}", &width.to_string());
            script_for_step = script_for_step.replace("{height}", &height.to_string());
            script_for_step = script_for_step.replace("{center_x}", &center_x.to_string());
            script_for_step = script_for_step.replace("{center_y}", &center_y.to_string());
            script_for_step = script_for_step.replace("{PI}", &std::f64::consts::PI.to_string());
            script_for_step = script_for_step.replace("{E}", &std::f64::consts::E.to_string());

            // Replace user-defined variables
            for (name, value) in &config.variables {
                script_for_step =
                    script_for_step.replace(&format!("{{{}}}", name), &value.to_string());
            }

            // Evaluate the script
            let result: Dynamic = engine.eval(&script_for_step).map_err(|_| {
                crate::errors::GcodeKitError::Script("Script evaluation error".to_string())
            })?;

            // Extract x and y from the returned map
            let map = result
                .try_cast::<std::collections::BTreeMap<rhai::ImmutableString, Dynamic>>()
                .ok_or_else(|| {
                    crate::errors::GcodeKitError::Script(
                        "Script must return a map with x and y".to_string(),
                    )
                })?;
            let key_x = rhai::ImmutableString::from("x");
            let key_y = rhai::ImmutableString::from("y");
            let x = map
                .get(&key_x)
                .and_then(|d| d.as_float().ok())
                .unwrap_or(0.0);
            let y = map
                .get(&key_y)
                .and_then(|d| d.as_float().ok())
                .unwrap_or(0.0);

            // Scale and center the points
            let scaled_x = center_x + x * scale as f64;
            let scaled_y = center_y + y * scale as f64;
            points.push((scaled_x as f32, scaled_y as f32));
        }

        Ok(points)
    }

    /// Generate common parametric shapes
    pub fn generate_shape(shape_type: &str, config: &ParametricConfig) -> String {
        match shape_type {
            "circle" => {
                let mut script = r#"
// Circle
let x = {radius} * cos({t} * 2.0 * PI);
let y = {radius} * sin({t} * 2.0 * PI);
"#
                .to_string();
                script = script.replace(
                    "{radius}",
                    &config.variables.get("radius").unwrap_or(&50.0).to_string(),
                );
                script
            }
            "ellipse" => {
                let mut script = r#"
// Ellipse
let a = {width} / 2.0;
let b = {height} / 2.0;
let x = a * cos({t} * 2.0 * PI);
let y = b * sin({t} * 2.0 * PI);
"#
                .to_string();
                script = script.replace(
                    "{width}",
                    &config.variables.get("width").unwrap_or(&100.0).to_string(),
                );
                script = script.replace(
                    "{height}",
                    &config.variables.get("height").unwrap_or(&100.0).to_string(),
                );
                script
            }
            "spiral" => {
                let mut script = r#"
// Spiral
let turns = {turns};
let max_radius = {radius};
let radius = max_radius * {t};
let angle = {t} * turns * 2.0 * PI;
let x = radius * cos(angle);
let y = radius * sin(angle);
"#
                .to_string();
                script = script.replace(
                    "{turns}",
                    &config.variables.get("turns").unwrap_or(&3.0).to_string(),
                );
                script = script.replace(
                    "{radius}",
                    &config.variables.get("radius").unwrap_or(&50.0).to_string(),
                );
                script
            }
            "star" => {
                let mut script = r#"
// Star
let outer_radius = {outer_radius};
let inner_radius = {inner_radius};
let points = {points};
let angle = {t} * 2.0 * PI;
let radius = if ({t} * points * 2.0) as i32 % 2 == 0 {outer_radius} else {inner_radius};
let x = radius * cos(angle);
let y = radius * sin(angle);
"#
                .to_string();
                script = script.replace(
                    "{outer_radius}",
                    &config
                        .variables
                        .get("outer_radius")
                        .unwrap_or(&50.0)
                        .to_string(),
                );
                script = script.replace(
                    "{inner_radius}",
                    &config
                        .variables
                        .get("inner_radius")
                        .unwrap_or(&25.0)
                        .to_string(),
                );
                script = script.replace(
                    "{points}",
                    &config.variables.get("points").unwrap_or(&5.0).to_string(),
                );
                script
            }
            "heart" => r#"
// Heart shape
let angle = {t} * 2.0 * PI;
let x = 16.0 * sin(angle).powf(3.0);
let y = 13.0 * cos(angle) - 5.0 * cos(2.0 * angle) - 2.0 * cos(3.0 * angle) - cos(4.0 * angle);
"#
            .to_string(),
            "wave" => {
                let mut script = r#"
// Sine wave
let amplitude = {amplitude};
let frequency = {frequency};
let x = {t} * {width};
let y = amplitude * sin({t} * frequency * 2.0 * PI);
"#
                .to_string();
                script = script.replace(
                    "{amplitude}",
                    &config
                        .variables
                        .get("amplitude")
                        .unwrap_or(&20.0)
                        .to_string(),
                );
                script = script.replace(
                    "{frequency}",
                    &config
                        .variables
                        .get("frequency")
                        .unwrap_or(&2.0)
                        .to_string(),
                );
                script = script.replace(
                    "{width}",
                    &config.variables.get("width").unwrap_or(&100.0).to_string(),
                );
                script
            }
            _ => config.script_template.clone(),
        }
    }

    /// Extract variables from script comments
    pub fn extract_variables(script: &str) -> HashMap<String, f64> {
        let mut variables = HashMap::new();

        for line in script.lines() {
            let line = line.trim();
            if (line.starts_with("// var ") || line.starts_with("// variable "))
                && let Some(eq_pos) = line.find('=') {
                    let var_part = &line[7..eq_pos].trim(); // Skip "// var "
                    let value_part = &line[eq_pos + 1..].trim();

                    if let (Some(var_name), Ok(value)) = (
                        var_part.split_whitespace().next(),
                        value_part.parse::<f64>(),
                    ) {
                        variables.insert(var_name.to_string(), value);
                    }
                }
        }

        variables
    }

    /// Optimize script performance by pre-compiling
    pub fn optimize_script(script: &str) -> Result<String> {
        // Basic optimization: remove comments and unnecessary whitespace
        let mut optimized = String::new();
        let mut in_multiline_comment = false;

        for line in script.lines() {
            let line = line.trim();

            // Handle multiline comments (basic)
            if line.contains("/*") {
                in_multiline_comment = true;
            }
            if in_multiline_comment {
                if line.contains("*/") {
                    in_multiline_comment = false;
                }
                continue;
            }

            // Skip single-line comments
            if line.starts_with("//") {
                continue;
            }

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            optimized.push_str(line);
            optimized.push('\n');
        }

        Ok(optimized)
    }
}
