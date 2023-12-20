use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, put, web, App, Error, HttpResponse, HttpServer, Result};
use futures_util::stream::StreamExt;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

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
            let mut path_file = data.dir.clone();
            path_file.push(&*path);
            path_file.push(filename);
            path_file
        };

        //Creation des dossiers si besoin
        //Creation du fichier
        let mut file = web::block(|| {
            if let Some(path) = path.parent() {
                std::fs::create_dir_all(&path)?;
            }
            std::fs::File::create(path)
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
    }

    Ok(HttpResponse::Ok().body(""))
}

#[get("/pull/{path:.*}")]
async fn get_file(path: web::Path<PathBuf>, data: web::Data<AppState>) -> Result<NamedFile> {
    let path = {
        let mut x = data.dir.clone();
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
        let mut x = data.dir.clone();
        x.push(path.to_owned());
        x
    };

    let hashes:HashMap<PathBuf,String> = calculate_hash_recursive(&path)
        .into_iter()
        .map(|(k, d)| (k.strip_prefix(data.dir.clone()).unwrap().to_owned(), d))
        .collect();

    Ok(HttpResponse::Ok().json(hashes))
}

#[get("/rm/{path:.*}")]
async fn remove_file(path: web::Path<PathBuf>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {

    let path = {
        let mut p = data.dir.clone();
        p.push(path.to_owned());
        p
    };
    if std::fs::remove_file(&path).is_err() {
        return Err(actix_web::error::ErrorBadRequest("Cannot delete"));
    }

    Ok(HttpResponse::Ok().body(""))
}

fn calculate_hash(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn calculate_hash_recursive(path: &Path) -> HashMap<PathBuf, String> {
    let mut hashes: HashMap<PathBuf, String> = HashMap::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(hash) = calculate_hash(entry.path()) {
                hashes.insert(entry.path().to_owned(), hash.to_string());
            }
        } else if entry.path() != path {
            let h = calculate_hash_recursive(entry.path());
            for (key, value) in h.into_iter() {
                hashes.insert(key, value);
            }
        }
    }

    hashes
}

struct AppState {
    dir: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Serveur en ecoute sur le port 8080");
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                dir: PathBuf::from("./files"),
            }))
            .service(
                web::scope("/files")
                    .service(push)
                    .service(get_file)
                    .service(get_hash)
                    .service(remove_file)
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
