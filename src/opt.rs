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
        short = "u",
        long = "user",
        help = "Server username and password, in <username:password>"
    )]
    pub basic: Option<String>,

    #[structopt(short, long, help = "Bearer token")]
    pub bearer: Option<String>,

    #[structopt(
        short = "-H",
        long = "header",
        multiple = true,
        number_of_values = 1,
        help = "Customized header, in <key:value>"
    )]
    pub headers: Option<Vec<String>>,

    #[structopt(short, long, help = "Post data file")]
    pub file: Option<String>,

    #[structopt(short = "k", long, help = "Allow insecure connections when using SSL")]
    pub insecure: bool,

    #[structopt(short = "j", long, help = "Cookie jar to send/save the cookies")]
    pub cookie_jar: Option<String>,

    #[structopt(
        short = "d",
        long,
        multiple = true,
        number_of_values = 1,
        help = "Post form data, in <key=value>"
    )]
    pub form: Option<Vec<String>>,

    #[structopt(
        short = "c",
        long = "cookie",
        multiple = true,
        number_of_values = 1,
        help = "Send individual cookie"
    )]
    pub cookies: Option<Vec<String>>,
}
