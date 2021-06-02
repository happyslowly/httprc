use super::opt::Method;
use super::opt::Opt;
use reqwest;
use reqwest::blocking::{Client, Request, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderName};
use serde_json;
use serde_json::Value;
use std::error::Error;
use std::fs::File;

pub fn process(opt: &Opt) {
    let client = make_client(opt);
    match opt.method {
        Method::Get => get(client.get(&opt.url), &client, opt),
        Method::Post => post_or_put(client.post(&opt.url), &client, opt),
        Method::Put => post_or_put(client.put(&opt.url), &client, opt),
        _ => {
            unimplemented!()
        }
    };
}

fn get(builder: RequestBuilder, client: &Client, opt: &Opt) {
    let builder = enrich_request(builder, opt);
    send(builder, client, opt);
}

fn post_or_put(builder: RequestBuilder, client: &Client, opt: &Opt) {
    let mut builder = enrich_request(builder, opt);
    if let Some(ref file) = opt.file {
        match File::open(file) {
            Ok(f) => builder = builder.body(f),
            Err(e) => {
                eprintln!("{}", e.to_string());
                std::process::exit(1);
            }
        }
    }
    send(builder, client, opt);
}

fn make_client(opt: &Opt) -> Client {
    let mut builder = Client::builder();
    if opt.insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }
    match builder.build() {
        Ok(client) => client,
        Err(e) => {
            handle_http_error(&e, opt.verbose);
            std::process::exit(1);
        }
    }
}

fn enrich_request(mut builder: RequestBuilder, opt: &Opt) -> RequestBuilder {
    if let Some(ref basic) = opt.basic {
        builder = basic_auth(builder, basic);
    }

    if let Method::Post | Method::Put = opt.method {
        builder = set_headers(builder, &vec!["content-type:application/json".to_string()])
    }

    if let Some(ref headers) = opt.headers {
        builder = set_headers(builder, headers)
    }
    builder
}

fn send(builder: RequestBuilder, client: &Client, opt: &Opt) {
    if let Ok(req) = builder.build() {
        if opt.verbose > 1 {
            dump_req(&req);
        }
        match client.execute(req) {
            Ok(r) => {
                if opt.verbose > 0 {
                    dump_version_and_status(&r);
                    dump_headers(r.headers(), false);
                }
                if let Ok(text) = r.text() {
                    dump_resp_body(&text, opt.verbose);
                }
            }
            Err(e) => handle_http_error(&e, opt.verbose),
        }
    }
}

fn handle_http_error(e: &reqwest::Error, verbose: u8) {
    if verbose > 0 {
        eprintln!("{}", e.to_string());
    } else {
        let mut err = e.source();
        while let Some(e) = err {
            if e.source().is_none() {
                eprintln!("{}", e);
                return;
            }
            err = e.source();
        }
        eprintln!("{}", e.to_string());
    }
}

fn dump_req(req: &Request) {
    println!("> {} {}", req.method(), req.url());
    dump_headers(req.headers(), true);
    println!();
}

fn dump_resp_body(text: &str, verbose: u8) {
    if verbose > 0 {
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

fn dump_headers(headers: &HeaderMap, is_req: bool) {
    let prefix = if is_req { ">" } else { "<" };
    for (k, v) in headers {
        println!("{} {}: {}", prefix, k, v.to_str().unwrap_or(""));
    }
}

fn dump_version_and_status(resp: &Response) {
    println!("< {:?} {:?}", resp.version(), resp.status());
}

fn basic_auth(builder: RequestBuilder, credential: &str) -> RequestBuilder {
    let v: Vec<&str> = credential.split(":").collect();
    if v.len() > 1 {
        let builder = builder.basic_auth(v[0], Some(v[1].to_owned()));
        return builder;
    }
    builder
}

fn set_headers(builder: RequestBuilder, headers: &Vec<String>) -> RequestBuilder {
    let mut hm = HeaderMap::new();
    for header in headers.iter() {
        let v = header.split(":").collect::<Vec<_>>();
        if v.len() > 1 {
            if let Ok(name) = HeaderName::from_bytes(v[0].as_bytes()) {
                hm.insert(name, v[1].parse().unwrap());
            }
        }
    }
    builder.headers(hm)
}
