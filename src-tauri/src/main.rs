// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessageContentPartImageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};

use futures::StreamExt;
//use base64::Engine;
use base64::{engine::general_purpose, Engine as _};
use screenshots::Screen;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::{env, path::PathBuf};
use std::{error::Error, fs, path::Path};
use tauri::{api::path::picture_dir, AppHandle, Manager, Window, Wry};
use tauri_plugin_store::with_store;
use tauri_plugin_store::StoreCollection;
use tokio::task;

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct Payload {
    data: String,
}

use image::{DynamicImage, ImageOutputFormat};
use std::io::Cursor;

fn image_to_base64(img: &DynamicImage) -> String {
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::Png)
        .unwrap();
    //let res_base64 = Engine::encode(image_data);
    let res_base64 = general_purpose::STANDARD.encode(image_data);
    // let res_base64 = base64::encode(image_data);
    format!("data:image/png;base64,{}", res_base64)
}

async fn call_chatgpt_image(
    prompt: String,
    filename: &str,
    app_handle: AppHandle,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    // Om det finns en tidigare prompt flyttar vi den till CHATGPT_PREVIOUS_PROMPT
    let last_prompt = get_latest_prompt().unwrap();
    if last_prompt.len() > 0 {
        let key = "CHATGPT_PREVIOUS_PROMPT";
        env::set_var(key, last_prompt.clone());
    }

    // Sparar nya prompten i CHATGPT_PROMPT
    let key = "CHATGPT_PROMPT";
    env::set_var(key, prompt.clone());

    // Om det finns ett tidigare svar från ChatGPT,
    // flyttar vi det till CHATGPT_PREVIOUS_RESPONSE
    //let mut last_response = "".to_string();
    match get_latest_response() {
        Ok(response) => {
            let key = "CHATGPT_PREVIOUS_RESPONSE";
            env::set_var(key, response.clone());
            //last_response = response;
        }
        Err(_e) => {}
    }

    let picture_dir = picture_dir().unwrap();
    let full_path_to_screenshot_dir = picture_dir.join(Path::new("gpt"));
    //let filename = "screenshot_20231125_212445.png";
    let full_path_to_screenshot = full_path_to_screenshot_dir.join(Path::new(&filename));

    match image::open(&full_path_to_screenshot) {
        Err(_e) => println!("{}", _e),
        Ok(_v) => {
            //println!("Funkar: {}", full_path_to_screenshot.to_str().unwrap());
            let img = image::open(full_path_to_screenshot).unwrap();

            let request = CreateChatCompletionRequestArgs::default()
                .max_tokens(512u16)
                //        .model("gpt-3.5-turbo")
                .model("gpt-4-vision-preview")
                .messages([
                    // ChatCompletionRequestUserMessageArgs::default()
                    //     .content(last_prompt)
                    //     .build()?
                    //     .into(),
                    // ChatCompletionRequestAssistantMessageArgs::default()
                    //     .content(last_response)
                    //     .build()?
                    //     .into(),
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(prompt)
                        .build()?
                        .into(),
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(
                            // vec!(
                            // ChatCompletionRequestMessageContentPartImageArgs::default()
                            // .image_url("https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg")
                            // .build()?
                            // .into()
                            // )
                            vec![ChatCompletionRequestMessageContentPartImageArgs::default()
                                .image_url(image_to_base64(&img))
                                .build()?
                                .into()],
                        )
                        .build()?
                        .into(),
                ])
                .build()?;

            let mut stream = client.chat().create_stream(request).await?;

            app_handle
                .emit_all(
                    "hide_spinner_in_frontend",
                    Payload {
                        data: "".to_string(),
                    },
                )
                .unwrap();

            let mut complete_response = "".to_string();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        response.choices.iter().for_each(|chat_choice| {
                            if let Some(ref content) = chat_choice.delta.content {
                                complete_response.push_str(content.as_str());
                                app_handle
                                    .emit_all(
                                        "stream_response_in_frontend",
                                        Payload {
                                            data: content.to_string(),
                                        },
                                    )
                                    .unwrap();

                                //write!(lock, "{}", content).unwrap();
                            }
                        });
                    }
                    Err(_err) => {
                        //writeln!(lock, "error: {err}").unwrap();
                    }
                }
            }

            app_handle
                .emit_all(
                    "format_response_in_frontend",
                    Payload {
                        data: "".to_string(),
                    },
                )
                .unwrap();

            // Sparar nya svaret i CHATGPT_RESPONSE
            let key = "CHATGPT_RESPONSE";
            env::set_var(key, complete_response);

            // let response = client.chat().create(request).await?;

            // // Sparar nya svaret i CHATGPT_RESPONSE
            // let key = "CHATGPT_RESPONSE";
            // for choice in response.choices {
            //     env::set_var(key, choice.message.content.clone().unwrap());
            // }
        }
    }
    Ok(())
}

