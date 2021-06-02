use clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum Method {
        Get,
        Post,
        Put,
        Delete
    }
}

fn parse_url(s: &str) -> String {
    if !s.starts_with("http") {
        return format!("http://{}", s);
    }
    s.to_owned()
}

#[derive(StructOpt, Debug)]
#[structopt(name = "hrc", about = "An HTTP client in Rust.")]
pub struct Opt {
    #[structopt(
        short,
        long,
        parse(from_occurrences),
        help = "Display more information"
    )]
    pub verbose: u8,

    #[structopt(name = "URL", parse(from_str = parse_url))]
    pub url: String,

    #[structopt(short, long,
        possible_values = &Method::variants(),
        case_insensitive = true,
        default_value = "Get",
        help = "HTTP method")]
    pub method: Method,

    #[structopt(
        short,
        long,
        help = "Server username and password, in <username:password>"
    )]
    pub basic: Option<String>,

    #[structopt(
        short = "-H",
        long,
        multiple = true,
        help = "Customized header, in <key:value>"
    )]
    pub headers: Option<Vec<String>>,

    #[structopt(short, long, help = "POST data file")]
    pub file: Option<String>,

    #[structopt(short = "k", long, help = "Allow insecure connections when using SSL")]
    pub insecure: bool,
}
