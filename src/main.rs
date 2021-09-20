extern crate clap;
extern crate anyhow;
extern crate reqwest;
extern crate tokio;
extern crate colored;
extern crate mime;

use std::{str::FromStr, collections::HashMap};
use clap::{AppSettings, Clap};
use anyhow::{anyhow, Result};
use colored::*;
use mime::Mime;
use reqwest::{header, Client, Response, Url};

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Ericoon <i@coooooc.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand
}

#[derive(Clap ,Debug)]
enum SubCommand {
    Get(Get),
    Post(Post)
}

#[derive(Clap, Debug)]
struct Get {
    /// HTTP 请求的 URL
    #[clap(parse(try_from_str = parse_url))]
    url: String
}

#[derive(Clap, Debug)]
struct Post {
    /// HTTP 请求的 URL
    #[clap(parse(try_from_str = parse_url))]
    url: String,

    /// HTTP 请求的 URL
    #[clap(parse(try_from_str = parse_kv_pair))]
    body: Vec<KVPair>
}

// 命令行中的 key=value 可以通过 parse_kv_pair 解析成 KVPair 结构
#[derive(Debug, PartialEq)]
struct KVPair {
    k: String,
    v: String
}

/// 当实现 FromStr trait 后， 可以用 str.parse() 方法将字符串解析成 KVPair
impl FromStr for KVPair {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 使用 = 进行 split，这会得到一个迭代器
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(
            Self {
                // 从迭代器中取第一个结果作为 key，迭代器返回 Some(T)/None
                // 将其转换成 Ok(T)/Err(E)，然后用 ? 处理错误
                k: (split.next().ok_or_else(err)?).to_string(),
                // 从迭代器中去第二个结果作为 value
                v: (split.next().ok_or_else(err)?).to_string()
            }            
        )
    } 
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    // 生成一个 HTTP 客户端
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED_BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    let client = Client::builder().default_headers(headers).build()?;
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?
    };

    println!("{:?}", opts);

    Ok(result)
}

// 处理 get 子命令
async fn get(client: Client, args: &Get) -> Result<()> {
    let response = client.get(&args.url).send().await?;
    Ok(print_response(response).await?)

}

// 处理 post 子命令
async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let response = client.post(&args.url).json(&body).send().await?;
    Ok(print_response(response).await?)
}

// 打印服务器版本号 & 状态码
fn print_status(response: &Response) {
    let status = format!("{:?} {}", response.version(), response.status());
    println!("{}\n", status)
}

// 打印服务器返回的 HTTP header
fn print_headers(response: &Response) {
    for (name, value) in response.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }

    print!("\n")
}

/// 打印服务器返回的 HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        // 对于 “application/json” pretty print
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())

        }
        // 其他 mime type，直接打印输出
        _ => println!("{}", body)
    }
}

fn get_content_type(response: &Response) -> Option<Mime> {
    response.headers().get(header::CONTENT_TYPE).map(|v| v.to_str().unwrap().parse().unwrap())
}

// 打印整个响应
async fn print_response(response: Response) -> Result<()> {
    print_status(&response);
    print_headers(&response);
    let mime = get_content_type(&response);
    let body = response.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn parse_url(s: &str) -> Result<String> {
    // 检查 URL 是否合法
    let _url: Url = s.parse()?;
    Ok(s.into())
}

// 为 KVPair 已经实现了 FromStr， 这里可以直接 s.parse() 得到 KVPair
fn parse_kv_pair(s: &str) -> Result<KVPair> {
    Ok(s.parse()?)
}