mod config;

use crate::config::Config;
use clap::{App, Arg};
use serde_json::Value;
use std::path::Path;
use warp::http::{Method, Uri};
use warp::{Filter, Rejection, Reply};

static QUERY: &str = r#"query AdvancedSearchQuery($cql: String!, $first: Int, $after: String, $includeArchivedSpaces: Boolean) {search(cql: $cql, first: $first, after: $after, includeArchivedSpaces: $includeArchivedSpaces) {nodes{url}}}"#;

pub async fn atlassian_search(
    config: &Config,
    ancestor: u64,
    title: &str,
) -> Result<Uri, Rejection> {
    let body = format!(
        r#"[{{"operationName":"AdvancedSearchQuery",
                                    "variables":{{"cql":"title ~ \"{}\" and ancestor = \"{}\" and type = \"page\"","first":5}},
                                    "query":"{}"}}]"#,
        title, ancestor, QUERY
    );
    let reply: Value = reqwest::Client::new()
        .post("https://tanker.atlassian.net/cgraphql?q=AdvancedSearchQuery")
        .basic_auth(&config.username, Some(&config.auth_token))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|_| warp::reject())?
        .json()
        .await
        .map_err(|_| warp::reject())?;
    let url = reply[0]["data"]["search"]["nodes"][0]["url"].as_str();
    let url = match url {
        Some(url) => url,
        None => return Err(warp::reject()),
    };

    let full_url = format!("https://tanker.atlassian.net/wiki{}", url);
    Ok(full_url.parse().unwrap())
}

pub async fn get_tep(config: &Config, idx: u32) -> Result<impl Reply, Rejection> {
    let tep_ancestor = 897909373;
    let uri = atlassian_search(config, tep_ancestor, &format!("{:>03}", idx)).await?;
    Ok(warp::redirect(uri))
}

pub async fn get_readme(config: &Config, title: String) -> Result<impl Reply, Rejection> {
    let readme_ancestor = 934412321;
    let uri = atlassian_search(config, readme_ancestor, &title).await?;
    Ok(warp::redirect(uri))
}

#[tokio::main]
async fn main() {
    let args = App::new("tanker-atlassiand")
        .about("Atlassian shortcut resolver daemon for Tanker")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .takes_value(true)
                .required(true)
                .about("Path to the config file"),
        )
        .get_matches();
    let config_path = args.value_of_os("config").unwrap();
    let config = Config::from_file(&Path::new(config_path));
    let config: &'static Config = Box::leak(Box::new(config));

    let health = warp::path!("health").map(|| "Ok");

    let tep = warp::path!("tep" / u32).and_then(move |idx| get_tep(config, idx));
    let readme = warp::path!("readme" / String).and_then(move |idx| get_readme(config, idx));

    let routes = warp::get()
        .and(health.or(tep).or(readme))
        .with(warp::cors().allow_any_origin().allow_method(Method::GET));

    warp::serve(routes).run(([0, 0, 0, 0], 80)).await;
}
