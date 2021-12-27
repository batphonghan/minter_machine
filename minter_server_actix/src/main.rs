use std::{io::Write, str::FromStr};

use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use anchor_client::Client;
use futures_util::TryStreamExt as _;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair, read_keypair_file, Keypair},
    signer::Signer,
};
use uuid::Uuid;

const TOKEN_PROGRAM_PUBKEY: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

mod mint_token;
async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field
            .content_disposition()
            .ok_or_else(|| HttpResponse::BadRequest().finish())?;

        let filename = content_disposition.get_filename().map_or_else(
            || Uuid::new_v4().to_string(),
            |f| sanitize_filename::sanitize(f),
        );
        let filepath = format!("./tmp/{}", filename);

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await?;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            println!(">> Size if chunk is {} ", chunk.len());
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await?;
        }

        let filepath = format!("./tmp/{}", filename);
        println!(">> DONE {}", &filepath);
    }

    Ok(HttpResponse::Ok().into())
}

fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // mint_nft().await.unwrap_or_else(|e| panic!("{:?}", e));
    // return Ok(());

    HttpServer::new(|| {
        App::new().wrap(middleware::Logger::default()).service(
            web::resource("/")
                .route(web::get().to(index))
                .route(web::post().to(save_file)),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

pub async fn mint_nft() -> anyhow::Result<Pubkey> {
    let matches = clap::App::new("minter machine server")
        .version("1.0")
        .author("vanhungdeveloper@gmail.com")
        .about("Handler request of a image to mint to nft")
        .arg(
            clap::Arg::with_name("program_id")
                .short("p")
                .long("program_id")
                .default_value(TOKEN_PROGRAM_PUBKEY),
        )
        .arg(
            clap::Arg::with_name("wallet")
                .short("w")
                .long("wallet")
                .default_value("/Users/batphonghan/.config/solana/id.json"),
        )
        .arg(
            clap::Arg::with_name("uri")
                .short("u")
                .long("uri")
                .default_value(""),
        )
        .arg(clap::Arg::with_name("name").long("name").default_value(""))
        .arg(
            clap::Arg::with_name("symbol")
                .long("symbol")
                .default_value(""),
        )
        .arg(
            clap::Arg::with_name("cluster")
                .short("c")
                .long("cluster")
                .default_value("devnet"),
        )
        .arg(
            clap::Arg::with_name("destination")
                .short("d")
                .long("destination-wallet")
                .default_value(""),
        )
        .get_matches();

    let wallet = matches.value_of("wallet").unwrap();
    println!("Value for wallet: {}", wallet);

    let destination = matches.value_of("destination").unwrap();
    println!("Value for destination: {}", destination);

    let cluster_url = matches.value_of("cluster").unwrap();
    println!("Value for cluster: {}", &cluster_url);

    let program_id = matches.value_of("program_id").unwrap();
    println!("Value for program ID: {}", program_id);

    let program_id = Pubkey::from_str(program_id).unwrap();

    std::env::set_var("RUST_LOG", "info");
    std::fs::create_dir_all("./tmp")?;

    let payer = read_keypair_file(wallet).expect("Requires a keypair file");
    let cluster = anchor_client::Cluster::from_str(cluster_url).unwrap();

    let client = Client::new_with_options(cluster, payer, CommitmentConfig::processed());
    let program_client = client.program(program_id);

    let minter = read_keypair_file(wallet).expect("Requires a keypair file");

    let dest_pubkey = Pubkey::from_str(destination).unwrap();
    let nft_pubkey =
        mint_token::init_and_mint_to_dest(&minter, &dest_pubkey, matches, program_client.rpc())?;

    Ok(nft_pubkey)
}
