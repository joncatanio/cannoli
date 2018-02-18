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

lazy_static! {
   static ref FILENAME_RE: Regex = Regex::new(r"(.+)\.(py|cannoli)$").unwrap();
}
