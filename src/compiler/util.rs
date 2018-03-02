use regex::Regex;

use super::errors::CompilerError;

/// Returns the root directory of the given file and the file name sans ext
pub fn get_file_prefix(file: &str) -> Result<(String, String), CompilerError> {
    if let Some(caps) = FILENAME_RE.captures(&file) {
        match (caps.at(1), caps.at(2)) {
            (Some(src_root), Some(module)) => {
                Ok((src_root.to_string(), module.to_string()))
            },
            (None, Some(module)) => {
                Ok(("./".to_string(), module.to_string()))
            },
            (Some(_), None) | (None, None) => {
                return Err(CompilerError::IOError(format!("'{}' not found",
                    file)))
            }
        }
    } else {
        return Err(CompilerError::IOError(format!("unsupported filetype for \
            file: {}", file)))
    }
}

lazy_static! {
   static ref FILENAME_RE: Regex = Regex::new(r"(.*/)?(.+)\.py$").unwrap();
}
