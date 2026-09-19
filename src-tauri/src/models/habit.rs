use sqlx::FromRow; 
//use uuid::Uuid;
use serde::{Serialize,Deserialize};
use sqlx::{SqlitePool};
use serde_json::json;

use crate::models::node::{Action,NodeType};



#[derive(Debug,FromRow,Clone,Serialize,Deserialize)] 
pub struct Habit{
    id: String, 
    node_id: String, 
    goal_id: Option<String>, 
    title: String,
    frequency: String,
    target_time: u32, 
    streak_count: u32,
}

#[typetag::serde]
impl Action for Habit{
    fn get_uuid(&self)->&str {
        &self.id
    }
    fn get_json_str(&self)->String {
        self.get_json_str()
    }
    fn get_node_type(&self)->NodeType {
        NodeType::HABIT
    }
    fn modify_fields(&mut self,json_str:String)->anyhow::Result<()>{
        let obj:Habit = serde_json::from_str(&json_str)?;
        self.title = obj.title;
        self.frequency= obj.frequency;
        self.target_time = obj.target_time;
        self.streak_count = obj.streak_count;
        Ok(())
    }
    fn get_json_fields(&self)->anyhow::Result<serde_json::Value>{
        Ok(serde_json::to_value(self).unwrap())
    }
    fn get_goal_id(&self)->Option<String>{
        self.goal_id.clone()
    }
}

impl Habit{
    pub fn new(id:&str, title:String, frequency: String, target_time:u32 ,goal_id:Option<String>,)->Self{
        Self{
            id:String::from(id),
            node_id:String::from(id), 
            goal_id,
            title, 
            frequency, 
            target_time, 
            streak_count:0
        }
    }
    pub fn get_json_str(&self)->String{
        let self_json = json!({
            "type":"HABIT",
            "id":self.id, 
            "node_id":self.node_id,
            "goal_id":self.goal_id, 
            "title":self.title, 
            "target_time":self.target_time,
            "streak_count":self.streak_count
        }); 
        self_json.to_string() 
    }
}


pub async fn upload_habit(pool: &SqlitePool, habit: &Habit ) -> anyhow::Result<()> {
    let query = "INSERT INTO habits (id,node_id,title,frequency,target_time,streak_count,goal_id) VALUES ($1,$2,$3,$4,&5,&6,&7)";
    sqlx::query(query)
        .bind(&habit.id)
        .bind(&habit.node_id)
        .bind(&habit.title)
        .bind(&habit.frequency)
        .bind(&habit.target_time)
        .bind(&habit.streak_count)
        .bind(&habit.goal_id)
        .execute(pool)
        .await?;
    Ok(())
}



pub async fn delete_habit(pool:&SqlitePool, habit:Habit)->anyhow::Result<()>{
    let query = "DELETE FROM habits WHERE id=$1";
    sqlx::query(query)
        .bind(habit.id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn save_habit(pool: &SqlitePool, habit: &Habit) -> anyhow::Result<()> {
    let query = "UPDATE habits SET node_id = ?, title = ?, frequency = ?, target_time = ?, streak_count = ?, goal_id = ? WHERE id = ?;";
    sqlx::query(query)
        .bind(&habit.node_id)
        .bind(&habit.title)
        .bind(&habit.frequency)
        .bind(&habit.target_time)
        .bind(&habit.streak_count)
        .bind(&habit.goal_id)
        .bind(&habit.id) // Always bind the WHERE clause variable last to match the query order
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_habits(pool: &SqlitePool, group_id:Option<&str>) -> anyhow::Result<Vec<Habit>> {
    let habits =
        sqlx::query_as::<_, Habit>(r#"SELECT id,node_id,goal_id,title,frequency,target_time,streak_count FROM habits
                                                   WHERE ($1 IS NULL or goal_id = $1)"#)
            .bind(group_id)
            .fetch_all(pool)
            .await?;
    Ok(habits)
}

pub async fn get_habit(pool: &SqlitePool,node_id:&str) -> anyhow::Result<Habit> {
    let habit =
        sqlx::query_as::<_, Habit>(r#"SELECT id,node_id,goal_id,title,frequency,target_time,streak_couns FROM habits WHERE id=$1"#)
            .bind(node_id)
            .fetch_one(pool)
            .await?;
    Ok(habit)
}
