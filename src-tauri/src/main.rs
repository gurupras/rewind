// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod proc;

use std::env;
use std::fs;
use proc::resume_process;
use proc::stop_process;
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};
use std::{process::{Command, Child}, path::Path};
use std::net::TcpListener;

use std::thread;
use std::fs::File;
use std::io::{Read, Write};

// Global variable to store the process handle
static mut FFMPEG_PROCESS: Option<Child> = None;
static mut RECORDING_DIR: Option<String> = None;


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn update_config(record_seconds_str: String) -> String {
    let record_seconds = record_seconds_str.parse::<i32>().unwrap();
    stop_ffmpeg();
    start_ffmpeg(record_seconds);
    "OK".into()
}

fn get_m3u8_path() -> String {
    let recording_dir = unsafe { RECORDING_DIR.as_ref().unwrap() };
    let m3u8_path = Path::new(recording_dir).join("list.m3u8");
    let m3u8_path_str = m3u8_path.to_string_lossy().to_string();
    return m3u8_path_str;
}

fn start_ffmpeg(duration: i32) {
    let ffmpeg_command = "ffmpeg";
    
    let num_segments = duration / 3;

    // Set the appropriate ffmpeg_args based on the platform
    let mut file = String::new();
    let mut input = String::new();
    if cfg!(target_os = "windows") {
        file = String::from("gdigrab");
        input = String::from("display");
    } else if cfg!(target_os = "linux") {
        file = String::from("x11grab");
        let display = env::var("DISPLAY").unwrap_or(":0".to_string());
        input = String::from(display) + ".0+0,0";
    } else if cfg!(target_os = "macos") {
        file = String::from("avfoundation");
        input = String::from("1");
    } else {
        panic!("Unsupported platform")
    };

    let recording_dir = unsafe { RECORDING_DIR.as_ref().unwrap() };

    let m3u8_path_str = get_m3u8_path();
    let seg_path = Path::new(recording_dir).join("seg%d.ts");
    let seg_path_str = seg_path.to_string_lossy().to_string();
    
    let num_segments_str = format!("{num_segments}");

    let ffmpeg_args = [
        "-y",
        "-f", &file,
        "-r", "30",
        "-i", &input,
        "-force_key_frames", "expr:gte(t,n_forced*4)",
        "-c:v", "h264",
        "-qscale", "0",
        "-crf", "30",
        "-preset", "ultrafast",
        "-f", "segment",
        "-segment_time", "4",
        "-segment_wrap", &num_segments_str,
        "-segment_list", &m3u8_path_str,
        "-segment_list_size", &num_segments_str,
        &seg_path_str
    ];
    
    // Execute the ffmpeg process with command-line arguments
    let process = Command::new(ffmpeg_command)
        .args(&ffmpeg_args)
        .spawn()
        .expect("Failed to start ffmpeg process");
    
    // Save the process handle globally
    unsafe {
        FFMPEG_PROCESS = Some(process);
    }
}

fn stop_ffmpeg() {
    unsafe {
        if let Some(mut process) = FFMPEG_PROCESS.take() {
            // Terminate the ffmpeg process
            process.kill().expect("Failed to terminate ffmpeg process");
            
            // Wait for the process to finish
            process.wait().expect("Failed to wait for ffmpeg process to finish");
        }
    }
}

fn ffmpeg_concat(m3u8_path: String) -> String {
    let recording_dir = unsafe { RECORDING_DIR.as_ref().unwrap() };
    
    let out = Path::new(recording_dir).join("output.mp4").to_string_lossy().to_string();

    let ffmpeg_args = [
        "-y",
        "-i", &m3u8_path,
        "-c", "copy",
        &out
    ];

    let mut process = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .spawn()
        .expect("Failed to start ffmpeg process");
    process.wait().expect("Failed to wait for ffmpeg concat process");
    return out;
}

pub fn setup_web_server(listener: TcpListener) {
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let response = match String::from_utf8(buffer.to_vec()) {
            Ok(request) => {
                if request.starts_with("GET /api/pause") {
                    unsafe {
                        if let Some(process) = FFMPEG_PROCESS.take() {
                            let pid = process.id();
                            println!("Pausing PID: {pid}");
                            stop_process(pid).expect("Failed to stop process");
                            FFMPEG_PROCESS.replace(process);
                        }
                    }
                    "HTTP/1.1 200 OK\r\n\r\nOK".to_string()
                }  else if request.starts_with("GET /api/resume") {
                    unsafe {
                        if let Some(process) = FFMPEG_PROCESS.take() {
                            let pid = process.id();
                            println!("Resuming PID: {pid}");
                            resume_process(pid).expect("Failed to resume process");
                            FFMPEG_PROCESS.replace(process);
                        }
                    }
                    "HTTP/1.1 200 OK\r\n\r\nOK".to_string()
                }  else if request.starts_with("GET /api/finalize") {
                    // Get m3u8 path, copy the file to another location
                    let m3u8_path = get_m3u8_path();
                    let copy_path = get_m3u8_path() + ".copy";
                    let mut src = File::open(m3u8_path).expect("Failed to open m3u8 file");
                    let mut dst = File::create(copy_path.clone()).expect("Failed to create the m3u8 copy file");
                    let mut contents = String::new();
                    src.read_to_string(&mut contents).expect("Failed to read m3u8 file contents");
                    // Write #EXT-X-ENDLIST to signal end of manifest
                    contents += "\n#EXT-X-ENDLIST";
                    dst.write_all(contents.as_bytes()).expect("Failed to copy bytes into m3u8 copy file");
                    // Ask ffmpeg to convert this into a file
                    let concat_file = ffmpeg_concat(copy_path.clone());
                    // Stream back file path
                    format!("HTTP/1.1 200 OK\r\n\r\n{concat_file}").to_string()
                } else if request.starts_with("GET /api/check") {
                    "HTTP/1.1 200 OK\r\n\r\nOK".to_string()
                } else {
                    "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
                }
            }
            Err(_) => "HTTP/1.1 400 BAD REQUEST\r\n\r\n".to_string(),
        };
        stream.write(response.as_bytes()).unwrap();
    }
}

fn main() {
    let mut recording_dir = env::temp_dir();
    recording_dir.push("recording");

    fs::create_dir_all(recording_dir.clone()).expect("Failed to create recording directory");

    // Save the path as a global variable
    unsafe {
        RECORDING_DIR = Some(recording_dir.to_string_lossy().to_string());
    }

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let open = CustomMenuItem::new("open".to_string(), "Open");

    let tray_menu = SystemTrayMenu::new()
    .add_item(open)
    .add_item(quit);
    let tray = SystemTray::new()
    .with_menu(tray_menu);

    let listener = TcpListener::bind("127.0.0.1:8666").unwrap();
    let handle = thread::spawn(move || {
        setup_web_server(listener);
    });

    tauri::Builder::default()
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![update_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Wait for the thread to finish
    handle.join().unwrap();
}
