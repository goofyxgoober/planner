//use tokio::sync::Mutex;

use std::collections::HashMap;



use crate::db::connection::{establish_connection};

use crate::core::dag::{Dag};
use crate::models::goal::{get_goals};


#[derive(Default)]
pub struct AppState{
    pub dag_map:HashMap<String,Dag>
}

pub async fn initilize_state(state:&mut AppState)->anyhow::Result<()>{
    let pool = establish_connection().await?;
    assert!(state.dag_map.is_empty());
    let goals=get_goals(&pool).await?;
    for goal in goals{
        let mut dag = Dag::create_with_goal(goal.clone()).await?;
        dag.download_dag().await?;
        state.dag_map.insert(String::from(goal.get_id()),dag);
    }
    Ok(())
}
