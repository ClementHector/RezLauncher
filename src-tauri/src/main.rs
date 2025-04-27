// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::sync::Mutex;
use std::fs::{OpenOptions, File};
use std::io::Write;
use tauri::State;
use tauri::Manager;
use futures::stream::StreamExt;
use tokio::sync::Mutex as TokioMutex;

// MongoDB connection struct
struct MongoConnection(TokioMutex<Option<Client>>);

// App state for logging
struct LogState(Mutex<File>);

// Package Collection data structure
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackageCollection {
    version: String,
    packages: Vec<String>,
    herit: String,
    tools: Vec<String>, // Changed from aliases to tools
    created_at: String,
    created_by: String,
    uri: String,
}

// Stage data structure
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

// Result structure for get_package_collections_by_uri
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackageCollectionResult {
    success: bool,
    message: Option<String>,
    collections: Option<Vec<PackageCollection>>,
}

// Helper function to get MongoDB client, initializing if needed
async fn get_mongo_client(
    mongo_state: &State<'_, MongoConnection>,
    log_state: &State<'_, LogState>,
) -> Result<Client, String> {
    let client_guard = mongo_state.0.lock().await;

    if let Some(client) = &*client_guard {
        return Ok(client.clone());
    }

    // Drop the guard before async operation
    drop(client_guard);

    // Initialize MongoDB
    init_mongodb(mongo_state.clone(), log_state.clone()).await?;

    // Get the initialized client
    let client_guard = mongo_state.0.lock().await;
    match &*client_guard {
        Some(client) => Ok(client.clone()),
        None => Err("Failed to initialize MongoDB connection".to_string())
    }
}

// Initialize MongoDB client
#[tauri::command]
async fn init_mongodb(
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<bool, String> {
    let mut client_opt = mongo_state.0.lock().await;

    if client_opt.is_some() {
        return Ok(true); // Already initialized
    }

    let connection_string = "mongodb://localhost:27017"; // Configuration should come from a secure source in production
    let client_options = ClientOptions::parse(connection_string)
        .await
        .map_err(|e| e.to_string())?;

    let client = Client::with_options(client_options)
        .map_err(|e| e.to_string())?;

    // Test connection
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await
        .map_err(|e| e.to_string())?;

    log_message(&log_state, "Connected to MongoDB successfully".to_string());

    *client_opt = Some(client);
    Ok(true)
}

// Save package collection to MongoDB
#[tauri::command]
async fn save_package_collection(
    package_data: PackageCollection,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<bool, String> {
    // Get MongoDB client using helper function
    let client = get_mongo_client(&mongo_state, &log_state).await?;

    let db = client.database("rez_launcher");
    let collection = db.collection::<PackageCollection>("package_collections");

    // Insert the document
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

// Save stage to MongoDB
#[tauri::command]
async fn save_stage_to_mongodb(
    stage_data: Stage,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<bool, String> {
    // Get MongoDB client using helper function
    let client = get_mongo_client(&mongo_state, &log_state).await?;

    let db = client.database("rez_launcher");
    let collection = db.collection::<Stage>("stages");

    // Insert the document
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

// Get all package collections
#[tauri::command]
async fn get_package_collections(
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<Vec<PackageCollection>, String> {
    // Get MongoDB client using helper function
    let client = get_mongo_client(&mongo_state, &log_state).await?;

    let db = client.database("rez_launcher");
    let collection = db.collection::<PackageCollection>("package_collections");

    // Find all documents
    let mut cursor = collection
        .find(None, None)
        .await
        .map_err(|e| e.to_string())?;

    let mut packages = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => packages.push(document),
            Err(e) => log_message(&log_state, format!("Error fetching package: {}", e)),
        }
    }

    log_message(&log_state, format!("Retrieved {} package collections", packages.len()));

    Ok(packages)
}

// Get package collections by URI
#[tauri::command]
async fn get_package_collections_by_uri(
    uri: String,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<PackageCollectionResult, String> {
    // Get MongoDB client using helper function
    let client = get_mongo_client(&mongo_state, &log_state).await?;

    let db = client.database("rez_launcher");
    let collection = db.collection::<PackageCollection>("package_collections");

    // Find documents matching the URI
    let filter = doc! { "uri": &uri };
    let mut cursor = collection
        .find(filter, None)
        .await
        .map_err(|e| e.to_string())?;

    let mut packages = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => packages.push(document),
            Err(e) => log_message(&log_state, format!("Error fetching package: {}", e)),
        }
    }

    log_message(&log_state, format!("Retrieved {} package collections with URI: {}", packages.len(), uri));

    // Return the result with appropriate message if no collections are found
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

// Get all package collections for dropdown
#[tauri::command]
async fn get_all_package_collections(
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<PackageCollectionResult, String> {
    // Get MongoDB client using helper function
    let client = get_mongo_client(&mongo_state, &log_state).await?;

    let db = client.database("rez_launcher");
    let collection = db.collection::<PackageCollection>("package_collections");

    // Find all documents
    let mut cursor = collection
        .find(None, None)
        .await
        .map_err(|e| e.to_string())?;

    let mut packages = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => packages.push(document),
            Err(e) => log_message(&log_state, format!("Error fetching package: {}", e)),
        }
    }

    log_message(&log_state, format!("Retrieved {} package collections for dropdown population", packages.len()));

    // Return the result with appropriate message if no collections are found
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

// Get tools for a specific package collection
#[tauri::command]
async fn get_package_collection_tools(
    version: String,
    uri: String,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<Vec<String>, String> {
    // Get MongoDB client using helper function
    let client = get_mongo_client(&mongo_state, &log_state).await?;

    log_message(&log_state, format!("Fetching tools for package collection version: {} uri: {}", version, uri));

    let db = client.database("rez_launcher");
    let collection = db.collection::<PackageCollection>("package_collections");

    // Find the specific package collection
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

// Get stages by URI
#[tauri::command]
async fn get_stages_by_uri(
    uri: String,
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<Vec<Stage>, String> {
    // Get MongoDB client using helper function
    let client = get_mongo_client(&mongo_state, &log_state).await?;

    log_message(&log_state, format!("Fetching stages with URI: {}", uri));

    let db = client.database("rez_launcher");
    let collection = db.collection::<Stage>("stages");

    // Find documents matching the URI
    let filter = doc! { "uri": &uri };
    let mut cursor = collection
        .find(filter, None)
        .await
        .map_err(|e| e.to_string())?;

    let mut stages = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => stages.push(document),
            Err(e) => log_message(&log_state, format!("Error fetching stage: {}", e)),
        }
    }

    log_message(&log_state, format!("Retrieved {} stages with URI: {}", stages.len(), uri));

    Ok(stages)
}

// Get current OS username
#[tauri::command]
fn get_current_username() -> Result<String, String> {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .map_err(|e| format!("Failed to get username: {}", e))
}

// Utility function to log messages
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

    // Also print to console in debug mode
    #[cfg(debug_assertions)]
    println!("{}", log_entry.trim());
}

// Initialize log file
fn init_log_file() -> Result<File, String> {
    // Utiliser le dossier temporaire du système au lieu d'un dossier fixe
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
    // Initialize log file
    let log_file = match init_log_file() {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to initialize log file: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize MongoDB client
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
        ])
        .setup(|app| {
            // Initialize MongoDB on startup
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
