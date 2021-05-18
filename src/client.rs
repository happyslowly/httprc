use super::opt::Method;
use super::opt::Opt;
use reqwest;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::HeaderMap;
use serde_json;
use serde_json::Value;

pub fn process(opt: &Opt) {
    let client = Client::new();
    match opt.method {
        Method::Get => get(&client, opt),
        _ => {
            unimplemented!()
        }
    };
}

fn get(client: &Client, opt: &Opt) {
    let mut req = client.get(&opt.url);

    if let Some(ref basic) = opt.basic {
        req = basic_auth(req, basic);
    }

    match req.send() {
        Ok(r) => {
            if opt.verbose > 0 {
                dump_version_and_status(&r);
                dump_headers(r.headers());
            }
            if let Ok(text) = r.text() {
                dump_body(&text, opt.verbose > 0);
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn dump_body(text: &str, verbose: bool) {
    if verbose {
        println!();
    }
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        if let Ok(s) = serde_json::to_string_pretty(&v) {
            println!("{}", s);
            return;
        }
    }
    println!("{}", text);
}

fn dump_headers(headers: &HeaderMap) {
    for (k, v) in headers {
        println!("{}: {:?}", k, v);
    }
}

fn dump_version_and_status(resp: &Response) {
    println!("{:?} {:?}", resp.version(), resp.status());
}

fn basic_auth(req: RequestBuilder, credential: &str) -> RequestBuilder {
    let v: Vec<&str> = credential.split(":").collect();
    if v.len() > 1 {
        let req = req.basic_auth(v[0], Some(v[1].to_owned()));
        return req;
    }
    req
}
