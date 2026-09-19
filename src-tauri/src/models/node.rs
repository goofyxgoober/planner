//use std::vec;
//use std::collections::{HashMap,VecDeque};
use serde::{Deserialize, Serialize};
//use sqlx::Sqlite;
use sqlx::Row;
use sqlx::sqlite::SqlitePool;
//use uuid::Uuid;

use thiserror::Error;
//Would help if I maintained my own dag datastructure in mem to manipulate then save periodically to the db 
//Would need a get_task_dependencies, and get_tasks by goal_id, as well as event_contexts
//Given tasks,habits,event_contexts, and task_dependencies Make apporpriate nodes and edges
//Make sure there's logic that prevents cross-edges that break dag and back-edges 
//Add Topological sort
use crate::models::habit::{get_habit,save_habit}; 
use crate::models::task::{get_task, save_task}; 
use crate::models::goal::{get_goal,save_goal};

#[derive(Error,Debug)]
pub enum DagError{
    #[error("Edge error: {message}")]
    EdgeError{message:String},

    #[error("Node error: {message}")]
    NodeError{message:String},

    #[error("Error Parsing Node Type")]
    ParseNodeTypeError,

    #[error("Graph Error: {message}")]
    GraphError{message:String},
}

#[derive(Serialize,Deserialize,PartialEq)]
pub enum NodeType{
    HABIT, 
    GOAL, 
    TASK
}


#[typetag::serde(tag="type")]
pub trait Action: Send + Sync{
    fn get_uuid(&self)->&str;
    fn get_goal_id(&self)->Option<String>;
    fn get_json_str(&self)->String;
    fn get_node_type(&self)->NodeType; 
    fn modify_fields(&mut self,json_str:String)->anyhow::Result<()>;
    fn get_json_fields(&self)->anyhow::Result<serde_json::Value>;
    
}

#[derive(Serialize)]
pub struct Node{
    id:String,
    pub node_type: NodeType, 
    pub x:Option<f32>, 
    pub y:Option<f32>,

    pub item: Box<dyn Action>
} 

impl Node{

    pub fn new(action: impl Action + 'static,x:Option<f32>,y:Option<f32>) -> Self{
        let action_pntr = Box::new(action);
        let node_type = action_pntr.get_node_type();
        let id = String::from(action_pntr.get_uuid()); 
        
        Self{
            id,
            item:action_pntr, 
            node_type,
            x,
            y
        }
    }
    pub fn with_item(item: Box<dyn Action>,x:Option<f32>,y:Option<f32>)->Self{
        let node_type = item.get_node_type();
        let id = String::from(item.get_uuid()); 
        
        Self{
            id,
            item:item, 
            node_type,
            x,
            y
        }
    }
    pub async fn fetch_coords(&mut self,pool:&SqlitePool)->anyhow::Result<()>{
        let query = "SELECT x,y FROM nodes WHERE id=$1";
        let row = sqlx::query(query)
            .bind(&self.id)
            .fetch_one(pool)
            .await?;
        self.x = row.get("x"); 
        self.y = row.get("y");
        Ok(())
    }
    pub async fn set_coords(&mut self, x:Option<f32>, y:Option<f32>){
        self.x = x;
        self.y= y;
    }
    pub async fn save_coords(&mut self,pool:&SqlitePool)->anyhow::Result<()>{
        let query = "UPDATE nodes SET x = ?, y= ? WHERE id= ?";
        sqlx::query(query)
            .bind(&self.x)
            .bind(&self.y)
            .bind(&self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    
}


pub async fn upload_node(pool:&SqlitePool,action: impl Action + 'static + Clone,x:Option<f32>,y:Option<f32>)->anyhow::Result<Node>{
    let query = "INSERT INTO nodes (id,node_type,x,y) VALUES ($1,$2,$3,$4)";
    let node = Node::new(action.clone(), x, y);
    let node_type_str = match action.get_node_type() {
        NodeType::GOAL => "goal",
        NodeType::HABIT => "habit",
        NodeType::TASK => "task"
    };
    sqlx::query(query)
        .bind(&node.id)
        .bind(&node_type_str)
        .bind(&node.x)
        .bind(&node.y)
        .execute(pool)
        .await?;
    Ok(node)
}

pub async fn get_node(pool: &SqlitePool,node_id:&str) -> anyhow::Result<Node> {
    let query = "SELECT node_type,x,y FROM nodes WHERE id=$1";
    
    let row = sqlx::query(query)
        .bind(node_id)
        .fetch_one(pool)
        .await?;
    let x:f32 = row.get("x"); 
    let y:f32 = row.get("y");
    let node_type_str:String = row.get("node_type");

    let node_type = match node_type_str.to_lowercase().as_str() {
            "goal" => Ok(NodeType::GOAL),
            "task" => Ok(NodeType::TASK),
            "habit" => Ok(NodeType::HABIT),
            _ => Err(DagError::ParseNodeTypeError), // Fallback case for unexpected inputs
        }?;
    
    let item:Box<dyn Action> = match node_type{
        NodeType::GOAL =>  Box::new(get_goal(pool, node_id).await?),
        NodeType::TASK => Box::new(get_task(pool, node_id).await?), 
        NodeType::HABIT =>  Box::new(get_habit(pool, node_id).await?)
    };
    Ok(Node::with_item(item,Some(x),Some(y)))
}

pub async fn delete_node(pool:&SqlitePool, node:Node)->anyhow::Result<()>{
    let query = "DELETE FROM nodes WHERE (id = $1)"; 
    sqlx::query(query)
        .bind(node.id)
        .execute(pool)
        .await?;
    Ok(())
}


pub async fn save_node(pool:&SqlitePool, node:&mut Node, position:(Option<f32>,Option<f32>))->anyhow::Result<()>{
    match node.node_type{
        NodeType::GOAL =>{
            let goal = get_goal(pool, &node.id).await?;
            save_goal(pool, &goal).await?;
        },
        NodeType::HABIT => {
            let habit = get_habit(pool,&node.id).await?;
            save_habit(pool, &habit).await?;
        }, 
        NodeType::TASK => {
            let task = get_task(pool,&node.id).await?;
            save_task(pool, &task).await?;
        }
    }
    let (x,y) =position;
    let query = "UPDATE nodes x = ?, y = ?, WHERE id = ?;";
    sqlx::query(query)
    .bind(&x)
    .bind(&y)
    .bind(&node.id)
    .execute(pool)
    .await?;

    Ok(())
}