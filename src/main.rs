use std::{env, io};

use actix_web::{App, get, HttpResponse, HttpServer, Responder, web};
use async_recursion::async_recursion;
use lazy_static::lazy_static;
use redis::AsyncCommands;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> io::Result<()> {
    let redis_addr = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let redis = redis::Client::open(redis_addr).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis.clone()))
            .service(check)
    }).bind(("0.0.0.0", 8080))?
        .run()
        .await
}

#[derive(Deserialize)]
struct VideoIdQuery {
    pub video_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RestrictionInfo {
    pub restricted: bool,
    pub regions: Option<Vec<String>>,
}

lazy_static!(
    static ref RE_VIDEO_ID: Regex = regex::Regex::new(r"^[a-zA-Z0-9_-]{11}$").unwrap();

    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:91.0) Gecko/20100101 Firefox/91.0")
        .proxy(reqwest::Proxy::all(env::var("PROXY").unwrap_or_else(|_| "socks5://127.0.0.1:9150".to_string())).unwrap())
        .build()
        .unwrap();
);

const ALL_REGIONS: [&str; 244] = [
    "AD",
    "AE",
    "AF",
    "AG",
    "AI",
    "AL",
    "AM",
    "AO",
    "AQ",
    "AR",
    "AS",
    "AT",
    "AU",
    "AW",
    "AX",
    "AZ",
    "BA",
    "BB",
    "BD",
    "BE",
    "BF",
    "BG",
    "BH",
    "BI",
    "BJ",
    "BL",
    "BM",
    "BN",
    "BO",
    "BR",
    "BS",
    "BT",
    "BV",
    "BW",
    "BY",
    "BZ",
    "CA",
    "CC",
    "CD",
    "CF",
    "CG",
    "CH",
    "CI",
    "CK",
    "CL",
    "CM",
    "CN",
    "CO",
    "CR",
    "CU",
    "CV",
    "CX",
    "CY",
    "CZ",
    "DE",
    "DJ",
    "DK",
    "DM",
    "DO",
    "DZ",
    "EC",
    "EE",
    "EG",
    "EH",
    "ER",
    "ES",
    "ET",
    "FI",
    "FJ",
    "FK",
    "FM",
    "FO",
    "FR",
    "GA",
    "GB",
    "GD",
    "GE",
    "GF",
    "GG",
    "GH",
    "GI",
    "GL",
    "GM",
    "GN",
    "GP",
    "GQ",
    "GR",
    "GS",
    "GT",
    "GU",
    "GW",
    "GY",
    "HK",
    "HM",
    "HN",
    "HR",
    "HT",
    "HU",
    "ID",
    "IE",
    "IL",
    "IM",
    "IN",
    "IO",
    "IQ",
    "IR",
    "IS",
    "IT",
    "JE",
    "JM",
    "JO",
    "JP",
    "KE",
    "KG",
    "KH",
    "KI",
    "KM",
    "KN",
    "KP",
    "KR",
    "KW",
    "KY",
    "KZ",
    "LA",
    "LB",
    "LC",
    "LI",
    "LK",
    "LR",
    "LS",
    "LT",
    "LU",
    "LV",
    "LY",
    "MA",
    "MC",
    "MD",
    "ME",
    "MG",
    "MH",
    "MK",
    "ML",
    "MM",
    "MN",
    "MO",
    "MP",
    "MQ",
    "MR",
    "MS",
    "MT",
    "MU",
    "MV",
    "MW",
    "MX",
    "MY",
    "MZ",
    "NA",
    "NC",
    "NE",
    "NF",
    "NG",
    "NI",
    "NL",
    "NO",
    "NP",
    "NR",
    "NU",
    "NZ",
    "OM",
    "PA",
    "PE",
    "PF",
    "PG",
    "PH",
    "PK",
    "PL",
    "PM",
    "PN",
    "PR",
    "PS",
    "PT",
    "PW",
    "PY",
    "QA",
    "RE",
    "RO",
    "RS",
    "RU",
    "RW",
    "SA",
    "SB",
    "SC",
    "SD",
    "SE",
    "SG",
    "SH",
    "SI",
    "SJ",
    "SK",
    "SL",
    "SM",
    "SN",
    "SO",
    "SR",
    "ST",
    "SV",
    "SY",
    "SZ",
    "TC",
    "TD",
    "TF",
    "TG",
    "TH",
    "TJ",
    "TK",
    "TL",
    "TM",
    "TN",
    "TO",
    "TR",
    "TT",
    "TV",
    "TW",
    "TZ",
    "UA",
    "UG",
    "UM",
    "US",
    "UY",
    "UZ",
    "VA",
    "VC",
    "VE",
    "VG",
    "VI",
    "VN",
    "VU",
    "WF",
    "WS",
    "YE",
    "YT",
    "ZA",
    "ZM",
    "ZW"
];