async fn call_chatgpt_chat(prompt: String, app_handle: AppHandle) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    // Om det finns en tidigare prompt flyttar vi den till CHATGPT_PREVIOUS_PROMPT

    let last_prompt = get_latest_prompt().unwrap();
    if last_prompt.len() > 0 {
        let key = "CHATGPT_PREVIOUS_PROMPT";
        env::set_var(key, last_prompt.clone());
    }

    // Sparar nya prompten i CHATGPT_PROMPT
    let key = "CHATGPT_PROMPT";
    env::set_var(key, prompt.clone());

    // Om det finns ett tidigare svar från ChatGPT,
    // flyttar vi det till CHATGPT_PREVIOUS_RESPONSE
    let mut last_response = "".to_string();
    match get_latest_response() {
        Ok(response) => {
            let key = "CHATGPT_PREVIOUS_RESPONSE";
            env::set_var(key, response.clone());
            last_response = response;
        }
        Err(_e) => {}
    }

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        //        .model("gpt-3.5-turbo")
        .model("gpt-4-1106-preview")
        .messages([
            //            ChatCompletionRequestSystemMessageArgs::default()
            //                .content("You are a helpful assistant.")
            //                .build()?
            //                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(last_prompt)
                .build()?
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(last_response)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    app_handle
        .emit_all(
            "hide_spinner_in_frontend",
            Payload {
                data: "".to_string(),
            },
        )
        .unwrap();

    let mut complete_response = "".to_string();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        complete_response.push_str(content.as_str());
                        app_handle
                            .emit_all(
                                "stream_response_in_frontend",
                                Payload {
                                    data: content.to_string(),
                                },
                            )
                            .unwrap();

                        //write!(lock, "{}", content).unwrap();
                    }
                });
            }
            Err(_err) => {
                //writeln!(lock, "error: {err}").unwrap();
            }
        }
    }

    app_handle
        .emit_all(
            "format_response_in_frontend",
            Payload {
                data: "".to_string(),
            },
        )
        .unwrap();

    // Sparar nya svaret i CHATGPT_RESPONSE
    let key = "CHATGPT_RESPONSE";
    env::set_var(key, complete_response);

    // let response = client.chat().create(request).await?;

    // // Sparar nya svaret i CHATGPT_RESPONSE
    // let key = "CHATGPT_RESPONSE";
    // for choice in response.choices {
    //     env::set_var(key, choice.message.content.clone().unwrap());

    // }

    Ok(())
}

