#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mongodb::{Client, options::ClientOptions, Collection, Database};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use chrono::Utc;
use std::sync::Mutex;
use std::fs::{OpenOptions, File};
use std::io::Write;
use tauri::State;
use tauri::Manager;
use futures::stream::StreamExt;
use tokio::sync::Mutex as TokioMutex;

// Application constants
const DB_NAME: &str = "rez_launcher";
const MONGO_URI: &str = "mongodb://localhost:27017";

// MongoDB connection struct
struct MongoConnection(TokioMutex<Option<Client>>);

// App state for logging
struct LogState(Mutex<File>);

// Data structures
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackageCollection {
    version: String,
    packages: Vec<String>,
    herit: String,
    tools: Vec<String>,
    created_at: String,
    created_by: String,
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Stage {
    name: String,
    uri: String,
    from_version: String,
    rxt_path: String,
    tools: Vec<String>,
    created_at: String,
    created_by: String,
    active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackageCollectionResult {
    success: bool,
    message: Option<String>,
    collections: Option<Vec<PackageCollection>>,
}

// Helper functions for MongoDB operations
async fn get_mongo_client(
    mongo_state: &State<'_, MongoConnection>,
    log_state: &State<'_, LogState>,
) -> Result<Client, String> {
    let client_guard = mongo_state.0.lock().await;

    if let Some(client) = &*client_guard {
        return Ok(client.clone());
    }

    drop(client_guard);

    init_mongodb(mongo_state.clone(), log_state.clone()).await?;

    let client_guard = mongo_state.0.lock().await;
    match &*client_guard {
        Some(client) => Ok(client.clone()),
        None => Err("Failed to initialize MongoDB connection".to_string())
    }
}

async fn get_db(
    mongo_state: &State<'_, MongoConnection>,
    log_state: &State<'_, LogState>,
) -> Result<Database, String> {
    let client = get_mongo_client(mongo_state, log_state).await?;
    Ok(client.database(DB_NAME))
}

async fn get_collection<T>(
    mongo_state: &State<'_, MongoConnection>,
    log_state: &State<'_, LogState>,
    collection_name: &str,
) -> Result<Collection<T>, String>
where
    T: DeserializeOwned + Serialize + Send + Sync + Unpin,
{
    let db = get_db(mongo_state, log_state).await?;
    Ok(db.collection::<T>(collection_name))
}

// Generic function to fetch documents from MongoDB
async fn fetch_documents<T, F>(
    collection: Collection<T>,
    filter: F,
    log_state: &State<'_, LogState>,
    log_msg_prefix: &str,
) -> Result<Vec<T>, String>
where
    T: DeserializeOwned + Send + Sync + Unpin,
    F: Into<Option<mongodb::bson::Document>>,
{
    let mut cursor = collection
        .find(filter, None)
        .await
        .map_err(|e| e.to_string())?;

    let mut documents = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => documents.push(document),
            Err(e) => log_message(log_state, format!("Error fetching document: {}", e)),
        }
    }

    log_message(log_state, format!("{}: {}", log_msg_prefix, documents.len()));
    Ok(documents)
}