#[async_recursion]
async fn fetch_restrictions(video_id: &str) -> RestrictionInfo {
    let resp = CLIENT.get(format!("https://content-youtube.googleapis.com/youtube/v3/videos?id={}&part=contentDetails&key=AIzaSyAa8yy0GdcGPHdtD083HiGGx_S0vMPScDM", video_id))
        .header("Accept", "*/*")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Referer", "https://content-youtube.googleapis.com/")
        .header("X-ClientDetails", "appVersion=5.0%20(Windows)&platform=Win32&userAgent=Mozilla%2F5.0%20(Windows%20NT%2010.0%3B%20rv%3A91.0)%20Gecko%2F20100101%20Firefox%2F91.0")
        .header("X-Requested-With", "XMLHttpRequest")
        .header("X-JavaScript-User-Agent", "apix/3.0.0 google-api-javascript-client/1.1.0")
        .header("X-Origin", "https://explorer.apis.google.com")
        .header("X-Referer", "https://explorer.apis.google.com")
        .header("X-Goog-Encode-Response-If-Executable", "base64")
        .header("Connection", "close")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .header("TE", "trailers")
        .send().await;

    let resp = resp.unwrap();

    if !resp.status().is_success() {
        return fetch_restrictions(video_id).await;
    }

    let resp = resp.text().await.unwrap();

    let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();

    let blocked = resp["items"][0]["contentDetails"]["regionRestriction"]["blocked"].as_array();
    let allowed = resp["items"][0]["contentDetails"]["regionRestriction"]["allowed"].as_array();

    let region = {
        if let Some(allowed) = allowed {
            let regions = allowed.iter().map(|x| x.as_str().unwrap().to_string()).collect();
            RestrictionInfo {
                restricted: true,
                regions: Some(regions),
            }
        } else if let Some(blocked) = blocked {
            let blocked: Vec<String> = blocked.iter().map(|x| x.as_str().unwrap().to_string()).collect();
            let regions = ALL_REGIONS.iter().filter(|x| !blocked.contains(&x.to_string())).map(|x| x.to_string()).collect();
            RestrictionInfo {
                restricted: true,
                regions: Some(regions),
            }
        } else {
            RestrictionInfo {
                restricted: false,
                regions: None,
            }
        }
    };

    region
}

#[get("/api/region/check")]
async fn check(query: web::Query<VideoIdQuery>, redis: web::Data<redis::Client>) -> impl Responder {
    let video_id = &query.video_id;

    if !RE_VIDEO_ID.is_match(video_id) {
        return HttpResponse::BadRequest()
            .body("Invalid video ID");
    }

    let mut conn = redis.get_tokio_connection().await.unwrap();

    let restrictions = conn.get::<_, String>(video_id).await;

    let restrictions = {
        if restrictions.is_ok() {
            serde_json::from_str(restrictions.unwrap().as_str()).unwrap()
        } else {
            let restrictions = fetch_restrictions(video_id).await;
            conn.set_ex::<_, String, ()>(video_id, serde_json::to_string(&restrictions).unwrap(), 3600 * 24 * 30).await
                .expect("Failed to set restrictions in redis");
            restrictions
        }
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&restrictions).unwrap())
}