#[tauri::command]
async fn call_chatgpt(prompt: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let key = "OPENAI_API_KEY";

    match env::var(key) {
        Ok(_val) => {
            let store_latest_screenshot =
                get_key_from_store("latest-screenshot".to_string(), app_handle.clone());

            // Kollar om vi har ett screenshot sparat i vår Store.
            // Om vi har det så skickar vi screenshot samt prompt till chatgpt vision
            // annars skickar vi bara prompten
            if store_latest_screenshot.len() > 0 {
                // Vi har ett screenshot som ska med
                let mut log_text = store_latest_screenshot.to_owned();
                log_text.push_str(": Prompt: ");
                log_text.push_str(prompt.clone().as_str());
                write_to_log_file(log_text.to_string(), app_handle.clone());

                match call_chatgpt_image(
                    prompt,
                    store_latest_screenshot.as_str(),
                    app_handle.clone(),
                )
                .await
                {
                    Ok(()) => {
                        write_to_log_file(
                            "Response: ".to_string() + get_latest_response().unwrap().as_str(),
                            app_handle.clone(),
                        );

                        /*
                           Nollar värdet av senaste screenshotet i vår store
                        */
                        match save_to_store(
                            "latest-screenshot".to_string(),
                            "".to_string(),
                            app_handle.clone(),
                        ) {
                            Ok(_v) => {}
                            Err(_e) => {}
                        }
                        app_handle
                            .emit_all(
                                "show_response_in_frontend",
                                Payload {
                                    data: get_latest_response().unwrap().as_str().into(),
                                },
                            )
                            .unwrap();
                        return Ok("success".to_string());
                    }
                    Err(_e) => {
                        println!("{:?}", _e);
                        Err(_e.to_string())
                    }
                }
            } else {
                // Vi har inget screenshot som ska med
                // Skickar bara prompt
                write_to_log_file(
                    "Prompt: ".to_string() + prompt.clone().as_str(),
                    app_handle.clone(),
                );

                match call_chatgpt_chat(prompt, app_handle.clone()).await {
                    Ok(()) => {
                        write_to_log_file(
                            "Response: ".to_string() + get_latest_response().unwrap().as_str(),
                            app_handle.clone(),
                        );

                        app_handle
                            .emit_all(
                                "show_response_in_frontend",
                                Payload {
                                    data: get_latest_response().unwrap().as_str().into(),
                                },
                            )
                            .unwrap();

                        return Ok("success".to_string());
                    }
                    Err(e) => {
                        println!("ERROR: call_chatgpt(): {}", e);

                        Err(e.to_string())
                    }
                }
            }
        }
        Err(_e) => {
            return Err("Error: Can't call ChatGPT: No api-key".to_string());
        }
    }
}

#[tauri::command]
fn show_screenshot_window(app: tauri::AppHandle) {
    let docs_window = tauri::WindowBuilder::new(
        &app,
        "screenshot",
        tauri::WindowUrl::App("screenshot_window.html".into()),
    )
    .transparent(true)
    .build()
    .expect("Failed to build window");

    let _ = docs_window.set_fullscreen(true);
}

fn get_latest_response() -> Result<String, &'static str> {
    let key = "CHATGPT_RESPONSE";
    match env::var(key) {
        Ok(val) => {
            return Ok(val);
        }
        Err(_e) => {
            return Err("error");
        }
    }
}

fn get_latest_prompt() -> Result<String, &'static str> {
    let key = "CHATGPT_PROMPT";
    match env::var(key) {
        Ok(val) => {
            return Ok(val);
        }
        Err(_e) => {
            return Err("error");
        }
    }
}

#[tauri::command]
fn send_response_to_frontend(window: Window) -> Result<String, &'static str> {
    //  println!("send_response_to_frontend in backend called");
    match get_latest_response() {
        Ok(val) => {
            //println!("Last response: {:?}", val.clone());
            window
                .emit(
                    "show_response_in_frontend",
                    Payload {
                        data: val.clone().into(),
                    },
                )
                .unwrap();
            return Ok(val);
        }
        Err(_e) => {
            //println!("Couldn't get last response: {_e}");
            return Err("Couldn't get last response");
        }
    }
}

#[tauri::command]
fn send_prompt_to_frontend(window: Window) -> Result<String, &'static str> {
    //println!("send_prompt_to_frontend in backend called");
    match get_latest_prompt() {
        Ok(val) => {
            //println!("Last prompt: {:?}", val.clone());
            window
                .emit(
                    "show_prompt_in_frontend",
                    Payload {
                        data: val.clone().into(),
                    },
                )
                .unwrap();
            return Ok(val);
        }
        Err(_e) => {
            //println!("Couldn't get last prompt: {_e}");
            return Err("Couldn't get last prompt");
        }
    }
}

