#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use async_trait::async_trait;
use mongodb::{Client, options::ClientOptions, Collection, Database};
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use std::fs::{self, OpenOptions, File};
use std::io::{Read, Write};
use std::process::Command;
use rand::Rng;
use tauri::State;
use futures::stream::StreamExt;
use once_cell::sync::Lazy;

// Configuration par défaut de MongoDB (utilisée si aucune configuration n'est fournie)
const DEFAULT_MONGO_URI: &str = "mongodb://localhost:27017";
const DB_NAME: &str = "rez_launcher";

// Variable globale pour stocker l'URI MongoDB actuelle
static MONGO_URI: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(DEFAULT_MONGO_URI.to_string()));

#[cfg_attr(test, mockall::automock)]
#[async_trait]
trait DbRepository: Send + Sync {
    async fn find_package_collections_by_uri(&self, uri: &str) -> Result<Vec<PackageCollection>, String>;
    async fn find_all_package_collections(&self) -> Result<Vec<PackageCollection>, String>;
    async fn insert_package_collection(&self, package_data: PackageCollection) -> Result<(), String>;
    async fn find_package_collection_tools(&self, version: &str, uri: &str) -> Result<Option<Vec<String>>, String>;
    async fn find_stages_by_uri(&self, uri: &str, active_only: Option<bool>) -> Result<Vec<Stage>, String>;
    async fn insert_stage(&self, stage_data: Stage) -> Result<(), String>;
    async fn update_stages_active_status(&self, name: &str, uri: &str, active: bool) -> Result<(), String>;
    async fn update_stage_active_status_by_id(&self, id: ObjectId, active: bool) -> Result<(), String>;
    async fn find_stage_by_id(&self, id: ObjectId) -> Result<Option<Stage>, String>;
    async fn find_stage_history(&self, stage_name: &str, uri: &str) -> Result<Vec<Stage>, String>;
    async fn find_distinct_stage_names(&self) -> Result<Vec<String>, String>;
}

struct MongoDbRepository {
    db: Database,
    log_state: LogState,
}

impl MongoDbRepository {
    fn get_collection<T>(&self, name: &str) -> Collection<T> {
        self.db.collection::<T>(name)
    }

    async fn fetch_documents_internal<T>(
        &self,
        collection_name: &str,
        filter: impl Into<Option<mongodb::bson::Document>>,
        log_msg_prefix: &str,
    ) -> Result<Vec<T>, String>
    where
        T: DeserializeOwned + Send + Sync + Unpin + Clone + std::fmt::Debug,
    {
        let collection = self.get_collection::<T>(collection_name);
        let mut cursor = collection
            .find(filter, None)
            .await
            .map_err(|e| e.to_string())?;

        let mut documents = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => documents.push(document),
                Err(e) => log_message(&self.log_state, format!("Error fetching document: {}", e)),
            }
        }

        #[cfg(debug_assertions)]
        {
            let doc_count = documents.len();
            log_message(&self.log_state, format!("{}: {} documents retrieved.", log_msg_prefix, doc_count));
        }
        #[cfg(not(debug_assertions))]
        log_message(&self.log_state, format!("{}: {}", log_msg_prefix, documents.len()));

        Ok(documents)
    }
}


#[async_trait]
impl DbRepository for MongoDbRepository {
    async fn find_package_collections_by_uri(&self, uri: &str) -> Result<Vec<PackageCollection>, String> {
        let filter = doc! { "uri": uri };
        self.fetch_documents_internal(
            "package_collections",
            filter,
            &format!("Retrieved package collections with URI: {}", uri)
        ).await
    }

    async fn find_all_package_collections(&self) -> Result<Vec<PackageCollection>, String> {
        self.fetch_documents_internal(
            "package_collections",
            None,
            "Retrieved all package collections"
        ).await
    }

