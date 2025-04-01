use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use reqwest::Client;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

struct UserStore(Mutex<HashMap<String, String>>);

#[derive(Serialize, Deserialize, Default, Clone)]
struct Settings {
    user_id: Option<String>,
    // RPC visibility settings
    show_anime_title: bool,
    show_episode_title: bool,
    show_episode_number: bool,
    show_progress: bool,
    show_timestamp: bool,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WatchingData {
    id: String,
    title: String,
    animeId: String,
    episodeId: String,
    episodeNumber: i32,
    episodeTitle: String,
    image: String,
    progress: f64,
    duration: f64,
    timestamp: String,
    updatedAt: String,
    userId: String,
    startTimestamp: u64,
    endTimestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    watching: Option<WatchingData>,
}

struct HttpState {
    client: Client,
    last_poll: Mutex<Instant>,
}

struct WatchingState(Mutex<Option<WatchingData>>);

struct DiscordRpcState {
    client: Mutex<Option<DiscordIpcClient>>,
    connected: Mutex<bool>,
}

const USER_ID_KEY: &str = "userId";
const SETTINGS_FILE: &str = "settings.json";
const API_BASE_URL: &str = "https://o.jwd.gg/api/users";
const POLL_INTERVAL: Duration = Duration::from_secs(15);
const DISCORD_CLIENT_ID: &str = "1213360640580653166";
const WATCH_URL_BASE: &str = "https://o.jwd.gg/watch";

fn get_settings_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // For development, print the path to help with debugging
    println!("Settings path: {}", app_dir.display());

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    }

    let settings_path = app_dir.join(SETTINGS_FILE);
    println!("Full settings file path: {}", settings_path.display());

    Ok(settings_path)
}

fn load_settings<R: Runtime>(app: &AppHandle<R>) -> Result<Settings, String> {
    let path = get_settings_path(app)?;

    if !path.exists() {
        return Ok(Settings {
            user_id: None,
            show_anime_title: true,
            show_episode_title: true,
            show_episode_number: true,
            show_progress: true,
            show_timestamp: true,
            enabled: true,
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    match serde_json::from_str::<Settings>(&content) {
        Ok(settings) => Ok(settings),
        Err(_) => {
            #[derive(Deserialize)]
            struct LegacySettings {
                user_id: Option<String>,
            }

            match serde_json::from_str::<LegacySettings>(&content) {
                Ok(legacy) => {
                    let settings = Settings {
                        user_id: legacy.user_id,
                        show_anime_title: true,
                        show_episode_title: true,
                        show_episode_number: true,
                        show_progress: true,
                        show_timestamp: true,
                        enabled: true,
                    };

                    if let Err(e) = save_settings(app, &settings) {
                        eprintln!("Failed to save updated settings: {}", e);
                    }

                    Ok(settings)
                },
                Err(e) => Err(format!("Failed to parse settings file: {}", e)),
            }
        }
    }
}

fn save_settings<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<(), String> {
    let path = get_settings_path(app)?;

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))
}

fn delete_settings_file<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let path = get_settings_path(app)?;

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete settings file: {}", e))?;
        println!("Settings file deleted: {}", path.display());
    }

    Ok(())
}

fn init_discord_rpc(discord_state: &DiscordRpcState) -> Result<(), String> {
    let mut discord_locked = discord_state.client.lock().map_err(|_| "Failed to acquire lock on Discord client".to_string())?;
    let mut connected_locked = discord_state.connected.lock().map_err(|_| "Failed to acquire lock on Discord connected flag".to_string())?;

    if discord_locked.is_none() {
        match DiscordIpcClient::new(DISCORD_CLIENT_ID) {
            Ok(client) => {
                *discord_locked = Some(client);
            },
            Err(e) => {
                return Err(format!("Failed to create Discord RPC client: {:?}", e));
            }
        }
    }

    if !*connected_locked {
        if let Some(client) = discord_locked.as_mut() {
            match client.connect() {
                Ok(_) => {
                    *connected_locked = true;
                    println!("Successfully connected to Discord RPC");
                },
                Err(e) => {
                    println!("Failed to connect to Discord RPC: {:?}", e);
                    return Err(format!("Failed to connect to Discord RPC: {:?}", e));
                }
            }
        }
    }

    Ok(())
}

