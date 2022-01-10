use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationError, ValidationFlags, Validator};
use naga::WithSpan;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("WGSL parse error: {0}")]
    WgslParseError(#[from] wgsl::ParseError),
    #[error("Validation error: {:?}", .0)]
    ValidationError(#[from] WithSpan<ValidationError>),
    #[error("Error reading file: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Recursive dependency on {} (stack: {:?})", .0.display(), .1)]
    RecursiveDependency(PathBuf, Vec<PathBuf>),
    #[error("Unknown directive: {0}")]
    UnknownDirective(String),
}

pub struct ShaderPreprocessor {
    base_directory: PathBuf,
    processed: HashMap<PathBuf, String>,
}
impl ShaderPreprocessor {
    pub fn new(base_directory: PathBuf) -> ShaderPreprocessor {
        ShaderPreprocessor {
            base_directory,
            processed: HashMap::new(),
        }
    }

    /*pub fn process_all(&mut self) -> Result<Vec<ShaderError>, std::io::Error> {
        let mut errors = vec![];
        for entry in std::fs::read_dir(&self.base_directory)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() { continue; }
            if let Some(extension) = path.extension() {
                if extension != "wgsl" { continue; }
                if let Err(e) = self.get(path, &vec![]) {
                    errors.push(e);
                }
            }
        }

        Ok(errors)
    }*/

    pub fn get<P: AsRef<Path>>(
        &mut self,
        rel_path: P,
        stack: &Vec<PathBuf>,
    ) -> Result<&str, ShaderError> {
        let path = self.base_directory.join(rel_path);
        if stack.contains(&path) {
            return Err(ShaderError::RecursiveDependency(path, stack.clone()));
        }

        match self.processed.get(&path) {
            None => {
                let data = self.preprocess(&path, stack)?;
                self.processed.insert(path.clone(), data);
            }
            _ => {}
        };
        Ok(self.processed.get(&path).unwrap())
    }

    fn preprocess(&mut self, path: &PathBuf, stack: &Vec<PathBuf>) -> Result<String, ShaderError> {
        if stack.contains(&path) {
            return Err(ShaderError::RecursiveDependency(
                path.clone(),
                stack.clone(),
            ));
        }

        let sub_stack = &{
            let mut ss = stack.clone();
            ss.push(path.clone());
            ss
        };

        let data = std::fs::read_to_string(path)?;
        let mut result = String::with_capacity(data.len());
        let mut validate = stack.is_empty();
        for line in data.lines() {
            if line.starts_with("//:") {
                let parts: Vec<_> = line["//:".len()..].split(' ').collect();
                if parts.len() > 0 && parts[0] == "no-validate" {
                    validate = false;
                } else if parts.len() > 1 && parts[0] == "include" {
                    let rel_path = parts[1..].join(" ");
                    let contents = self.get(rel_path, &sub_stack)?;
                    result.push_str(contents);
                    result.push('\n');
                } else {
                    return Err(ShaderError::UnknownDirective(line.to_string()));
                }
                continue;
            }

            result.push_str(line);
            result.push('\n');
        }

        if validate {
            let module = wgsl::parse_str(&result)?;
            let _ =
                Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module)?;
        }
        Ok(result)
    }
}

impl Default for ShaderPreprocessor {
    fn default() -> Self {
        let base_directory = match std::env::var_os("CARGO_MANIFEST_DIR") {
            Some(manifest_dir) => PathBuf::from(manifest_dir).join("shader"),
            None => PathBuf::from("./"),
        };
        ShaderPreprocessor::new(base_directory)
    }
}
