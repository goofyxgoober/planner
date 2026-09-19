use sqlx::sqlite::SqlitePool;

use sqlx::{FromRow};
use serde::{Serialize,Deserialize};


use crate::models::node::{Action,NodeType};
use serde_json::json;

#[derive(Clone,FromRow,Serialize,Deserialize,Debug)]
pub struct Task {
    id: String,
    node_id:String,
    goal_id: Option<String>,
    event_context_id: Option<String>,
    title: String,
    task_type: String,
    base_duration: u32,
    scheduled_start: String,
    scheduled_end: String,
    urgency_score: f64,
    importance_score: f64,
    priority_weight: f64,
    status: String,
}


/*

    Task trait implementation


*/

#[typetag::serde]
impl Action for Task{
    fn get_uuid(&self)->&str {
        &self.id
    }
    fn get_json_str(&self)->String {
        self.get_json_str()
    }
    fn get_node_type(&self)->NodeType {
        NodeType::TASK
    }
    fn modify_fields(&mut self,json_str:String)->anyhow::Result<()>{
        let obj:Task = serde_json::from_str(&json_str)?;
        self.title = obj.title;
        self.task_type = obj.task_type;
        self.base_duration = obj.base_duration;
        self.scheduled_start = obj.scheduled_start;
        self.scheduled_end = obj.scheduled_end;
        self.urgency_score = obj.urgency_score;
        self.importance_score = obj.importance_score;
        self.priority_weight = obj.priority_weight;
        self.status = obj.status;
        Ok(())
    }
    fn get_json_fields(&self)->anyhow::Result<serde_json::Value>{
        Ok(serde_json::to_value(self).unwrap())
    }
    fn get_goal_id(&self)->Option<String>{
        self.goal_id.clone()
    }
    
}


impl Task{
    pub fn new(
        id:&str,
        goal_id: Option<String>,
        event_context_id: Option<String>,
        title: String,
        task_type: String,
        base_duration: u32,
        scheduled_start: String,
        scheduled_end: String,
        urgency_score: f64,
        importance_score: f64,
        priority_weight: f64,
        status: String,
    ) -> Self {
        Self {
            id: String::from(id),
            node_id: String::from(id),
            goal_id,
            event_context_id,
            title,
            task_type,
            base_duration,
            scheduled_start,
            scheduled_end,
            urgency_score,
            importance_score,
            priority_weight,
            status,
        }
    }
    pub fn get_json_str(&self)->String{
        let self_json = json!({
            "type":"TASK",
            "id":self.id,
            "node_id":self.node_id, 
            "goal_id":self.goal_id, 
            "event_context_id":self.event_context_id, 
            "title":self.title,
            "task_type":self.task_type,
            "base_duration":self.base_duration,
            "schedule_start":self.scheduled_start,
            "schedule_end":self.scheduled_end,
            "urgency_score":self.urgency_score,
            "importance_score":self.importance_score,
            "priority_weight":self.priority_weight,
            "status":self.status,
        }); 
        self_json.to_string() 
    }
}

pub async fn upload_task( pool: &SqlitePool, task: &Task) -> anyhow::Result<()> {
    let query = "INSERT INTO tasks
                       (id,node_id,goal_id,event_context_id,title,task_type,base_duration,scheduled_start,scheduled_end,urgency_score,importance_score,priority_weight,status)
                       VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,&13)";
    sqlx::query(query)
        .bind(&task.id)
        .bind(&task.node_id)
        .bind(&task.goal_id)
        .bind(&task.event_context_id)
        .bind(&task.title)
        .bind(&task.task_type)
        .bind(&task.base_duration)
        .bind(&task.scheduled_start)
        .bind(&task.scheduled_end)
        .bind(task.urgency_score)
        .bind(task.importance_score)
        .bind(task.priority_weight)
        .bind(&task.status)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_task(pool:&SqlitePool, task:Task)->anyhow::Result<()>{
    let query = "DELETE FROM tasks WHERE id=$1";
    sqlx::query(query)
        .bind(task.id)
        .execute(pool)
        .await?;
    Ok(())
}