fn clear_discord_rpc(discord_state: &DiscordRpcState) -> Result<(), String> {
    let mut discord_locked = discord_state.client.lock().map_err(|_| "Failed to acquire lock on Discord client".to_string())?;
    let connected_locked = discord_state.connected.lock().map_err(|_| "Failed to acquire lock on Discord connected flag".to_string())?;

    if *connected_locked {
        if let Some(client) = discord_locked.as_mut() {
            match client.clear_activity() {
                Ok(_) => println!("Cleared Discord activity"),
                Err(e) => println!("Failed to clear Discord activity: {:?}", e)
            }
        }
    }

    Ok(())
}

fn close_discord_rpc(discord_state: &DiscordRpcState) -> Result<(), String> {
    let mut discord_locked = discord_state.client.lock().map_err(|_| "Failed to acquire lock on Discord client".to_string())?;
    let mut connected_locked = discord_state.connected.lock().map_err(|_| "Failed to acquire lock on Discord connected flag".to_string())?;

    if *connected_locked {
        if let Some(client) = discord_locked.as_mut() {
            match client.close() {
                Ok(_) => {
                    *connected_locked = false;
                    println!("Closed Discord RPC connection");
                },
                Err(e) => println!("Failed to close Discord RPC connection: {:?}", e)
            }
        }
    }

    Ok(())
}

#[tauri::command]
async fn save_user_id<R: Runtime>(
    app: AppHandle<R>,
    store: State<'_, UserStore>,
    id: String
) -> Result<(), String> {
    let mut settings = load_settings(&app)?;
    settings.user_id = Some(id.clone());
    save_settings(&app, &settings)?;

    {
        let mut store = store.0.lock().map_err(|_| "Failed to acquire lock on store".to_string())?;
        store.insert(USER_ID_KEY.to_string(), id);
    }

    refresh_watching_data(&app).await?;

    Ok(())
}

#[tauri::command]
async fn get_user_id<R: Runtime>(
    app: AppHandle<R>,
    store: State<'_, UserStore>
) -> Result<Option<String>, String> {
    let settings = load_settings(&app)?;

    if let Some(id) = settings.user_id {
        {
            let mut store = store.0.lock().map_err(|_| "Failed to acquire lock on store".to_string())?;
            store.insert(USER_ID_KEY.to_string(), id.clone());
        }
        return Ok(Some(id));
    }

    let store = store.0.lock().map_err(|_| "Failed to acquire lock on store".to_string())?;
    Ok(store.get(USER_ID_KEY).cloned())
}

#[tauri::command]
async fn get_settings<R: Runtime>(
    app: AppHandle<R>
) -> Result<Settings, String> {
    load_settings(&app)
}

#[tauri::command]
async fn save_settings_command<R: Runtime>(
    app: AppHandle<R>,
    settings: Settings
) -> Result<(), String> {
    save_settings(&app, &settings)?;
    update_rpc(&app).await?;
    Ok(())
}

#[tauri::command]
async fn get_watching_data(
    state: State<'_, WatchingState>
) -> Result<Option<WatchingData>, String> {
    let watching = state.0.lock().map_err(|_| "Failed to acquire lock on watching state".to_string())?;
    Ok(watching.clone())
}

async fn fetch_watching_data(client: &Client, user_id: &str) -> Result<Option<WatchingData>, String> {
    let url = format!("{}/{}/watching", API_BASE_URL, user_id);

    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch watching data: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    let api_response: ApiResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    Ok(api_response.watching)
}

async fn refresh_watching_data<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let settings = load_settings(app)?;

    let user_id = match settings.user_id {
        Some(id) => id,
        None => return Ok(()), // No user ID set, nothing to do
    };

    let http_state = app.state::<HttpState>();

    {
        let mut last_poll = http_state.last_poll.lock().map_err(|_| "Failed to acquire lock on last_poll".to_string())?;
        *last_poll = Instant::now();
    }

    let watching = fetch_watching_data(&http_state.client, &user_id).await?;
    let watching_state = app.state::<WatchingState>();
    let watching_clone = watching.clone();

    {
        let mut watching_lock = watching_state.0.lock().map_err(|_| "Failed to acquire lock on watching state".to_string())?;
        *watching_lock = watching;
    }

    app.emit("watching-updated", watching_clone)
        .map_err(|e| format!("Failed to emit watching-updated event: {}", e))?;

    update_rpc(app).await?;

    Ok(())
}