#[tauri::command]
fn save_to_store(
    key: String,
    key_value: String,
    app_handle: tauri::AppHandle,
) -> Result<String, &'static str> {
    let stores = app_handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from(".settings.dat");
    let store_result = with_store(app_handle.clone(), stores, path.clone(), |store| {
        store.insert(key.to_string(), json!(key_value))?;
        store.save()
    });

    if key == "api-key" {
        match store_result {
            Ok(()) => {
                let key = "OPENAI_API_KEY";
                env::set_var(key, key_value);
                Ok("success".to_string())
            }
            Err(_) => Err("Något error"),
        }
    } else {
        Ok("success".to_string())
    }
}

#[tauri::command]
fn get_key_from_store(key: String, app_handle: tauri::AppHandle) -> String {
    let stores = app_handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from(".settings.dat");

    let store_result = with_store(app_handle.clone(), stores, path.clone(), |store| {
        //store.insert("d".to_string(), json!("c")); store.save()  // <-- WORKS!
        store
            .get(key.clone())
            .map(|json| json!(json.as_str()))
            /* .and_then(|json| Some(json!(json)))*/
            .ok_or(tauri_plugin_store::Error::NotFound(path.clone()))
    });

    match store_result {
        Ok(value) => {
            let result: String = serde_json::from_value(value).unwrap();
            //println!("INFO: Successfully retrieved {} from store", key);
            result
        }
        Err(_) => "".to_string(),
    }
}

#[tauri::command]
fn take_screenshot(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    app_handle: tauri::AppHandle,
    window: Window,
) -> String {
    let x1: i32;
    let width: i32;
    let y1: i32;
    let height: i32;
    if start_x < end_x {
        x1 = start_x + 1;
        width = end_x - start_x - 2;
    } else {
        x1 = end_x + 1;
        width = start_x - end_x - 2;
    }

    if start_y < end_y {
        y1 = start_y + 1;
        height = end_y - start_y - 2;
    } else {
        y1 = end_y + 1;
        height = start_y - end_y - 2;
    }

    let current_monitor_x = window.current_monitor().unwrap().unwrap().position().x;
    let current_monitor_y = window.current_monitor().unwrap().unwrap().position().y;
    // let screens = Screen::all().unwrap();
    // println!("{:?}", screens);

    let screen = Screen::from_point(current_monitor_x, current_monitor_y).unwrap();
    // println!(
    //     "Capture screen: Start: {}/{}, Size: {}/{}",
    //     x1, y1, width, height
    // );
    // println!("Capture screen: {screen:?}");

    let image = screen
        .capture_area(
            x1,
            y1,
            width.try_into().unwrap(),
            height.try_into().unwrap(),
        )
        .unwrap();

    let picture_dir = picture_dir().unwrap();
    let full_path_to_screenshot_dir = picture_dir.join(Path::new("gpt"));
    //println!("{:?}", full_path_to_screenshot_dir.canonicalize());
    let now = chrono::offset::Local::now();
    let custom_datetime_format = now.format("%Y%m%d_%H%M%S").to_string();
    //println!("{}", custom_datetime_format);

    let filename = "screenshot_".to_string() + custom_datetime_format.as_str() + ".png";
    //println!("{}", filename);

    //println!("{:?}", picture_dir.canonicalize());
    let full_path_to_screenshot = full_path_to_screenshot_dir.join(Path::new(&filename));
    //println!("a: {:?}", full_path_to_screenshot.to_str().unwrap());
    match image.save(full_path_to_screenshot) {
        Ok(_v) => {
            // Sparar namnet på senaste screenshotet i vår store.
            // Ska visas för användaren
            match save_to_store(
                "latest-screenshot".to_string(),
                filename.clone(),
                app_handle.clone(),
            ) {
                Ok(_v) => {
                    //println!("Skickar event till frontend");
                    app_handle
                        .get_window("main")
                        .unwrap()
                        .emit("show-latest-screenshot", Payload { data: filename })
                        .unwrap();
                }
                Err(_e) => {}
            }
        }
        Err(_e) => {}
    }

    "".to_string()
}