     async fn insert_package_collection(&self, package_data: PackageCollection) -> Result<(), String> {
        let collection = self.get_collection::<PackageCollection>("package_collections");
        collection
            .insert_one(package_data, None)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn find_package_collection_tools(&self, version: &str, uri: &str) -> Result<Option<Vec<String>>, String> {
        let collection = self.get_collection::<PackageCollection>("package_collections");
        let filter = doc! { "version": version, "uri": uri };
        match collection.find_one(filter, None).await {
            Ok(Some(package)) => Ok(Some(package.tools)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn find_stages_by_uri(&self, uri: &str, active_only: Option<bool>) -> Result<Vec<Stage>, String> {
        let mut filter = doc! { "uri": uri };
        if let Some(true) = active_only {
            filter.insert("active", true);
        }
        let filter_status = if active_only.unwrap_or(false) { "active " } else { "" };
        let log_msg = format!("Retrieved {}stages with URI: {}", filter_status, uri);
        self.fetch_documents_internal("stages", filter, &log_msg).await
    }

     async fn insert_stage(&self, stage_data: Stage) -> Result<(), String> {
        let collection = self.get_collection::<Stage>("stages");
        collection
            .insert_one(stage_data, None)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn update_stages_active_status(&self, name: &str, uri: &str, active: bool) -> Result<(), String> {
        let collection = self.get_collection::<Stage>("stages");
        let filter = doc! { "name": name, "uri": uri };
        let update = doc! { "$set": { "active": active } };
        collection
            .update_many(filter, update, None)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

     async fn update_stage_active_status_by_id(&self, id: ObjectId, active: bool) -> Result<(), String> {
        let collection = self.get_collection::<Stage>("stages");
        let filter = doc! { "_id": id };
        let update = doc! { "$set": { "active": active } };
        collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn find_stage_by_id(&self, id: ObjectId) -> Result<Option<Stage>, String> {
        let collection = self.get_collection::<Stage>("stages");
        let filter = doc! { "_id": id };
        collection
            .find_one(filter, None)
            .await
            .map_err(|e| e.to_string())
    }

    async fn find_stage_history(&self, stage_name: &str, uri: &str) -> Result<Vec<Stage>, String> {
        let filter = doc! { "name": stage_name, "uri": uri };
        let log_msg = format!("Retrieved stage versions for '{}' with URI '{}'", stage_name, uri);
        self.fetch_documents_internal("stages", filter, &log_msg).await
    }

    async fn find_distinct_stage_names(&self) -> Result<Vec<String>, String> {
        let collection = self.get_collection::<Stage>("stages");
        log_message(&self.log_state, "Fetching all unique stage names".to_string());
        match collection.distinct("name", None, None).await {
            Ok(names_bson) => {
                let names: Vec<String> = names_bson.into_iter()
                    .filter_map(|bson| match bson {
                        Bson::String(s) => Some(s),
                        _ => {
                            log_message(&self.log_state, format!("Non-string value found in distinct stage names: {:?}", bson));
                            None
                        }
                    })
                    .collect();
                log_message(&self.log_state, format!("Retrieved {} unique stage names", names.len()));
                Ok(names)
            }
            Err(e) => {
                let error_msg = format!("Error fetching distinct stage names: {}", e);
                log_message(&self.log_state, error_msg.clone());
                Err(error_msg)
            }
        }
    }
}

struct LogState(Mutex<File>);

struct AppState {
    db_repo: Arc<dyn DbRepository>,
    log_state: LogState,
}

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
    rxt: String,
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

    if (!log_dir.exists()) {
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

#[tauri::command]
async fn init_command() -> Result<bool, String> {
    Ok(true)
}

#[tauri::command]
async fn save_package_collection(
    package_data: PackageCollection,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state.db_repo.insert_package_collection(package_data.clone()).await?;
    log_message(
        &state.log_state,
        format!("Package collection '{}' saved via repository", package_data.version)
    );
    Ok(true)
}

#[tauri::command]
async fn save_stage_to_mongodb(
    stage_data: Stage,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // First, find the source package collection to get the list of packages
    let source_package = state.db_repo.find_package_collections_by_uri(&stage_data.uri)
        .await?
        .into_iter()
        .find(|pkg| pkg.version == stage_data.from_version);

    let packages = match source_package {
        Some(pkg) => pkg.packages,
        None => {
            let error_msg = format!("Package collection {} not found for RXT generation", stage_data.from_version);
            log_message(&state.log_state, error_msg.clone());
            return Err(error_msg);
        }
    };

    // Generate the RXT file content from the package list
    log_message(
        &state.log_state,
        format!("Generating RXT file for stage '{}' with {} packages", stage_data.name, packages.len())
    );

    let rxt_content = match generate_rxt_file(&packages, &state.log_state).await {
        Ok(content) => {
            log_message(
                &state.log_state,
                format!("Successfully generated RXT file for stage '{}'", stage_data.name)
            );
            content
        },
        Err(e) => {
            let error_msg = format!("Failed to generate RXT file: {}", e);
            log_message(&state.log_state, error_msg.clone());
            return Err(error_msg);
        }
    };

    // Set all existing stages with the same name and URI to inactive
    state.db_repo.update_stages_active_status(&stage_data.name, &stage_data.uri, false).await?;

    log_message(
        &state.log_state,
        format!("Set active=false for all existing stages with name '{}' via repository", stage_data.name)
    );

    // Create the new stage with the RXT content and set it to active
    let mut stage_to_insert = stage_data.clone();
    stage_to_insert.active = true;
    stage_to_insert.rxt = rxt_content;

    state.db_repo.insert_stage(stage_to_insert).await?;

    log_message(
        &state.log_state,
        format!("Stage '{}' saved via repository with RXT content", stage_data.name)
    );

    Ok(true)
}

#[tauri::command]
async fn get_package_collections_by_uri(
    uri: String,
    state: State<'_, AppState>,
) -> Result<PackageCollectionResult, String> {
    let packages = state.db_repo.find_package_collections_by_uri(&uri).await?;

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
    state: State<'_, AppState>,
) -> Result<PackageCollectionResult, String> {
    let packages = state.db_repo.find_all_package_collections().await?;

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
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    match state.db_repo.find_package_collection_tools(&version, &uri).await? {
        Some(tools) => {
            log_message(&state.log_state, format!("Found package collection with {} tools via repository", tools.len()));
            Ok(tools)
        },
        None => {
            log_message(&state.log_state, format!("Package collection not found with version {} and URI {} via repository", version, uri));
            Ok(vec![])
        }
    }
}

#[tauri::command]
async fn get_stages_by_uri(
    uri: String,
    active_only: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<Stage>, String> {
    state.db_repo.find_stages_by_uri(&uri, active_only).await
}

#[tauri::command]
async fn revert_stage(
    stage_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let object_id = ObjectId::parse_str(&stage_id).map_err(|e| e.to_string())?;

    let stage_to_activate = state.db_repo.find_stage_by_id(object_id).await?
        .ok_or_else(|| "Stage not found".to_string())?;

    let stage_name = stage_to_activate.name.clone();
    let stage_uri = stage_to_activate.uri.clone();

    log_message(
        &state.log_state,
        format!("Reverting stage '{}' with URI '{}' via repository", stage_name, stage_uri)
    );

    state.db_repo.update_stages_active_status(&stage_name, &stage_uri, false).await?;

    log_message(
        &state.log_state,
        format!("Set active=false for all existing stages with name '{}' via repository", stage_name)
    );

    state.db_repo.update_stage_active_status_by_id(object_id, true).await?;

    log_message(
        &state.log_state,
        format!("Set stage '{}' to active via repository", stage_name)
    );

    Ok(true)
}

#[tauri::command]
async fn get_stage_history(
    stage_name: String,
    uri: String,
    state: State<'_, AppState>,
) -> Result<Vec<Stage>, String> {
    state.db_repo.find_stage_history(&stage_name, &uri).await
}

#[tauri::command]
fn get_current_username() -> Result<String, String> {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .map_err(|e| format!("Failed to get username: {}", e))
}

#[tauri::command]
async fn get_all_stage_names(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    state.db_repo.find_distinct_stage_names().await
}

#[tauri::command]
async fn open_tool_in_terminal(tool_name: String, packages: Vec<String>, state: State<'_, AppState>) -> Result<bool, String> {
    log_message(&state.log_state, format!("Attempting to open tool: {} with packages: {:?}", tool_name, packages));

    // Construire la commande rez env avec la liste des packages
    let packages_str = packages.join(" ");
    let rez_command = format!("rez env {} -- {}", packages_str, tool_name);
    log_message(&state.log_state, format!("Executing rez command: {}", rez_command));

    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/c").arg(&rez_command);
        cmd
    } else {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(&rez_command);
        cmd
    };

    match command.spawn() {
        Ok(_) => {
            log_message(&state.log_state, format!("Tool launched successfully in rez environment: {}", tool_name));
            Ok(true)
        },
        Err(e) => {
            log_message(&state.log_state, format!("Failed to launch tool in rez environment: {}", e));
            Err(format!("Failed to launch tool in rez environment: {}", e))
        }
    }
}

#[tauri::command]
async fn open_rez_env_in_terminal(packages: Vec<String>, state: State<'_, AppState>) -> Result<bool, String> {
    log_message(&state.log_state, format!("Attempting to open rez environment with packages: {:?}", packages));

    // Construire la commande rez env avec la liste des packages
    let packages_str = packages.join(" ");
    let rez_command = format!("rez env {}", packages_str);
    log_message(&state.log_state, format!("Executing rez command in new terminal: {}", rez_command));

    let mut command = if cfg!(target_os = "windows") {
        // Sur Windows, utiliser "start cmd" pour ouvrir une nouvelle fenêtre de terminal
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/c").arg("start").arg("cmd").arg("/k").arg(&rez_command);
        cmd
    } else {
        // Sur Linux/Mac, utiliser xterm ou terminal
        let terminal_cmd = if std::path::Path::new("/usr/bin/xterm").exists() {
            "xterm"
        } else if std::path::Path::new("/usr/bin/gnome-terminal").exists() {
            "gnome-terminal"
        } else {
            "x-terminal-emulator"
        };

        let mut cmd = std::process::Command::new(terminal_cmd);
        cmd.arg("-e").arg(format!("bash -c '{} && bash'", rez_command));
        cmd
    };

    match command.spawn() {
        Ok(_) => {
            log_message(&state.log_state, format!("Rez environment opened successfully in new terminal with packages: {}", packages_str));
            Ok(true)
        },
        Err(e) => {
            log_message(&state.log_state, format!("Failed to open rez environment in new terminal: {}", e));
            Err(format!("Failed to open rez environment in new terminal: {}", e))
        }
    }
}

#[tauri::command]
async fn test_mongodb_connection(mongo_uri: String) -> Result<bool, String> {
    // Mettre à jour l'URI globale si la connexion réussit
    match ClientOptions::parse(&mongo_uri).await {
        Ok(options) => {
            match Client::with_options(options) {
                Ok(client) => {
                    // Tester la connexion avec un ping
                    match client.database("admin").run_command(doc! {"ping": 1}, None).await {
                        Ok(_) => {
                            // Connexion réussie, mettre à jour l'URI globale
                            let mut current_uri = MONGO_URI.lock().unwrap();
                            *current_uri = mongo_uri;
                            Ok(true)
                        },
                        Err(e) => {
                            Err(format!("Échec du ping MongoDB: {}", e))
                        }
                    }
                },
                Err(e) => {
                    Err(format!("Impossible de créer le client MongoDB: {}", e))
                }
            }
        },
        Err(e) => {
            Err(format!("URI MongoDB invalide: {}", e))
        }
    }
}

// Generate an RXT file from a list of packages using the rez env command
// Returns the content of the RXT file as a string
async fn generate_rxt_file(packages: &[String], log_state: &LogState) -> Result<String, String> {
    log_message(log_state, format!("Generating RXT file for packages: {:?}", packages));

    // Create a temporary file path
    let temp_dir = std::env::temp_dir();
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let random_suffix: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let temp_file_path = temp_dir.join(format!("rez_env_{}_{}.rxt", timestamp, random_suffix));
    let temp_file_path_str = temp_file_path.to_string_lossy().to_string();

    log_message(log_state, format!("Using temporary file: {}", temp_file_path_str));

    // Build the rez env command
    let packages_str = packages.join(" ");
    let rez_command = format!("rez env {} -o {}", packages_str, temp_file_path_str);
    log_message(log_state, format!("Executing rez command: {}", rez_command));

    // Execute the command
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/c")
            .arg(&rez_command)
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&rez_command)
            .output()
    };

    // Check if command execution was successful
    match output {
        Ok(output) => {
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr).to_string();
                log_message(log_state, format!("Failed to generate RXT file: {}", error));
                return Err(format!("Failed to generate RXT file: {}", error));
            }

            // Read the content of the generated RXT file
            match fs::read_to_string(&temp_file_path) {
                Ok(content) => {
                    log_message(log_state, format!("Successfully read RXT file (size: {} bytes)", content.len()));

                    // Delete the temporary file
                    if let Err(e) = fs::remove_file(&temp_file_path) {
                        log_message(log_state, format!("Warning: Failed to delete temporary RXT file: {}", e));
                    } else {
                        log_message(log_state, format!("Deleted temporary RXT file: {}", temp_file_path_str));
                    }

                    Ok(content)
                },
                Err(e) => {
                    log_message(log_state, format!("Failed to read RXT file: {}", e));
                    Err(format!("Failed to read RXT file: {}", e))
                }
            }
        },
        Err(e) => {
            log_message(log_state, format!("Failed to execute rez command: {}", e));
            Err(format!("Failed to execute rez command: {}", e))
        }
    }
}

#[tauri::command]
async fn load_stage_by_id(
    stage_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // Parse the ObjectId
    let object_id = ObjectId::parse_str(&stage_id).map_err(|e| e.to_string())?;

    // Find the stage by ID
    let stage = state.db_repo.find_stage_by_id(object_id).await?
        .ok_or_else(|| "Stage not found".to_string())?;

    log_message(
        &state.log_state,
        format!("Loading stage '{}' with ID '{}'", stage.name, stage_id)
    );

    if stage.rxt.is_empty() {
        return Err("Stage has no RXT content".to_string());
    }

    // Create a temporary file for the RXT content
    let temp_dir = std::env::temp_dir();
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let random_suffix: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let temp_file_path = temp_dir.join(format!("rez_stage_{}_{}.rxt", timestamp, random_suffix));
    let temp_file_path_str = temp_file_path.to_string_lossy().to_string();

    log_message(&state.log_state, format!("Saving RXT content to temporary file: {}", temp_file_path_str));

    // Write the RXT content to the temporary file
    fs::write(&temp_file_path, &stage.rxt)
        .map_err(|e| format!("Failed to write RXT content to file: {}", e))?;

    // Build the rez command to load the RXT environment
    let rez_command = format!("rez env -i {}", temp_file_path_str);
    log_message(&state.log_state, format!("Executing rez command: {}", rez_command));

    // Execute the command in a new terminal
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/c").arg("start").arg("cmd").arg("/k").arg(&rez_command);
        cmd
    } else {
        // On Linux/Mac, use xterm or terminal
        let terminal_cmd = if std::path::Path::new("/usr/bin/xterm").exists() {
            "xterm"
        } else if std::path::Path::new("/usr/bin/gnome-terminal").exists() {
            "gnome-terminal"
        } else {
            "x-terminal-emulator"
        };

        let mut cmd = std::process::Command::new(terminal_cmd);
        cmd.arg("-e").arg(format!("bash -c '{} && bash'", rez_command));
        cmd
    };

    match command.spawn() {
        Ok(_) => {
            log_message(
                &state.log_state,
                format!("Rez environment loaded successfully for stage '{}' using RXT file", stage.name)
            );
            Ok(true)
        },
        Err(e) => {
            let error_msg = format!("Failed to launch rez environment: {}", e);
            log_message(&state.log_state, error_msg.clone());
            Err(error_msg)
        }
    }
}

fn main() {
    let log_file = match init_log_file() {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to initialize log file: {}", e);
            std::process::exit(1);
        }
    };
    let log_state = LogState(Mutex::new(log_file));

    let app_state = tauri::async_runtime::block_on(async {
        // Récupérer l'URI MongoDB actuelle depuis la variable globale
        let mongo_uri = MONGO_URI.lock().unwrap().clone();
        log_message(&log_state, format!("Initializing MongoDB connection with URI: {}", mongo_uri.split('@').next().unwrap_or(&mongo_uri)));

        let client_options = match ClientOptions::parse(&mongo_uri).await {
            Ok(options) => options,
            Err(e) => {
                log_message(&log_state, format!("Failed to parse MongoDB URI: {}", e));
                // Continuer avec l'URI par défaut si l'URI configurée est invalide
                let default_uri = DEFAULT_MONGO_URI.to_string();
                log_message(&log_state, format!("Falling back to default URI: {}", default_uri));

                ClientOptions::parse(DEFAULT_MONGO_URI)
                    .await
                    .expect("Failed to parse default MongoDB URI")
            }
        };

        let client = match Client::with_options(client_options) {
            Ok(client) => client,
            Err(e) => {
                log_message(&log_state, format!("Failed to create MongoDB client: {}", e));
                // Au lieu de planter, on crée un client avec une URI par défaut
                // qui sera remplacée plus tard par la configuration utilisateur
                log_message(&log_state, "Creating placeholder MongoDB client - connection will be established later".to_string());
                Client::with_uri_str(DEFAULT_MONGO_URI)
                    .await
                    .expect("Failed to create placeholder MongoDB client")
            }
        };

        // Essayer de ping MongoDB, mais ne pas planter si ça échoue
        match client.database("admin").run_command(doc! {"ping": 1}, None).await {
            Ok(_) => log_message(&log_state, "Connected to MongoDB successfully during init".to_string()),
            Err(e) => {
                log_message(&log_state, format!("Failed to ping MongoDB: {}", e));
                log_message(&log_state, "Application will start and prompt for MongoDB configuration".to_string());
                // Ne pas panic! ici - on laisse l'interface s'afficher
            }
        }

        let db = client.database(DB_NAME);
        let cloned_log_file = log_state.0.lock().unwrap().try_clone().expect("Failed to clone log file handle during init");
        let repo_log_state = LogState(Mutex::new(cloned_log_file));

        // Création d'un repository MongoDB même si la connexion a échoué
        // Les fonctions individuelles géreront les erreurs de connexion quand elles seront appelées
        let db_repo: Arc<dyn DbRepository> = Arc::new(MongoDbRepository { db, log_state: repo_log_state });

        AppState { db_repo, log_state }
    });


    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            init_command,
            save_package_collection,
            save_stage_to_mongodb,
            get_package_collections_by_uri,
            get_current_username,
            get_all_package_collections,
            get_package_collection_tools,
            get_stages_by_uri,
            revert_stage,
            get_stage_history,
            get_all_stage_names,
            open_tool_in_terminal,
            open_rez_env_in_terminal,
            test_mongodb_connection,
            load_stage_by_id
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use super::MockDbRepository;
    use rand::{distributions::Alphanumeric, Rng};
    use std::fs;
    use std::path::PathBuf;
    use std::collections::HashSet;

    fn generate_random_suffix(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    fn create_test_log_state() -> (LogState, PathBuf) {
        let unique_suffix = generate_random_suffix(8);
        let log_file_name = format!("test_log_{}.log", unique_suffix);
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join(&log_file_name);
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open test log file");
        (LogState(Mutex::new(log_file)), log_path)
    }

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

    #[tokio::test]
    async fn test_get_package_collections_by_uri_found() {
        let uri1 = "test/uri/1";
        let pkg1 = create_dummy_package_collection("1.0", uri1);
        let pkg2 = create_dummy_package_collection("2.0", uri1);
        let expected_packages = vec![pkg1.clone(), pkg2.clone()];

        let mut mock_repo = MockDbRepository::new();
        mock_repo.expect_find_package_collections_by_uri()
            .with(eq(uri1))
            .times(1)
            .returning(move |_| Ok(expected_packages.clone()));

        let (log_state, _log_path) = create_test_log_state();
        let app_state = AppState {
            db_repo: Arc::new(mock_repo),
            log_state,
        };

        let result = app_state.db_repo.find_package_collections_by_uri(uri1).await;

        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&pkg1));
        assert!(collections.contains(&pkg2));

        let _ = fs::remove_file(_log_path);
    }

    #[tokio::test]
    async fn test_get_package_collections_by_uri_not_found() {
        let non_existent_uri = "test/uri/non_existent";
        let expected_packages: Vec<PackageCollection> = vec![];

        let mut mock_repo = MockDbRepository::new();
        mock_repo.expect_find_package_collections_by_uri()
            .with(eq(non_existent_uri))
            .times(1)
            .returning(move |_| Ok(expected_packages.clone()));

        let (log_state, _log_path) = create_test_log_state();
        let app_state = AppState {
            db_repo: Arc::new(mock_repo),
            log_state,
        };

        let result = app_state.db_repo.find_package_collections_by_uri(non_existent_uri).await;

        assert!(result.is_ok());
        let collections = result.unwrap();
        assert!(collections.is_empty());

        let _ = fs::remove_file(_log_path);
    }

     #[tokio::test]
    async fn test_get_package_collections_by_uri_repo_error() {
         let uri = "test/uri/error";
         let error_message = "Database connection failed".to_string();

         let mut mock_repo = MockDbRepository::new();
         mock_repo.expect_find_package_collections_by_uri()
             .with(eq(uri))
             .times(1)
             .returning(move |_| Err(error_message.clone()));

         let (log_state, _log_path) = create_test_log_state();
         let app_state = AppState {
             db_repo: Arc::new(mock_repo),
             log_state,
         };

         let result = app_state.db_repo.find_package_collections_by_uri(uri).await;

         assert!(result.is_err());
         assert_eq!(result.err().unwrap(), "Database connection failed");

         let _ = fs::remove_file(_log_path);
     }


    #[tokio::test]
    async fn test_get_all_package_collections_found() {
        let uri1 = "test/uri/1";
        let uri2 = "test/uri/2";
        let pkg1 = create_dummy_package_collection("1.0", uri1);
        let pkg2 = create_dummy_package_collection("2.0", uri1);
        let pkg3 = create_dummy_package_collection("1.0", uri2);
        let expected_packages = vec![pkg1.clone(), pkg2.clone(), pkg3.clone()];

        let mut mock_repo = MockDbRepository::new();
        mock_repo.expect_find_all_package_collections()
            .times(1)
            .returning(move || Ok(expected_packages.clone()));

        let (log_state, _log_path) = create_test_log_state();
        let app_state = AppState {
            db_repo: Arc::new(mock_repo),
            log_state,
        };

        let result = app_state.db_repo.find_all_package_collections().await;

        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 3);
        assert!(collections.contains(&pkg1));
        assert!(collections.contains(&pkg2));
        assert!(collections.contains(&pkg3));

        let _ = fs::remove_file(_log_path);
    }

