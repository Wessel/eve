use serde_yaml::{Value, from_str};
use std::collections::HashMap;

pub fn replace_key_value(input: String, yaml_string: String) -> String {
  // Read the YAML file into a string
  // let yaml_string = std::fs::read_to_string("subsitutions.yml")
  //     .expect("Failed to read YAML file");

  // Parse the YAML string into a serde_yaml::Value object
  let yaml_value: Value = from_str(&yaml_string).expect("Failed to parse YAML string");

  // Create a map of keys and values from the YAML object
  let mut replacements = std::collections::HashMap::new();
  for (key, value) in yaml_value.as_mapping().unwrap().iter() {
      replacements.insert(key.as_str().unwrap(), value.as_str().unwrap());
  }

  // Replace all matching words in the input string with their corresponding values
  let input_string = input;
  let mut output_string = input_string.to_owned();
  for (key, value) in replacements.iter() {
      output_string = output_string.replace(key, value);
  }

  output_string
}

pub fn parse_cli_args(input: String) -> (HashMap<String, String>, String) {
  let mut args = HashMap::new();
  let mut replaced_string = input.to_owned();

  for arg in input.split_whitespace() {
      if arg.starts_with("--") {
        replaced_string = replaced_string.replace(arg, "");
          let parts: Vec<&str> = arg.splitn(2, '=').collect();
          if parts.len() == 2 {
              let key = parts[0][2..].to_string();
              let value = parts[1].to_string();
              args.insert(key, value);
          }
      }
  }

  (args, replaced_string)
}

pub fn ellipsis(input: String, length: usize) -> String {
  if input.len() < length {
    return input;
  }

  let truncated = &input[..(length - 3)];
  format!("{truncated}...")
}