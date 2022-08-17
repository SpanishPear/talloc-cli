use clap::{AppSettings, Clap};
use futures::{StreamExt, stream};
use tokio::fs; 
use std::{sync::Mutex, collections::HashMap};
use reqwest::Client;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Shrey S. <shrey.somaiya@unsw.edu.au>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Apps(Apps),
}

/// A subcommand for fetching applications
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Apps {
    /// Print debug info
    #[clap(short)]
    debug: bool,
    
    #[clap(short, long)]
    force: bool,

    /// Fetch detailed data for n applicants else all applicants
    zids: Vec<String>,

}

#[derive(Debug)]
struct AppsArgs {
    jwt: String,
}

const TERM: u32 = 1;

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match opts.verbose {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        _ => println!("Don't be ridiculous"),
    }
    
    let client = Client::new();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::Apps(t) => {
            let args = AppsArgs {
                jwt: get_jwt().await,
            };

            if t.zids.is_empty() {
                dump_all_applications(&client, &args, &t.force).await;
            } else {
                dump_applications(&client, &args, &t.zids).await;
            }
        }
    }

    // more program logic goes here...
}

async fn dump_applications(client: &Client, args: &AppsArgs, zids: &[String]) {
   
    let data = Mutex::new(HashMap::<&String, String>::new());

    {
        let data = &data;
        stream::iter(zids)
            .for_each_concurrent(None, |zid| async move {
                let res = client.get(format!("https://talloc.web.cse.unsw.edu.au/api/v1/terms/{}/applications/{}", TERM, zid))
                            .header("x-jwt-auth", args.jwt.as_str())
                            .send()
                            .await
                            .expect("All application requst failed")
                            .text()
                            .await
                            .expect("could not unwrap req into text");
                data.lock().unwrap().insert(zid, res);
            }).await;
    }

    let map = data.lock().unwrap();
    for zid in zids {
        println!("{}", map[zid]); 
    }
}

async fn dump_all_applications(client: &Client, args: &AppsArgs, force: &bool) {
    // only works for term 1 lol
    
    let mut res = client.get(format!("https://talloc.web.cse.unsw.edu.au/api/v1/terms/{}/applications", TERM))
                .header("x-jwt-auth", args.jwt.as_str());

    if *force {
        res = res.header("x-apicache-bypass", "true");        
    }
   
    // todo
    let result = res
            .send()
            .await
            .expect("All application requst failed")
            .text()
            .await
            .expect("could not unwrap req into text");
     
    println!("{}", result);
}

async fn get_jwt() -> String {
    fs::read_to_string(".talloc.jwt")
        .await
        .expect("Missing .talloc.jwt token")
        .trim()
        .to_string()
}
