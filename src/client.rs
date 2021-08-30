use super::opt::Method;
use super::opt::Opt;
use anyhow::{Context, Result};
use reqwest;
use reqwest::blocking::{Client, Request, RequestBuilder, Response};
use reqwest::header::COOKIE;
use reqwest::header::SET_COOKIE;
use reqwest::header::{HeaderMap, HeaderName};
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

macro_rules! println {
    ($($arg:tt)*) => {
        let stdout = io::stdout();
        let handle = stdout.lock();
        let mut handle = io::BufWriter::new(handle);
        if let Err(e) = writeln!(&mut handle, $($arg)*) {
            if e.kind() != io::ErrorKind::BrokenPipe {
                eprintln!("{:?}", e);
            }
            return;
        }
    };
}

pub fn process(opt: &Opt) -> Result<()> {
    let client = make_client(opt).with_context(|| format!("Cannot create HTTP client"))?;
    match opt.method {
        Method::Get => get(client.get(&opt.url), &client, opt),
        Method::Post => post_or_put(client.post(&opt.url), &client, opt),
        Method::Put => post_or_put(client.put(&opt.url), &client, opt),
        _ => {
            unimplemented!()
        }
    }
}

fn get(builder: RequestBuilder, client: &Client, opt: &Opt) -> Result<()> {
    let builder = enrich_request(builder, opt)?;
    send(builder, client, opt)
}

fn post_or_put(builder: RequestBuilder, client: &Client, opt: &Opt) -> Result<()> {
    let mut builder = enrich_request(builder, opt)?;

    if let Some(ref file) = opt.file {
        let file = File::open(file).with_context(|| format!("Cannot open file `{}`", file))?;
        builder = builder.body(file);
    } else {
        if atty::isnt(atty::Stream::Stdin) {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .with_context(|| format!("Cannot read from stdin"))?;
            builder = builder.body(buffer);
        }
    }

    if let Some(ref form) = opt.form {
        let mut params = HashMap::new();
        for f in form {
            let kv = f.split("=").collect::<Vec<&str>>();
            if kv.len() > 1 {
                params.insert(kv[0], kv[1]);
            }
        }
        builder = builder.form(&params);
    }

    send(builder, client, opt)
}

fn make_client(opt: &Opt) -> Result<Client, reqwest::Error> {
    let mut builder = Client::builder();
    if opt.insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }
    builder.build()
}

fn enrich_request(mut builder: RequestBuilder, opt: &Opt) -> Result<RequestBuilder> {
    if let Some(ref basic) = opt.basic {
        builder = basic_auth(builder, basic);
    }

    if let Some(ref token) = opt.bearer {
        builder = builder.bearer_auth(token);
    }

    if let Method::Post | Method::Put = opt.method {
        builder = set_headers(builder, &vec!["content-type:application/json".to_string()])
    }

    if let Some(ref headers) = opt.headers {
        builder = set_headers(builder, headers)
    }

    let mut cookie_header = String::new();
    cookie_header.push_str(COOKIE.as_str());
    cookie_header.push(':');
    if let Some(ref cookies) = opt.cookies {
        for c in cookies {
            cookie_header.push_str(c);
            cookie_header.push(';');
        }
    }
    if let Some(ref cookie_jar) = opt.cookie_jar {
        match fs::read_to_string(cookie_jar) {
            Ok(jar) => cookie_header.push_str(&jar),
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(anyhow::Error::new(e)),
            _ => (),
        };
    }
    builder = set_headers(builder, &vec![cookie_header]);

    Ok(builder)
}

fn send(builder: RequestBuilder, client: &Client, opt: &Opt) -> Result<()> {
    let req = builder
        .build()
        .with_context(|| format!("Failed to create request"))?;

    if opt.verbose > 1 {
        dump_req(&req);
    }

    let resp = client
        .execute(req)
        .with_context(|| format!("Failed to send request"))?;

    if opt.verbose > 0 {
        dump_version_and_status(&resp);
        dump_headers(resp.headers(), false);
    }

    if let Some(cookie_jar) = &opt.cookie_jar {
        save_cookie(resp.headers(), cookie_jar)
            .with_context(|| format!("Failed to save cookies"))?;
    }

    let text = resp
        .text()
        .with_context(|| format!("Failed to extract response body"))?;
    dump_resp_body(&text, opt.verbose);
    Ok(())
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

fn save_cookie(headers: &HeaderMap, cookie_jar: &String) -> Result<()> {
    if let Some(cookies) = headers.get(SET_COOKIE) {
        fs::write(cookie_jar, cookies)
            .with_context(|| format!("Cannot write to cookie jar, `{}`", cookie_jar))?
    }
    Ok(())
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
