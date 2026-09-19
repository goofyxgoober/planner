// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod core;
pub mod models; 
pub mod db; 
pub mod state;

use core::dag::{Dag};
use std::{ ops::DerefMut};
use db::connection::{establish_connection};
use serde_json::{Value};
use models::operation::{Op,apply_op};

use state::state::{AppState,initilize_state};

use tauri::{ Manager,State};
use tokio::sync::Mutex;


#[tauri::command]
async fn get_snapshot(goal_id: String) -> Result<(Value,i64),String> {
    let mut dag = Dag::new(&goal_id).await.map_err(|err| format!("{err:?}"))?;

    dag.download_dag().await.map_err(|err| {
        eprintln!("get_snapshot error: {err:?}");
        format!("{err:?}")
    })?;

    let result = serde_json::to_value(dag.to_snapshot()).map_err(|err| format!("{err:?}"))?;
    Ok((result,dag.get_version_num().clone()))
}


//Todo make proper returning interface for apply opp
//Figure outhow to keep a map of dags persistent in memory and load it
#[tauri::command]
async fn propose_op(state:State<'_,Mutex<AppState>>,op:Op,base_version:i64,goal_id: Option<String>)->Result<(Vec<Op>,i64,Value),String>{
    let mut mut_gaurd = state.lock().await;
    let state = mut_gaurd.deref_mut();    
    let pool = establish_connection().await.map_err(|err|format!("{err:?}") )?;
    let ops = apply_op(&mut state.dag_map, &pool, &op, &base_version, &goal_id).await.map_err(|err| format!("{err:?}"));
    ops

}

#[tauri::command]
async fn fetch_goal_ids(state:State<'_,Mutex<AppState>>)->Result<Vec<String>,String>{
    let mut mut_gaurd = state.lock().await; 
    let state = mut_gaurd.deref_mut();  
    let goal_ids:Vec<&String> = state.dag_map.keys().collect();
    let mut  goal_ids_copy:Vec<String> = [].to_vec();
    for id in goal_ids{
        goal_ids_copy.push(String::from(id));
    }

    Ok(goal_ids_copy)
}


#[tauri::command]
fn greet(name: &str) -> String {
    let return_str = format!("Hello, {}! You've been greeted from Rust!", name);
    return_str
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let mut app_state = AppState::default();
    initilize_state(&mut app_state).await.unwrap();
    tauri::Builder::default()
        .setup(move |app|{
            let handle = app.handle().clone();
            handle.manage(Mutex::new(app_state)); 
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet,get_snapshot,propose_op,fetch_goal_ids])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