fn write_to_log_file(content: String, app_handle: tauri::AppHandle) {
    let app_data_path = app_handle.path_resolver().app_data_dir().unwrap();

    let full_path = app_data_path.join(Path::new("gpt-log.txt"));
    if full_path.exists() {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(full_path)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", content) {
            eprintln!("Error: Couldn't write to file: {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    /*
     let quit = CustomMenuItem::new("quit".to_string(), "Quit");
     let toggle = CustomMenuItem::new("toggle".to_string(), "Hide");
     let tray_menu = SystemTrayMenu::new()
         .add_item(quit)
         .add_native_item(SystemTrayMenuItem::Separator)
         .add_item(toggle);
     let system_tray = SystemTray::new().with_menu(tray_menu);
    */
    tauri::Builder::default()
        .setup(|app| {
            //app.get_window("main").unwrap().open_devtools(); // `main` is the first window from tauri.conf.json without an explicit label

            
            let app_handle_clone: tauri::AppHandle = app.handle();

            let _ = app.listen_global("hidemainwindow", move |_event| {
                //println!("got hidemainwindow with payload: {:?}", _event.payload());

                let _ = app_handle_clone.get_window("main").unwrap().hide();
            });

            let app_handle_clone: tauri::AppHandle = app.handle();
            let _ = app.listen_global("showmainwindow", move |_event| {
                //println!("got showmainwindow with payload: {:?}", _event.payload());

                let _ = app_handle_clone.get_window("main").unwrap().show();
            });

            let app_handle_clone: tauri::AppHandle = app.handle();
            let _ = app.listen_global("close_screenshot_window", move |_event| {
                //println!("got showmainwindow with payload: {:?}", _event.payload());

                let _ = app_handle_clone.get_window("screenshot").unwrap().close();
            });

            /*
               Nollar värdet av senaste screenshotet i vår store
            */
            match save_to_store(
                "latest-screenshot".to_string(),
                "".to_string(),
                app.app_handle(),
            ) {
                Ok(_v) => {}
                Err(_e) => {}
            }

            /*
               Skapar en log-fil i data-dir om det inte finns en sedan tidigare
            */
            let app_data_path = app.path_resolver().app_data_dir().unwrap();
            //println!("{:?}", app_data_path.canonicalize());

            let full_path_log_file = app_data_path.join(Path::new("gpt-log.txt"));
            if !full_path_log_file.exists() {
                let _ = fs::File::create(&full_path_log_file).unwrap();
                println!(
                    "Creating log file: {:?}",
                    full_path_log_file.to_str().unwrap()
                );
            }

            /*
               Skapar en katalog 'gpt' i användarens Picture-dir som används för screenshots
            */
            let picture_dir = picture_dir().unwrap();
            //println!("{:?}", picture_dir.canonicalize());
            let full_path_to_screenshot_dir = picture_dir.join(Path::new("gpt"));
            if !full_path_to_screenshot_dir.exists() {
                let _ = fs::create_dir(full_path_to_screenshot_dir);
                println!("Creating screenshot dir 'gpt' insude user's Picture directory");
            }

            //Kollar om vi har en API-key sparad i vår Store
            /* let stores = app.state::<StoreCollection<Wry>>();
            let path = PathBuf::from(".settings.dat");

            let store_result = with_store(app.handle(), stores, path.clone(), |store| {
                //store.insert("d".to_string(), json!("c")); store.save()  // <-- WORKS!
                store
                    .get("api-key")
                    .and_then(|json| Some(json!(json)))
                    .ok_or(tauri_plugin_store::Error::NotFound(path.clone()))
            });

            match store_result {
                Ok(value) => {
                    let api_key: String = serde_json::from_value(value).unwrap();
                    println!("{}", api_key);
                    let key = "OPENAI_API_KEY";
                    env::set_var(key, api_key);

                    /* let api_key_object = value.get("value");
                    match api_key_object {
                        Some(v) => {
                            let api_key: String = serde_json::from_value(v.clone()).unwrap();
                            println!("{}", api_key);
                            let key = "OPENAI_API_KEY";
                            env::set_var(key, api_key);
                        }
                        None => {
                          println!("Error: None from store");
                        }
                    } */
                }
                Err(_) => {
                    println!("Error: unable to fetch api-key from store")
                }
            } */

            let mut prompt = "".to_string();

            match app.get_cli_matches() {
                Ok(matches) => {
                    // Vi hamnar här om man har startat med något argument som finns, eller utan argument

                    //println!("{:?}", matches.args);
                    // println!("{:?}", matches.args.get("prompt"));

                    // Hanterar argumentet "prompt" eller "-p"
                    match matches.args.get("prompt").unwrap().value.as_array() {
                        None => {}
                        Some(value_array) => {
                            println!("Startar med prompt från --prompt eller -p");
                            for value in value_array.iter() {
                                prompt.push_str(value.as_str().unwrap());
                                prompt.push_str(" ");
                            }
                        }
                    }

                    match matches.args.get("screenshot").unwrap().value.as_bool() {
                        None => {}
                        Some(value) => {
                            if value {
                                println!("Startar med --screnshot eller -s");
                                let docs_window = tauri::WindowBuilder::new(
                                    app,
                                    "screenshot",
                                    tauri::WindowUrl::App("screenshot_window.html".into()),
                                )
                                .transparent(true)
                                .build()?;

                                let _ = docs_window.set_fullscreen(true);
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("Startat med prompt från Catch all");
                    // No matches
                    let mut args: Vec<String> = env::args().collect();
                    args.remove(0); // Tar bort första som är path till executable
                    if args.len() > 0 {
                        // Appen startade med argument som vi inte känner igen, tex
                        //   ./minapp <någon text>
                        // Då ska vi behandla <någon text> som en prompt, dvs
                        // ./minapp -p <någon text> och ./minapp <någon text> ska behandlas lika

                        prompt = args.join(" ");
                    }
                }
            }

            //println!("{}", prompt.trim());

            let key = "CHATGPT_PROMPT";
            env::set_var(key, prompt.clone());

            // Kollar så vi har vår API key
            let mut key_exist = false;
            let store_key = get_key_from_store("api-key".to_string(), app.handle());
            if store_key.len() > 0 {
                key_exist = true;
                let key = "OPENAI_API_KEY";
                env::set_var(key, store_key);
            } else {
                println!("INFO: No api-key in store on startup");
            }

            if !prompt.is_empty() && key_exist {
                let app_handle_clone = app.app_handle().clone();

                let _heavy = task::spawn(async move {
                    match call_chatgpt(prompt.clone(), app_handle_clone.clone()).await {
                        Ok(_v) => {
                            app_handle_clone
                                .get_window("main")
                                .expect("test")
                                .emit("createPromptAndResponse", Payload { data: prompt })
                                .unwrap();
                        }
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_response_to_frontend,
            send_prompt_to_frontend,
            call_chatgpt,
            save_to_store,
            get_key_from_store,
            take_screenshot,
            show_screenshot_window
        ])
        /*
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a left click");
            }
            SystemTrayEvent::RightClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a right click");
            }
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                println!("system tray received a double click");
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "toggle" => {
                    // and move it to another function or thread
                    let item_handle = app.tray_handle().get_item(&id);
                    let window = app.get_window("main").unwrap();
                    match window.is_visible() {
                        Err(_) => println!("Got an error"),
                        Ok(true) => {
                            window.hide().unwrap();
                            item_handle.set_title("Show").unwrap();
                        }
                        Ok(false) => {
                            window.show().unwrap();
                            item_handle.set_title("Hide").unwrap();
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        })
         */
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application")

    // Använd nedan ihop med Tray icon
    /*
    .build(tauri::generate_context!())
    .expect("error while running tauri application")
    .run(|_app_handle, event| match event {
        tauri::RunEvent::WindowEvent {
            label,
            event: win_event,
            ..
        } => match win_event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let win = _app_handle.get_window(label.as_str()).unwrap();
                win.hide().unwrap();

                let menuitem = "toggle".to_string();
                let item_handle = _app_handle.tray_handle().get_item(&menuitem);
                item_handle.set_title("Show").unwrap();

                api.prevent_close();
            }
            _ => {}
        },
        _ => {}
    });
     */
}
