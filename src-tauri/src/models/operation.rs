use serde::{Deserialize, Serialize};
use std::collections::{HashMap};


use crate::models::habit::{Habit}; 
use crate::models::task::{Task}; 
use crate::models::goal::{Goal};

use crate::core::dag::{Dag};
use crate::models::node::{Action, DagError, NodeType, get_node, upload_node};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Op {
    AddTask {
        task:Task,
        x: Option<f32>,
        y: Option<f32>,
    },
    AddHabit {
        habit:Habit,
        x: Option<f32>,
        y: Option<f32>,
    },
    AddGoal {
        goal:Goal,
        x: Option<f32>,
        y: Option<f32>,
    },
    RemoveNode {
        id: String,
    },
    AddEdge {
        predecessor_id: String,
        successor_id: String,
    },
    RemoveEdge {
        predecessor_id: String,
        successor_id: String,
    },
    MoveNode {
        id: String,
        x: f32,
        y: f32,
    },
    ModifyNode {
        id: String,
        json_str: String,
    },
    Batch {
        ops: Vec<Op>,
    },
}


pub fn get_valid_dag<'a>(
    dag_map: &'a mut HashMap<String, Dag>,
    goal_id: &Option<String>,
    base_version: &i64,
) -> anyhow::Result<&'a mut Dag> {
    let gid = goal_id.as_ref().ok_or_else(|| {
        DagError::GraphError { message: "goal_id is required".to_string() }
    })?;

    let dag = dag_map.get_mut(gid).ok_or_else(|| {
        DagError::NodeError { message: "goal graph doesn't exist in memory".to_string() }
    })?;

    if dag.fetch_goal().get_version_num() != base_version {
        return Err(DagError::GraphError { message: "base versions do not match, try again".to_string() })?;
    }

    Ok(dag)
}



