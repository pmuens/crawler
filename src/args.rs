use std::error::Error;

#[derive(PartialEq, Debug)]
pub struct Args<'a> {
    pub url: &'a str,
    pub out_dir: &'a str,
    pub num_threads: usize,
}

impl<'a> Args<'a> {
    pub fn new(args: &'a [String]) -> Result<Self, Box<dyn Error>> {
        if args.len() == 4 {
            let args = Args {
                url: args[1].as_str(),
                out_dir: args[2].as_str(),
                num_threads: args[3].parse::<usize>().unwrap(),
            };
            return Ok(args);
        }
        Err(Box::from("Usage: crawler URL OUT_DIR NUM_THREADS"))
    }
}

#[cfg(test)]
mod tests {
    use crate::args::Args;

    #[test]
    fn args_success() {
        let args = vec![
            "file".to_string(),
            "http://example.com".to_string(),
            "./crawlings".to_string(),
            "6".to_string(),
        ];
        assert_eq!(
            Args::new(&args).unwrap(),
            Args {
                url: "http://example.com",
                out_dir: "./crawlings",
                num_threads: 6,
            }
        );
    }

    #[test]
    fn args_failure_missing_arguments() {
        let args = vec!["file".to_string()];
        assert!(Args::new(&args).unwrap_err().to_string().contains("Usage:"));
    }
}
