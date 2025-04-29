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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct PackageCollection {
    version: String,
    packages: Vec<String>,
    herit: String,
    tools: Vec<String>,
    created_at: String,
    created_by: String,
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Stage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    uri: String,
    from_version: String,
    rxt_path: String,
    tools: Vec<String>,
    created_at: String,
    created_by: String,
    active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    // Get DB connection
    let db = get_db(&mongo_state, &log_state).await?;
    // Call the core logic function
    fetch_package_collections_by_uri_logic(&db, &log_state, &uri, "package_collections").await
}

#[tauri::command]
async fn get_all_package_collections(
    mongo_state: State<'_, MongoConnection>,
    log_state: State<'_, LogState>,
) -> Result<PackageCollectionResult, String> {
    // Get DB connection
    let db = get_db(&mongo_state, &log_state).await?;
    // Call the core logic function
    fetch_all_package_collections_logic(&db, &log_state, "package_collections").await
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
fn log_message(log_state: &LogState, message: String) {
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
                let log_state_setup = app_handle.state::<LogState>();
                if let Err(e) = init_mongodb(state.clone(), log_state_setup.clone()).await {
                     eprintln!("Failed to initialize MongoDB during setup: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Helper function to fetch documents, extracted for potential reuse if needed elsewhere
async fn fetch_documents_logic<T>(
    collection: &Collection<T>,
    filter: Option<mongodb::bson::Document>,
    log_state: &LogState, // Changed to reference
    log_msg_prefix: &str,
) -> Result<Vec<T>, String>
where
    T: DeserializeOwned + Send + Sync + Unpin + Clone + std::fmt::Debug, // Added Clone and Debug constraints
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

    #[cfg(debug_assertions)]
    {
        let doc_count = documents.len();
        log_message(log_state, format!("{}: {} documents retrieved.", log_msg_prefix, doc_count));
        if doc_count < 5 {
             log_message(log_state, format!("First few documents: {:?}", documents.iter().take(5).collect::<Vec<_>>()));
        }
    }
    #[cfg(not(debug_assertions))]
    log_message(log_state, format!("{}: {}", log_msg_prefix, documents.len()));


    Ok(documents)
}

async fn fetch_package_collections_by_uri_logic(
    db: &Database,
    log_state: &LogState,
    uri: &str,
    collection_name: &str,
) -> Result<PackageCollectionResult, String> {
    let collection = db.collection::<PackageCollection>(collection_name);
    let filter = doc! { "uri": uri };

    let packages = fetch_documents_logic(
        &collection,
        Some(filter),
        log_state,
        &format!("Retrieved package collections from {} with URI: {}", collection_name, uri)
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

async fn fetch_all_package_collections_logic(
    db: &Database,
    log_state: &LogState,
    collection_name: &str,
) -> Result<PackageCollectionResult, String> {
    let collection = db.collection::<PackageCollection>(collection_name);

    let packages = fetch_documents_logic(
        &collection,
        None,
        log_state,
        &format!("Retrieved package collections from {} for dropdown population", collection_name)
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

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::Database;
    use rand::{distributions::Alphanumeric, Rng};
    use std::fs;
    use std::io::Read;
    use std::path::PathBuf;

    const TEST_MONGO_URI: &str = "mongodb://localhost:27017";
    const TEST_DB_NAME: &str = "rez_launcher_test_db";

    fn generate_random_suffix(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }


    // Helper to get a test database connection and ensure cleanup
    // Returns Client, Database, LogState, Log Path, and Unique Collection Name
    async fn setup_test_db() -> Result<(Client, Database, LogState, PathBuf, String), String> {
        let unique_suffix = generate_random_suffix(8);
        let collection_name = format!("package_collections_{}", unique_suffix);
        let log_file_name = format!("test_log_{}.log", unique_suffix);

        // Initialize LogState for tests (writes to a temp file with unique name)
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join(&log_file_name); // Use unique log file name
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| format!("Failed to open test log file {:?}: {}", log_path, e))?;
        let log_state = LogState(Mutex::new(log_file));


        let client_options = ClientOptions::parse(TEST_MONGO_URI)
            .await
            .map_err(|e| format!("Failed to parse test MongoDB URI: {}", e))?;
        let client = Client::with_options(client_options)
             .map_err(|e| format!("Failed to create test MongoDB client: {}", e))?;

        // Ping to ensure connection
        client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await
            .map_err(|e| format!("Failed to ping test MongoDB: {}. Ensure MongoDB is running at {}", e, TEST_MONGO_URI))?;

        let db = client.database(TEST_DB_NAME);
        // Clear the specific test collection before the run
        db.collection::<PackageCollection>(&collection_name)
          .drop(None)
          .await
          .map_err(|e| format!("Failed to drop test collection '{}': {}", collection_name, e))?;

        Ok((client, db, log_state, log_path, collection_name)) // Return log_path and collection_name
    }

    // Helper to create a dummy PackageCollection
    fn create_dummy_package_collection(version: &str, uri: &str) -> PackageCollection {
        PackageCollection {
            version: version.to_string(),
            packages: vec!["pkg1".to_string(), "pkg2".to_string()],
            herit: "parent".to_string(),
            tools: vec!["toolA".to_string(), "toolB".to_string()],
            created_at: Utc::now().to_rfc3339(),
            created_by: "test_user".to_string(),
            uri: uri.to_string(),
        }
    }

    // --- Test Cases ---

    #[tokio::test]
    async fn test_fetch_package_collections_by_uri_found() {
        // Get unique collection name from setup
        let (_client, db, log_state, _log_path, collection_name) = setup_test_db().await.expect("Test DB setup failed");
        // Use unique collection name
        let collection = db.collection::<PackageCollection>(&collection_name);

        let uri1 = "test/uri/1";
        let uri2 = "test/uri/2";
        let pkg1 = create_dummy_package_collection("1.0", uri1);
        let pkg2 = create_dummy_package_collection("2.0", uri1);
        let pkg3 = create_dummy_package_collection("1.0", uri2); // Different URI

        collection.insert_many(vec![pkg1.clone(), pkg2.clone(), pkg3.clone()], None).await.unwrap();

        // Pass unique collection name to logic function
        let result = fetch_package_collections_by_uri_logic(&db, &log_state, uri1, &collection_name).await;

        assert!(result.is_ok());
        let package_result = result.unwrap();
        assert!(package_result.success);
        assert!(package_result.message.is_none());
        assert!(package_result.collections.is_some());
        let collections = package_result.collections.unwrap();
        assert_eq!(collections.len(), 2); // Assertion should now pass
        // Order isn't guaranteed, so check contents carefully or sort
        assert!(collections.contains(&pkg1));
        assert!(collections.contains(&pkg2));
    }

    #[tokio::test]
    async fn test_fetch_package_collections_by_uri_not_found() {
        // Get unique collection name from setup
        let (_client, db, log_state, _log_path, collection_name) = setup_test_db().await.expect("Test DB setup failed");
        // Use unique collection name
        let collection = db.collection::<PackageCollection>(&collection_name);

        let uri1 = "test/uri/1";
        let pkg1 = create_dummy_package_collection("1.0", uri1);
        collection.insert_one(pkg1.clone(), None).await.unwrap();

        let non_existent_uri = "test/uri/non_existent";
        // Pass unique collection name to logic function
        let result = fetch_package_collections_by_uri_logic(&db, &log_state, non_existent_uri, &collection_name).await;

        assert!(result.is_ok());
        let package_result = result.unwrap();
        assert!(package_result.success);
        assert!(package_result.message.is_some());
        assert_eq!(package_result.message.unwrap(), format!("no collection found in {}", non_existent_uri));
        assert!(package_result.collections.is_none());
    }

     #[tokio::test]
    async fn test_fetch_package_collections_by_uri_empty_db() {
        // Get unique collection name from setup
        let (_client, db, log_state, _log_path, collection_name) = setup_test_db().await.expect("Test DB setup failed");

        let uri1 = "test/uri/1";
        // Pass unique collection name to logic function
        let result = fetch_package_collections_by_uri_logic(&db, &log_state, uri1, &collection_name).await;

        assert!(result.is_ok());
        let package_result = result.unwrap();
        assert!(package_result.success);
        assert!(package_result.message.is_some());
        assert_eq!(package_result.message.unwrap(), format!("no collection found in {}", uri1));
        assert!(package_result.collections.is_none());
    }

    #[tokio::test]
    async fn test_fetch_all_package_collections_found() {
        // Get unique collection name from setup
        let (_client, db, log_state, _log_path, collection_name) = setup_test_db().await.expect("Test DB setup failed");
        // Use unique collection name
        let collection = db.collection::<PackageCollection>(&collection_name);

        let uri1 = "test/uri/1";
        let uri2 = "test/uri/2";
        let pkg1 = create_dummy_package_collection("1.0", uri1);
        let pkg2 = create_dummy_package_collection("2.0", uri1);
        let pkg3 = create_dummy_package_collection("1.0", uri2);

        collection.insert_many(vec![pkg1.clone(), pkg2.clone(), pkg3.clone()], None).await.unwrap();

        // Pass unique collection name to logic function
        let result = fetch_all_package_collections_logic(&db, &log_state, &collection_name).await;

        assert!(result.is_ok());
        let package_result = result.unwrap();
        assert!(package_result.success);
        assert!(package_result.message.is_none());
        assert!(package_result.collections.is_some());
        let collections = package_result.collections.unwrap();
        assert_eq!(collections.len(), 3); // Assertion should now pass
        assert!(collections.contains(&pkg1));
        assert!(collections.contains(&pkg2));
        assert!(collections.contains(&pkg3));
    }

    #[tokio::test]
    async fn test_fetch_all_package_collections_empty_db() {
         // Get unique collection name from setup
         let (_client, db, log_state, _log_path, collection_name) = setup_test_db().await.expect("Test DB setup failed");

        // Pass unique collection name to logic function
        let result = fetch_all_package_collections_logic(&db, &log_state, &collection_name).await;

        assert!(result.is_ok());
        let package_result = result.unwrap();
        assert!(package_result.success);
        assert!(package_result.message.is_some());
        assert_eq!(package_result.message.unwrap(), "No package collections found in database");
        assert!(package_result.collections.is_none());
    }

    // --- Test Logging (Basic Check) ---
    #[tokio::test]
    async fn test_logging_writes_to_file() {
        // Get log_path directly from setup, mark db and collection_name as unused
        let (_client, _db, log_state, log_path, _collection_name) = setup_test_db().await.expect("Test DB setup failed");
        // Removed brittle path finding logic


        let test_message = "This is a test log message.";
        log_message(&log_state, test_message.to_string());

        // Drop the guard to ensure the file is flushed/closed before reading
        drop(log_state);

        // Read the log file content using the known path
        let mut file_content = String::new();
        // Add error handling for file opening
        let mut file = File::open(&log_path)
            .map_err(|e| format!("Failed to reopen log file {:?} for reading: {}", log_path, e))
            .expect("Failed to reopen log file");
        file.read_to_string(&mut file_content).expect("Failed to read log file content");

        // Check if the message exists (might have timestamp prepended)
        assert!(file_content.contains(test_message), "Log file content did not contain the test message. Content: '{}'", file_content);

        // Clean up the log file
        let _ = fs::remove_file(&log_path);
    }
}