pub async fn save_task(pool: &SqlitePool, task: &Task) -> anyhow::Result<()> {
    let query = "UPDATE tasks SET node_id = ?, goal_id = ?, event_context_id = ?, title = ?, task_type = ?, base_duration = ?, scheduled_start = ?, scheduled_end = ?, urgency_score = ?, importance_score = ?, priority_weight = ?, status = ? WHERE id = ?;";
    sqlx::query(query)
        .bind(&task.node_id)
        .bind(&task.goal_id)
        .bind(&task.event_context_id)
        .bind(&task.title)
        .bind(&task.task_type)
        .bind(&task.base_duration)
        .bind(&task.scheduled_start)
        .bind(&task.scheduled_end)
        .bind(&task.urgency_score)
        .bind(&task.importance_score)
        .bind(&task.priority_weight)
        .bind(&task.status)
        .bind(&task.id) // Always bind the WHERE clause variable last to match the query order
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_tasks(
    pool: &SqlitePool,
    goal_id: Option<&str>,
) -> anyhow::Result<Vec<Task>> {
    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT id,node_id,goal_id,event_context_id,
               title,task_type,base_duration,
               scheduled_start,scheduled_end,
               urgency_score,importance_score,
               priority_weight,status 
        FROM tasks WHERE ($1 IS NULL or goal_id = $1)"#,
    )
    .bind(goal_id)
    .fetch_all(pool)
    .await?;
    Ok(tasks)
}

pub async fn get_task(pool: &SqlitePool,node_id:&str) -> anyhow::Result<Task> {
    let habit =
        sqlx::query_as::<_, Task>(
            r#"
            SELECT id,node_id,goal_id,event_context_id,
                   title,task_type,base_duration,
                   scheduled_start,scheduled_end,
                   urgency_score,importance_score,
                   priority_weight,status  
            FROM tasks WHERE id=$1"#)
            .bind(node_id)
            .fetch_one(pool)
            .await?;
    Ok(habit)
}


#[derive(Debug, FromRow)]
pub struct TaskDependency {
    pub predecessor_id: String,
    pub successor_id: String,
    pub goal_id: String
}

impl TaskDependency{
    pub fn get_json_str(&self)->String{
        let self_json = json!({
            "type":"EDGE",
            "predecessor_id":self.predecessor_id, 
            "successor_id":self.successor_id, 
            "goal_id":self.goal_id
        }); 
        self_json.to_string() 
    }
}
pub async fn upload_task_dependency(
    pool: &SqlitePool,
    predecessor_id: &str,
    successor_id: &str,
    goal_id: &str,
) -> anyhow::Result<()> {
    let query = "INSERT INTO task_dependencies (predecessor_id,successor_id,goal_id) VALUES ($1,$2,$3)";
    sqlx::query(query)
        .bind(predecessor_id)
        .bind(successor_id)
        .bind(goal_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_task_dependency(pool:&SqlitePool, successor_id: &str, predecessor_id: &str)->anyhow::Result<()>{
    let query = "DELETE FROM task_dependencies WHERE (successor_id=$1 AND predecessor_id=$2)";
    sqlx::query(query)
        .bind(successor_id)
        .bind(predecessor_id)
        .execute(pool)
        .await?;
    Ok(())
}



pub async fn get_task_dependencies(pool:&SqlitePool,goal_id:&str)->anyhow::Result<Vec<TaskDependency>>{
    let task_deps =
        sqlx::query_as::<_, TaskDependency>(r#"SELECT predecessor_id,successor_id,goal_id FROM task_dependencies 
                                                            WHERE ($1 IS NULL or goal_id = $1)"#)
            .bind(goal_id)
            .fetch_all(pool)
            .await?;
    Ok(task_deps)
}


/*

#[derive(Debug, FromRow)]
pub struct TaskFeedback {
    id: String,
    task_id: String,
    estimated_duration: u32,
    actual_duration: u32,
    user_sentiment: String,
    completion_quality: u8,
    created_at: String,
}

*/