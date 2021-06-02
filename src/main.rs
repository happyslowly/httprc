use httprc::client;
use httprc::opt::Opt;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    let result = client::process(&opt);
    if let Err(e) = result {
        let error = anyhow::Error::from(e);
        let mut msg = String::from("Error: ");
        if opt.verbose > 0 {
            msg.push_str(&format!("{:?}", error));
        } else {
            msg.push_str(&error.to_string());
        }
        eprintln!("{}", msg);
        std::process::exit(1);
    }
}
