use clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum Method {
        Get,
        Post,
        Put,
    }
}

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(name = "URL")]
    pub url: String,

    #[structopt(short, long,
        possible_values = &Method::variants(),
        case_insensitive = true,
        default_value = "Get")]
    pub method: Method,

    #[structopt(short, long)]
    pub basic: Option<String>,
}