async fn update_rpc<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let settings = load_settings(app)?;
    let discord_state = app.state::<DiscordRpcState>();

    if !settings.enabled {
        clear_discord_rpc(&discord_state)?;
        return Ok(());
    }

    let watching_option = {
        let watching_state = app.state::<WatchingState>();
        let watching_lock = watching_state.0.lock().map_err(|_| "Failed to acquire lock on watching state".to_string())?;
        watching_lock.clone()
    };

    init_discord_rpc(&discord_state)?;

    if let Some(watching) = watching_option {
        let mut details = String::new();
        let mut state = String::new();

        if settings.show_anime_title {
            details = watching.title.clone();
        }

        if settings.show_episode_number && settings.show_episode_title {
            state = format!("Episode {}: {}", watching.episodeNumber, watching.episodeTitle);
        } else if settings.show_episode_number {
            state = format!("Episode {}", watching.episodeNumber);
        } else if settings.show_episode_title {
            state = watching.episodeTitle.clone();
        }

        let mut activity = activity::Activity::new()
            .activity_type(activity::ActivityType::Watching);

        if !details.is_empty() {
            activity = activity.details(&details);
        }

        if !state.is_empty() {
            activity = activity.state(&state);
        }

        if settings.show_timestamp {
            let timestamps = activity::Timestamps::new()
                .start(watching.startTimestamp as i64)
                .end(watching.endTimestamp as i64);
            activity = activity.timestamps(timestamps);
        }

        let assets = activity::Assets::new()
            .large_image(&watching.image)
            .large_text(&watching.title);
        activity = activity.assets(assets);

        let watch_url = format!("{}/{}/{}", WATCH_URL_BASE, watching.animeId, watching.episodeId);
        let buttons = vec![
            activity::Button::new("Watch Now", &watch_url)
        ];
        activity = activity.buttons(buttons);

        let mut discord_locked = discord_state.client.lock().map_err(|_| "Failed to acquire lock on Discord client".to_string())?;
        if let Some(client) = discord_locked.as_mut() {
            match client.set_activity(activity) {
                Ok(_) => {
                    println!("Discord RPC Update:");
                    println!("  Details: {}", details);
                    println!("  State: {}", state);
                    println!("  Start: {}", watching.startTimestamp);
                    println!("  End: {}", watching.endTimestamp);
                    println!("  Image: {}", watching.image);
                    println!("  Watch URL: {}/{}/{}", WATCH_URL_BASE, watching.animeId, watching.episodeId);
                },
                Err(e) => {
                    println!("Failed to update Discord RPC: {:?}", e);
                    if let Err(e) = client.reconnect() {
                        println!("Failed to reconnect to Discord RPC: {:?}", e);
                    }
                }
            }
        }
    } else {
        clear_discord_rpc(&discord_state)?;
    }

    Ok(())
}

async fn poll_api<R: Runtime>(app: AppHandle<R>) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let should_poll = {
            let http_state = app.state::<HttpState>();
            let last_poll_guard = http_state.last_poll.lock();

            if let Ok(last_poll) = last_poll_guard {
                last_poll.elapsed() >= POLL_INTERVAL
            } else {
                true // If we can't get the lock, assume we should poll
            }
        };

        if should_poll {
            if let Err(e) = refresh_watching_data(&app).await {
                eprintln!("Failed to refresh watching data: {}", e);
            }
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();

    let store = UserStore(Mutex::new(HashMap::new()));
    let watching_state = WatchingState(Mutex::new(None));
    let http_state = HttpState {
        client: Client::new(),
        last_poll: Mutex::new(Instant::now() - POLL_INTERVAL),
    };
    let discord_state = DiscordRpcState {
        client: Mutex::new(None),
        connected: Mutex::new(false),
    };

    tauri::Builder::default()
        .manage(store)
        .manage(watching_state)
        .manage(http_state)
        .manage(discord_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            save_user_id,
            get_user_id,
            get_settings,
            save_settings_command,
            get_watching_data,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                poll_api(app_handle).await;
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let app_handle = window.app_handle();
                let discord_state = app_handle.state::<DiscordRpcState>();
                if let Err(e) = close_discord_rpc(&discord_state) {
                    eprintln!("Error closing Discord RPC: {}", e);
                }
            }
        })
        .run(context)
        .expect("error while running tauri application");
}
