use httprc::client;
use httprc::opt::Opt;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    client::process(&opt);
}
