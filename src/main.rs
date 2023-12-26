use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, put, web, App, Error, HttpResponse, HttpServer, Result};
use clap::Parser;
use futures_util::stream::StreamExt;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sauvegarde::{
    domain::{self, delete_file_hash},
    driven,
};
use std::{collections::HashMap, io::Write, path::PathBuf};

#[put("/push/{path:.*}")]
async fn push(
    path: web::Path<PathBuf>,
    data: web::Data<AppState>,
    mut multipart: Multipart,
) -> Result<HttpResponse, Error> {
    while let Some(item) = multipart.next().await {
        let mut field = item?;

        //Recuperation du nom
        let filename = field
            .content_disposition()
            .get_filename()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Nom de fichier manquant"))?;

        //On ajoute le nom du fichier a la fin
        let path = {
            let mut path_file = data.config.file_path.clone();
            path_file.push(&*path);
            path_file.push(filename);
            path_file
        };
        //Creation des dossiers si besoin
        //Creation du fichier
        let mut file = web::block({
            let path = path.clone();
            move || {
                if let Some(path) = path.parent() {
                    std::fs::create_dir_all(path)?;
                }
                std::fs::File::create(path)
            }
        })
        .await
        .unwrap()?;

        //Ecriture
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            file = web::block(move || file.write_all(&chunk).map(|_| file))
                .await
                .unwrap()?;
        }

        if domain::create_file_hash::create_file_hash(&path, &data.stockage).is_err() {
            return Err(actix_web::error::ErrorBadGateway("erreur du serveur"));
        }
    }

    Ok(HttpResponse::Ok().body("created"))
}

#[get("/pull/{path:.*}")]
async fn get_file(path: web::Path<PathBuf>, data: web::Data<AppState>) -> Result<NamedFile> {
    let path = {
        let mut x = data.config.file_path.clone();
        x.push(path.to_owned());
        x
    };

    Ok(NamedFile::open_async(path).await?)
}

#[get("/hash/{path:.*}")]
async fn get_hash(
    path: web::Path<PathBuf>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let path = {
        let mut x = data.config.file_path.clone();
        x.push(path.to_owned());
        x
    };

    let hashes: HashMap<PathBuf, String> =
        domain::read_all_file_hash::read_all_file_hash(&path, &data.stockage)
            .unwrap()
            .into_iter()
            .map(|e| e.into())
            .map(|(k, d)| {
                (
                    k.value()
                        .strip_prefix(data.config.file_path.clone())
                        .unwrap()
                        .to_owned(),
                    d.value().to_string(),
                )
            })
            .collect();
    Ok(HttpResponse::Ok().json(hashes))
}

#[get("/rm/{path:.*}")]
async fn remove_file(
    path: web::Path<PathBuf>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let path = {
        let mut p = data.config.file_path.clone();
        p.push(path.to_owned());
        p
    };
    if std::fs::remove_file(&path).is_err() {
        return Err(actix_web::error::ErrorBadRequest("Cannot delete"));
    }

    if delete_file_hash::delete_file_hash(&path, &data.stockage).is_err() {
        return Err(actix_web::error::ErrorBadGateway(
            "Erreur de suppression sur la base de donnée",
        ));
    }

    Ok(HttpResponse::Ok().body(""))
}

struct AppState {
    stockage: driven::stockage_sqlite::SqliteStockage,
    config: Config,
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Config {
    #[arg(short, long, env, default_value = "./files")]
    file_path: PathBuf,
    #[arg(short, long, env, default_value = "./cert.pem")]
    cert_path: PathBuf,
    #[arg(short, long, env, default_value = "./key.pem")]
    key_path: PathBuf,
    #[arg(short, long, default_value = "8080")]
    port: u16,
    #[arg(short, long, default_value = "8081")]
    tls_port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::parse();

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(&config.key_path, SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file(&config.cert_path)
        .unwrap();

    println!(
        "Serveur en ecoute sur le port {} pour http et {} pour https",
        config.port, config.tls_port
    );

    HttpServer::new({
        || {
            App::new()
                .app_data(web::Data::new(AppState {
                    config: Config::parse(),
                    stockage: driven::stockage_sqlite::SqliteStockage::new("database.db"),
                }))
                .service(
                    web::scope("/files")
                        .service(push)
                        .service(get_file)
                        .service(get_hash)
                        .service(remove_file),
                )
        }
    })
    .bind(("0.0.0.0", config.port))?
    .bind_openssl(("0.0.0.0", config.tls_port), builder)?
    .run()
    .await
}
