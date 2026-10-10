use sqlx::sqlite::SqlitePool;

use sqlx::{FromRow};
use serde::{Serialize,Deserialize};


use crate::models::node::{Action,NodeType};
use serde_json::json;
use core::fmt;
use std::fmt::{Display};
#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    id: String,
    #[sqlx(rename = "nodeId")]
    node_id: String,
    #[sqlx(rename = "goalId")]
    goal_id: Option<String>,
    #[sqlx(rename = "eventContextId")]
    event_context_id: Option<String>,
    title: String,
    #[sqlx(rename = "taskType")]
    task_type: String,
    #[sqlx(rename = "baseDuration")]
    base_duration: u32,
    #[sqlx(rename = "scheduledStart")]
    scheduled_start: String,
    #[sqlx(rename = "scheduledEnd")]
    scheduled_end: String,
    #[sqlx(rename = "urgencyScore")]
    urgency_score: f64,
    #[sqlx(rename = "importanceScore")]
    importance_score: f64,
    #[sqlx(rename = "priorityWeight")]
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
        match serde_json::to_value(self){
            Err(err) => panic!("Task json string not-avail:{err}"),
            Ok(val) => val.to_string()

        }
    }
    fn get_node_type(&self)->NodeType {
        NodeType::TASK
    }
    fn modify_fields(&mut self,json_str:String)->anyhow::Result<()>{
        println!("Object debug:{self}");
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

impl Display for Task{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{},{},{},{},{},{},{},)",self.title,self.task_type,self.base_duration,self.scheduled_start,
        self.scheduled_end,self.urgency_score,self.importance_score,self.priority_weight,self.status)
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
            "nodeId":self.node_id, 
            "goalId":self.goal_id, 
            "eventContextId":self.event_context_id, 
            "title":self.title,
            "taskType":self.task_type,
            "baseDuration":self.base_duration,
            "scheduleStart":self.scheduled_start,
            "scheduleEnd":self.scheduled_end,
            "urgencyScore":self.urgency_score,
            "importanceScore":self.importance_score,
            "priorityWeight":self.priority_weight,
            "status":self.status,
        }); 
        self_json.to_string() 
    }
}

pub async fn upload_task( pool: &SqlitePool, task: &Task) -> anyhow::Result<()> {
    let query = "INSERT INTO tasks
                       (id,nodeId,goalId,eventContextId,title,taskType,baseDuration,scheduledStart,scheduledEnd,urgencyScore,importanceScore,priorityWeight,status)
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
    let query = "UPDATE tasks SET nodeId = ?, goalId = ?, eventContextId = ?, title = ?, taskType = ?, baseDuration = ?, scheduledStart = ?, scheduledEnd = ?, urgencyScore = ?, importanceScore = ?, priorityWeight = ?, status = ? WHERE id = ?;";
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
        SELECT id,nodeId,goalId,eventContextId,
               title,taskType,baseDuration,
               scheduledStart,scheduledEnd,
               urgencyScore,importanceScore,
               priorityWeight,status 
        FROM tasks WHERE ($1 IS NULL or goalId = $1)"#,
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
            SELECT id,nodeId,goalId,eventContextId,
                   title,taskType,baseDuration,
                   scheduledStart,scheduledEnd,
                   urgencyScore,importanceScore,
                   priorityWeight,status  
            FROM tasks WHERE id=$1"#)
            .bind(node_id)
            .fetch_one(pool)
            .await?;
    Ok(habit)
}


#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskDependency {
    #[sqlx(rename = "predecessorId")]
    pub predecessor_id: String,
    #[sqlx(rename = "successorId")]
    pub successor_id: String,
    #[sqlx(rename = "goalId")]
    pub goal_id: String,
}

impl TaskDependency{
    pub fn get_json_str(&self)->String{
        let self_json = json!({
            "type":"EDGE",
            "predecessorId":self.predecessor_id, 
            "successorId":self.successor_id, 
            "goalId":self.goal_id
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
    let query = "INSERT INTO taskDependencies (predecessorId,successorId,goalId) VALUES ($1,$2,$3)";
    sqlx::query(query)
        .bind(predecessor_id)
        .bind(successor_id)
        .bind(goal_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_task_dependency(pool:&SqlitePool, successor_id: &str, predecessor_id: &str)->anyhow::Result<()>{
    let query = "DELETE FROM taskDependencies WHERE (successorId=$1 AND predecessorId=$2)";
    sqlx::query(query)
        .bind(successor_id)
        .bind(predecessor_id)
        .execute(pool)
        .await?;
    Ok(())
}



pub async fn get_task_dependencies(pool:&SqlitePool,goal_id:&str)->anyhow::Result<Vec<TaskDependency>>{
    let task_deps =
        sqlx::query_as::<_, TaskDependency>(r#"SELECT predecessorId,successorId,goalId FROM taskDependencies 
                                                            WHERE ($1 IS NULL or goalId = $1)"#)
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