

use sqlx::sqlite::SqlitePool;
use serde::{Serialize,Deserialize};
use sqlx::{FromRow};
use serde_json::json;

use crate::models::node::{Action,NodeType};
#[derive(FromRow,Serialize,Deserialize,Debug,Clone)]
pub struct Goal {
    id: String,
    node_id:String,
    title: String,
    target_date: String,
    status: String,
    version: i64,
}

#[typetag::serde]
impl Action for Goal{
    fn get_uuid(&self)->&str {
        return &self.id
    }
    fn get_json_str(&self)->String {
        self.get_json_str()
    }
    fn get_node_type(&self)->NodeType {
        NodeType::GOAL
    }
    fn modify_fields(&mut self,json_str:String)->anyhow::Result<()>{
        let obj:Goal = serde_json::from_str(&json_str)?;
        self.title = obj.title;
        self.target_date= obj.target_date;
        self.status = obj.status;
        Ok(())
    }
    fn get_json_fields(&self)->anyhow::Result<serde_json::Value>{
        Ok(serde_json::to_value(self).unwrap())
    }
    fn get_goal_id(&self)->Option<String>{
        Some(self.id.clone())
    }

}

impl Goal {
    pub fn new(id:&str, title: String, target_date: String, status: String, version: i64) -> Self {
        Self {
            id: String::from(id),
            node_id: String::from(id),
            title,
            target_date,
            status,
            version
        }
    }
    pub fn get_id(&self)->&str{
        &self.id
    }
    pub fn get_json_str(&self)->String{
        let self_json = json!({
            "type":"GOAL",
            "id":self.id, 
            "node_id":self.node_id,
            "title":self.title, 
            "target_date":self.target_date, 
            "status":self.status
        }); 
        self_json.to_string() 
    }
    pub fn get_version_num(&self)->&i64{
        &self.version
    }
}

pub async fn upload_goal(pool: &SqlitePool, goal: Goal) -> anyhow::Result<Goal> {
    let query = "INSERT INTO goals (id,node_id,title,target_date,status,version) VALUES ($1,$2,$3,$4,&5,&6)";
    sqlx::query(query)
        .bind(&goal.id)
        .bind(&goal.node_id)
        .bind(&goal.title)
        .bind(&goal.target_date)
        .bind(&goal.status)
        .execute(pool)
        .await?;

    Ok(Goal {
        id: goal.id,
        node_id:goal.node_id,
        title: goal.title,
        target_date: goal.target_date,
        status: goal.status,
        version:goal.version
    })
}

pub async fn delete_goal(pool:&SqlitePool, goal:Goal)->anyhow::Result<()>{
    let query = "DELETE FROM goals WHERE id=$1";
    sqlx::query(query)
        .bind(goal.id)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn save_goal(pool:&SqlitePool,goal:&Goal)->anyhow::Result<()>{
    let query = "UPDATE goals SET node_id = ?, title = ?, target_date = ?, status = ? WHERE id = ?;";
    sqlx::query(query)
    .bind(&goal.node_id)
    .bind(&goal.title)
    .bind(&goal.target_date)
    .bind(&goal.status)
    .bind(&goal.id) // Always bind the WHERE clause variable last to match the query order
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_goals(pool: &SqlitePool) -> anyhow::Result<Vec<Goal>> {
    let goals =
        sqlx::query_as::<_, Goal>(r#"SELECT id,node_id,title,target_date,status,version FROM goals"#)
            .fetch_all(pool)
            .await?;
    Ok(goals)
}

pub async fn get_goal(pool: &SqlitePool,node_id:&str) -> anyhow::Result<Goal> {
    let goal =
        sqlx::query_as::<_, Goal>(r#"SELECT id,node_id,title,target_date,status,version FROM goals WHERE id=$1"#)
            .bind(node_id)
            .fetch_one(pool)
            .await?;
    Ok(goal)
}
