use std::error::Error;

#[derive(PartialEq, Debug)]
pub struct Args<'a> {
    pub url: &'a str,
    pub out_dir: &'a str,
}

impl<'a> Args<'a> {
    pub fn new(args: &'a Vec<String>) -> Result<Self, Box<dyn Error>> {
        if args.len() == 3 {
            return Ok(Args {
                url: args[1].as_str(),
                out_dir: args[2].as_str(),
            });
        }
        Err(Box::from("Usage: single URL OUT_DIR"))
    }
}

#[test]
fn args_success() {
    let args = vec![
        "file".to_string(),
        "http://example.com".to_string(),
        "./crawlings".to_string(),
    ];
    assert_eq!(
        Args::new(&args).unwrap(),
        Args {
            url: "http://example.com",
            out_dir: "./crawlings"
        }
    );
}

#[test]
fn args_failure() {
    let args = vec!["file".to_string()];
    assert!(Args::new(&args).is_err());
}