// Initialize MongoDB client
#[tauri::command]
async fn init_mongodb(
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<bool, String> {
    let mut client_opt = mongo_state.0.lock().await;

    if client_opt.is_some() {
        return Ok(true);
    }

    let client_options = ClientOptions::parse(MONGO_URI)
        .await
        .map_err(|e| e.to_string())?;

    let client = Client::with_options(client_options)
        .map_err(|e| e.to_string())?;

    // Verify connection
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await
        .map_err(|e| e.to_string())?;

    log_message(&log_state, "Connected to MongoDB successfully".to_string());

    *client_opt = Some(client);
    Ok(true)
}

// Database operations
#[tauri::command]
async fn save_package_collection(
    package_data: PackageCollection,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<bool, String> {
    let collection = get_collection::<PackageCollection>(&mongo_state, &log_state, "package_collections").await?;

    collection
        .insert_one(package_data.clone(), None)
        .await
        .map_err(|e| e.to_string())?;

    log_message(
        &log_state,
        format!("Package collection '{}' saved to MongoDB", package_data.version)
    );

    Ok(true)
}

#[tauri::command]
async fn save_stage_to_mongodb(
    stage_data: Stage,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<bool, String> {
    let collection = get_collection::<Stage>(&mongo_state, &log_state, "stages").await?;

    // First, update all existing stages with the same name to set active = false
    let filter = doc! { "name": &stage_data.name };
    let update = doc! { "$set": { "active": false } };

    collection
        .update_many(filter, update, None)
        .await
        .map_err(|e| e.to_string())?;

    log_message(
        &log_state,
        format!("Set active=false for all existing stages with name '{}'", stage_data.name)
    );

    // Now insert the new stage
    collection
        .insert_one(stage_data.clone(), None)
        .await
        .map_err(|e| e.to_string())?;

    log_message(
        &log_state,
        format!("Stage '{}' saved to MongoDB", stage_data.name)
    );

    Ok(true)
}

#[tauri::command]
async fn get_package_collections(
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<Vec<PackageCollection>, String> {
    let collection = get_collection::<PackageCollection>(&mongo_state, &log_state, "package_collections").await?;
    fetch_documents(collection, None, &log_state, "Retrieved package collections").await
}

#[tauri::command]
async fn get_package_collections_by_uri(
    uri: String,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<PackageCollectionResult, String> {
    let collection = get_collection::<PackageCollection>(&mongo_state, &log_state, "package_collections").await?;
    let filter = doc! { "uri": &uri };

    let packages = fetch_documents(
        collection,
        filter,
        &log_state,
        &format!("Retrieved package collections with URI: {}", uri)
    ).await?;

    if packages.is_empty() {
        Ok(PackageCollectionResult {
            success: true,
            message: Some(format!("no collection found in {}", uri)),
            collections: None,
        })
    } else {
        Ok(PackageCollectionResult {
            success: true,
            message: None,
            collections: Some(packages),
        })
    }
}

#[tauri::command]
async fn get_all_package_collections(
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<PackageCollectionResult, String> {
    let collection = get_collection::<PackageCollection>(&mongo_state, &log_state, "package_collections").await?;

    let packages = fetch_documents(
        collection,
        None,
        &log_state,
        "Retrieved package collections for dropdown population"
    ).await?;

    if packages.is_empty() {
        Ok(PackageCollectionResult {
            success: true,
            message: Some("No package collections found in database".to_string()),
            collections: None,
        })
    } else {
        Ok(PackageCollectionResult {
            success: true,
            message: None,
            collections: Some(packages),
        })
    }
}

#[tauri::command]
async fn get_package_collection_tools(
    version: String,
    uri: String,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<Vec<String>, String> {
    let collection = get_collection::<PackageCollection>(&mongo_state, &log_state, "package_collections").await?;

    let filter = doc! {
        "version": &version,
        "uri": &uri
    };

    match collection.find_one(filter, None).await {
        Ok(Some(package)) => {
            log_message(&log_state, format!("Found package collection with {} tools", package.tools.len()));
            Ok(package.tools)
        },
        Ok(None) => {
            log_message(&log_state, format!("Package collection not found with version {} and URI {}", version, uri));
            Ok(vec![])
        },
        Err(e) => {
            let error_msg = format!("Error fetching package collection: {}", e);
            log_message(&log_state, error_msg.clone());
            Err(error_msg)
        }
    }
}

#[tauri::command]
async fn get_stages_by_uri(
    uri: String,
    active_only: Option<bool>,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<Vec<Stage>, String> {
    let collection = get_collection::<Stage>(&mongo_state, &log_state, "stages").await?;

    // Build filter based on URI and active status
    let mut filter = doc! { "uri": &uri };

    // If active_only is provided and true, filter by active status
    if let Some(true) = active_only {
        filter.insert("active", true);
        log_message(&log_state, format!("Filtering for active stages with URI: {}", uri));
    }

    let filter_status = if active_only.unwrap_or(false) { "active " } else { "" };
    let log_msg = format!("Retrieved {}stages with URI: {}", filter_status, uri);

    fetch_documents(collection, filter, &log_state, &log_msg).await
}

#[tauri::command]
async fn revert_stage(
    stage_id: String,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<bool, String> {
    let collection = get_collection::<Stage>(&mongo_state, &log_state, "stages").await?;

    // First, get the stage to revert to
    let object_id = ObjectId::parse_str(&stage_id).map_err(|e| e.to_string())?;
    let filter = doc! { "_id": object_id };
    let stage_to_activate = collection
        .find_one(filter, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Stage not found".to_string())?;

    // Get the stage name and URI
    let stage_name = stage_to_activate.name.clone();
    let stage_uri = stage_to_activate.uri.clone();

    log_message(
        &log_state,
        format!("Reverting stage '{}' with URI '{}'", stage_name, stage_uri)
    );

    // First, set all stages with the same name to inactive
    let filter = doc! {
        "name": &stage_name,
        "uri": &stage_uri
    };
    let update = doc! { "$set": { "active": false } };

    collection
        .update_many(filter, update, None)
        .await
        .map_err(|e| e.to_string())?;

    log_message(
        &log_state,
        format!("Set active=false for all existing stages with name '{}'", stage_name)
    );

    // Now set the selected stage to active
    let filter = doc! { "_id": object_id };
    let update = doc! { "$set": { "active": true } };

    collection
        .update_one(filter, update, None)
        .await
        .map_err(|e| e.to_string())?;

    log_message(
        &log_state,
        format!("Set stage '{}' to active", stage_name)
    );

    Ok(true)
}

#[tauri::command]
async fn get_stage_history(
    stage_name: String,
    uri: String,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<Vec<Stage>, String> {
    let collection = get_collection::<Stage>(&mongo_state, &log_state, "stages").await?;

    // Find all stages with the same name and URI
    let filter = doc! {
        "name": &stage_name,
        "uri": &uri
    };

    let log_msg = format!("Retrieved stage versions for '{}' with URI '{}'", stage_name, uri);
    fetch_documents(collection, filter, &log_state, &log_msg).await
}

#[tauri::command]
fn get_current_username() -> Result<String, String> {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .map_err(|e| format!("Failed to get username: {}", e))
}

// Utility functions
fn log_message(log_state: &State<LogState>, message: String) {
    let mut log_file = match log_state.0.lock() {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to lock log file: {}", e);
            return;
        }
    };

    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let log_entry = format!("[{}] {}\n", timestamp, message);

    if let Err(e) = log_file.write_all(log_entry.as_bytes()) {
        eprintln!("Failed to write to log file: {}", e);
    }

    #[cfg(debug_assertions)]
    println!("{}", log_entry.trim());
}

fn init_log_file() -> Result<File, String> {
    let temp_dir = std::env::temp_dir();
    let log_dir = temp_dir.join("rezlauncher_logs");

    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let log_path = log_dir.join(format!("rezlauncher_{}.log", timestamp));

    OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))
}

fn main() {
    let log_file = match init_log_file() {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to initialize log file: {}", e);
            std::process::exit(1);
        }
    };

    let mongo_connection = MongoConnection(TokioMutex::new(None));
    let log_state = LogState(Mutex::new(log_file));

    tauri::Builder::default()
        .manage(mongo_connection)
        .manage(log_state)
        .invoke_handler(tauri::generate_handler![
            init_mongodb,
            save_package_collection,
            save_stage_to_mongodb,
            get_package_collections,
            get_package_collections_by_uri,
            get_current_username,
            get_all_package_collections,
            get_package_collection_tools,
            get_stages_by_uri,
            revert_stage,
            get_stage_history,
        ])
        .setup(|app| {
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<MongoConnection>();
                let log_state = app_handle.state::<LogState>();
                if let Err(e) = init_mongodb(state, log_state).await {
                    eprintln!("Failed to initialize MongoDB: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