pub async fn apply_op(dag_map: &mut HashMap<String, Dag>,pool:&sqlx::SqlitePool, op: &Op, base_version:&i64, goal_id:&Option<String>) -> anyhow::Result<(Vec<Op>,i64,serde_json::Value)> {
    let mut inverse_ops:Vec<Op> = Vec::new();
    let mut next_version = base_version.clone();
   

    if let Op::AddGoal { goal, x, y } = op {
        upload_node(pool, goal.clone(), x.clone(), y.clone()).await?;
        inverse_ops.push(Op::RemoveNode { id: goal.get_uuid().to_string() });
        let dag: Dag = Dag::create_with_goal(goal.clone()).await?;
        next_version+=1;
        let result = (inverse_ops,next_version,dag.to_snapshot());
        dag_map.insert(goal.get_id().to_string(),dag);
        return Ok(result);
    }

    if let Op::Batch { ops } = op{
        for sub_op in ops {
            next_version+=1;
            let (inv_op,_,_) = Box::pin(apply_op(dag_map, pool,sub_op,&next_version,goal_id)).await?; // recursion needs boxing (async fn)
            inverse_ops.extend(inv_op);
        }
        let dag =  get_valid_dag(dag_map, goal_id, base_version)?;
        return Ok((inverse_ops,next_version,dag.to_snapshot()));

    }
    
    let mut snapshot:serde_json::Value = serde_json::from_str("")?;
    //let root_goal = get_goal(pool, goal_id).await?;
    //let dag = get_valid_dag(dag_map, goal_id, base_version)?;

    match op {
        Op::AddTask {task, x, y } => {
            //upload_node(pool, task.clone(), x.clone(), y.clone()).await?;
            //calculation inverse operation
            let dag = get_valid_dag(dag_map, goal_id, base_version)?;
            dag.add_task(task.clone(),x.clone(),y.clone()).await?;
            inverse_ops.push(Op::RemoveNode { id: (String::from(task.get_uuid()))});
            snapshot = dag.to_snapshot();
            next_version+=1;
        }
        Op::AddHabit {habit, x, y } => {
            //upload_node(pool, habit.clone(), x.clone(), y.clone()).await?;
            //calculation inverse operation
            let dag = get_valid_dag(dag_map, goal_id, base_version)?;
            dag.add_habit(habit.clone(),x.clone(),y.clone()).await?;
            next_version+=1;
            inverse_ops.push(Op::RemoveNode { id: (String::from(habit.get_uuid()))});
        }
        Op::AddGoal {goal, x, y } => !unreachable!(),
        Op::AddEdge { predecessor_id, successor_id } => {
            let dag = get_valid_dag(dag_map, goal_id, base_version)?;
            let succ_node = dag.get_node(successor_id)?;
            let pred_node = dag.get_node(predecessor_id)?;
            match pred_node.item.get_goal_id(){
                Some(goal_id)=>goal_id, 
                None => return Err(DagError::EdgeError { message: ("Predecessor node needs to be connected to graph".to_string()) })?
            };

            
            dag.add_edge(successor_id.clone(), predecessor_id.clone()).await?;

            //calculating inverse operation
            snapshot = dag.to_snapshot();
            next_version+=1;
            inverse_ops.push(Op::RemoveEdge { predecessor_id: (predecessor_id.to_string()), successor_id: (successor_id.to_string()) });

        }
        Op::RemoveNode { id } =>{
            let dag = get_valid_dag(dag_map, goal_id, base_version)?;
            let node = get_node(pool,id).await?;
            let goal_id = match node.item.get_goal_id() {
                Some(goal_id) => goal_id,          // now goal_id: String
                None => {
                    return Err(DagError::EdgeError {
                        message: "Predecessor node needs to be connected to graph".to_string(),
                    })?
                }
            };

            
            //calculating inverse operation
            match node.node_type{
                NodeType::GOAL => {
                    let goal:Goal = serde_json::from_value(node.item.get_json_fields().unwrap()).unwrap();
                    inverse_ops.push(Op::AddGoal { goal, x: (node.x), y: (node.y)})
                },
                NodeType::TASK => {
                    let task:Task = serde_json::from_value(node.item.get_json_fields().unwrap()).unwrap();
                    inverse_ops.push(Op::AddTask { task, x: (node.x), y: (node.y)})
                },
                NodeType::HABIT => {
                    let habit:Habit = serde_json::from_value(node.item.get_json_fields().unwrap()).unwrap();
                    inverse_ops.push(Op::AddHabit { habit, x: (node.x), y: (node.y)})
                },
            }


            dag.delete_node(id).await?;
            snapshot = dag.to_snapshot();
            next_version+=1;
            if dag.get_goal().get_id() == goal_id{
                dag_map.remove(id);
            }
            
            
            
        }
        Op::RemoveEdge{predecessor_id,successor_id} =>{
            
        }
        Op::MoveNode { id, x, y } => {
            let dag = get_valid_dag(dag_map, goal_id, base_version)?;
            let node = dag.get_node(id)?; 

            let prev_x = node.x.unwrap_or(x.clone());
            let prev_y = node.y.unwrap_or(x.clone());
            
            dag.modify_node(id, None, Some(x.clone()), Some(y.clone())).await?;
            snapshot = dag.to_snapshot();
            next_version+=1;
            inverse_ops.push(Op::MoveNode { id: (id.to_string()), x: (prev_x), y: (prev_y) });
        }
        Op::ModifyNode {id,json_str} =>{
            let dag = get_valid_dag(dag_map, goal_id, base_version)?;
            let prev_json_str = dag.get_node(id)?.item.get_json_str();
            dag.modify_node(id, Some(json_str.to_string()), None, None).await?;
            next_version+=1;
            inverse_ops.push(Op::ModifyNode { id: (String::from(id)), json_str: (prev_json_str) });
        }
        Op::Batch { ops } => !unreachable!(),
    }
    Ok((inverse_ops,next_version,snapshot))
}