    #[tokio::test]
    async fn test_get_all_package_collections_empty_db() {
         let expected_packages: Vec<PackageCollection> = vec![];

         let mut mock_repo = MockDbRepository::new();
         mock_repo.expect_find_all_package_collections()
             .times(1)
             .returning(move || Ok(expected_packages.clone()));

         let (log_state, _log_path) = create_test_log_state();
         let app_state = AppState {
             db_repo: Arc::new(mock_repo),
             log_state,
         };

         let result = app_state.db_repo.find_all_package_collections().await;

         assert!(result.is_ok());
         let collections = result.unwrap();
         assert!(collections.is_empty());

         let _ = fs::remove_file(_log_path);
    }

    #[tokio::test]
    async fn test_logging_writes_to_file_via_repo_call() {
        let uri = "test/uri/log";
        let pkg_to_save = create_dummy_package_collection("log_pkg", uri);

        let (log_state, log_path) = create_test_log_state();
        let log_path_clone = log_path.clone();

        let mut mock_repo = MockDbRepository::new();
        mock_repo.expect_insert_package_collection()
            .with(eq(pkg_to_save.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let app_state = AppState {
            db_repo: Arc::new(mock_repo),
            log_state,
        };

        let result = app_state.db_repo.insert_package_collection(pkg_to_save).await;
        assert!(result.is_ok());

        let _ = fs::remove_file(log_path_clone);
    }


    #[tokio::test]
    async fn test_get_all_stage_names_found() {
        let expected_names = vec!["StageA".to_string(), "StageB".to_string(), "StageC".to_string()];
        let expected_names_clone = expected_names.clone();

        let mut mock_repo = MockDbRepository::new();
        mock_repo.expect_find_distinct_stage_names()
            .times(1)
            .returning(move || Ok(expected_names_clone.clone()));

        let (log_state, _log_path) = create_test_log_state();
        let app_state = AppState {
            db_repo: Arc::new(mock_repo),
            log_state,
        };

        let result = app_state.db_repo.find_distinct_stage_names().await;

        assert!(result.is_ok(), "find_distinct_stage_names failed: {:?}", result.err());
        let names = result.unwrap();

        let expected_set: HashSet<String> = expected_names.into_iter().collect();
        let actual_set: HashSet<String> = names.into_iter().collect();

        assert_eq!(actual_set.len(), 3, "Incorrect number of unique names returned");
        assert_eq!(actual_set, expected_set, "Returned names do not match expected unique names");

        let _ = fs::remove_file(_log_path);
    }

     #[tokio::test]
    async fn test_get_all_stage_names_empty_db() {
        let expected_names: Vec<String> = vec![];
        let expected_names_clone = expected_names.clone();

        let mut mock_repo = MockDbRepository::new();
        mock_repo.expect_find_distinct_stage_names()
            .times(1)
            .returning(move || Ok(expected_names_clone.clone()));

        let (log_state, _log_path) = create_test_log_state();
        let app_state = AppState {
            db_repo: Arc::new(mock_repo),
            log_state,
        };

        let result = app_state.db_repo.find_distinct_stage_names().await;

        assert!(result.is_ok(), "find_distinct_stage_names failed: {:?}", result.err());
        let names = result.unwrap();
        assert!(names.is_empty(), "Expected empty vector for empty database, got: {:?}", names);

        let _ = fs::remove_file(_log_path);
    }
}
