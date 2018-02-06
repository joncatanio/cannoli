use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use regex::Regex;

pub fn get_file_prefix(file: &str) -> String {
    if let Some(caps) = FILENAME_RE.captures(&file) {
        caps[1].to_string()
    } else {
        println!("unsupported filetype for file: {}", file);
        // TODO return CompilerError
        unimplemented!()
    }
}

/// Takes a string `name` and returns `name` appended with its hash value.
pub fn mangle_name(name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    let mut result = name.to_string();

    result.hash(&mut hasher);
    result.push_str("_");
    result.push_str(&hasher.finish().to_string());
    result
}

lazy_static! {
   static ref FILENAME_RE: Regex = Regex::new(r"(.+)\.(py|cannoli)$").unwrap();
}
