use std::collections::{HashMap,VecDeque};
use serde::{Serialize};
use sqlx::sqlite::SqlitePool;
//Would help if I maintained my own dag datastructure in mem to manipulate then save periodically to the db 
//Would need a get_task_dependencies, and get_tasks by goal_id, as well as event_contexts
//Given tasks,habits,event_contexts, and task_dependencies Make apporpriate nodes and edges
//Make sure there's logic that prevents cross-edges that break dag and back-edges 
//Add Topological sort
use crate::models::habit::{Habit,upload_habit,get_habits}; 
use crate::models::task::{Task, get_task_dependencies, get_tasks, upload_task, upload_task_dependency,delete_task_dependency}; 
use crate::models::goal::{Goal, get_goal};
use crate::db::connection::{establish_connection};
use crate::models::node::{Action, DagError, Node, NodeType, delete_node, save_node, upload_node};

use serde_json::{Value, json};

//We can make a hash-map relating uuid:Node
//1.Need function that downloads Nodes(Tasks/Habits) from sql and configures hashmap 
//2.Need function that downloads Edges(task_dependencies) and configures list of Edges
//3.Need to implement appropriate upload and delete functions for task dependencies 
//4.We could make another hashmap using the uuids of the first element of the edges, and the respective 
//  vector of edges that exist there
pub struct Dag{
    

    pool:SqlitePool,

    goal:Goal,

    sn:Snapshot
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot{

    nodes:HashMap<String,Node>, 
    successors: HashMap<String, Vec<String>>, 
    predecessors: HashMap<String, Vec<String>>
}

//1.Make a delete node function

//2.Need to create a delete_edge function when given one hash deletes all instances of that hash in both
//  predecessors and sucessors, when given both edge hashes only deletes those hashes 



impl Dag{
    pub async fn new(goal_id:&str)->anyhow::Result<Self>{
        let pool =  establish_connection().await?;
        let goal = get_goal(&pool, goal_id).await?;
        Ok(
            Self{   
            pool,
            goal,
            sn:
                Snapshot{
                    nodes:HashMap::new(), 
                    successors:HashMap::new(), 
                    predecessors:HashMap::new() 
                }
        })

    }
    pub async fn create_with_goal(goal:Goal)->anyhow::Result<Self>{
        let pool = establish_connection().await?;
        Ok(
            Self{   
            pool,
            goal,
            sn:
                Snapshot{
                    nodes:HashMap::new(), 
                    successors:HashMap::new(), 
                    predecessors:HashMap::new()
                }
        })
    }
    pub fn fetch_goal(&self)->&Goal{
        &self.goal
    }
    pub async fn download_dag(&mut self)->anyhow::Result<()>{ 
        let tasks = get_tasks(&self.pool, Some(self.goal.get_id())).await?;
        let habits = get_habits(&self.pool, Some(self.goal.get_id())).await?;
        let task_dependencies = get_task_dependencies(&self.pool, self.goal.get_id()).await?;
        
        //Insert root node before rest
        self.sn.nodes.insert(String::from(self.goal.get_uuid()),Node::new(self.goal.clone(),None,None));
        for task in tasks{
            self.sn.nodes.insert(String::from(task.get_uuid()),Node::new(task,None,None));
        }
        for habit in habits{
            self.sn.nodes.insert(String::from(habit.get_uuid()),Node::new(habit,None,None));
        }
        //TODO: turn this O(n) fetch into a singular batch coords fetch
        for (_, node) in &mut self.sn.nodes{
            node.fetch_coords(&self.pool).await?;
        }
        for task_dep in task_dependencies{
            let succ_id = String::from(task_dep.successor_id); 
            let pred_id = String::from(task_dep.predecessor_id); 
            let sucessor_vec = self.sn.successors.entry(pred_id.clone()).or_insert(Vec::new());
            let predecessor_vec = self.sn.predecessors.entry(succ_id.clone()).or_insert(Vec::new());
            sucessor_vec.push(succ_id);
            predecessor_vec.push(pred_id);

        }
        Ok(())
    }

    pub async fn add_edge(&mut self, successor_id:String,predecessor_id:String)-> anyhow::Result<()>{
        let sucessor_vec = self.sn.successors.entry(String::from(&predecessor_id)).or_insert(Vec::new());
        let predecessor_vec = self.sn.predecessors.entry(String::from(&successor_id)).or_insert(Vec::new());
        if sucessor_vec.contains(&successor_id) || predecessor_vec.contains(&predecessor_id){
            Err(DagError::EdgeError { message: ("Edge already exists".to_string()) })?;
        }
        
        upload_task_dependency(&self.pool, &predecessor_id, &successor_id, self.goal.get_id()).await?;

          
        Ok(())
    }
    pub async fn add_task(&mut self,task:Task,x:Option<f32>,y:Option<f32>)-> anyhow::Result<()>{
        
        if self.sn.nodes.get(task.get_uuid()).is_some(){
            Err(DagError::NodeError { message: ("Task already exists".to_string()) })?;
        }

        upload_task(&self.pool, &task).await?;
        let node = upload_node(&self.pool, task, x, y).await?;
        self.sn.nodes.insert(String::from(node.item.get_uuid()),node);
        
        Ok(())
    }
    pub async fn add_habit(&mut self, habit:Habit,x:Option<f32>,y:Option<f32>)->anyhow::Result<()>{
        if self.sn.nodes.get(habit.get_uuid()).is_some(){
            Err(DagError::NodeError { message: ("Habit already exists".to_string()) })?;
        }

        upload_habit(&self.pool, &habit).await?;

        let node = upload_node(&self.pool, habit, x, y).await?;
        self.sn.nodes.insert(String::from(node.item.get_uuid()),node);
        
        Ok(())
    }

    pub async fn add_node_if_not_present(&mut self, mut node: Node) -> anyhow::Result<()> {
        if node.node_type != NodeType::GOAL {
            let mut mutable_json_fields = node.item.get_json_fields()?.clone();
            mutable_json_fields["goal_id"] = json!(self.goal.get_id());
            node.item.modify_fields(mutable_json_fields.to_string())?;
        }
        self.sn.nodes.entry(String::from(node.item.get_uuid())).or_insert(node);
        Ok(())
    }

    pub async fn modify_node(&mut self, id:&str,json_str:Option<String>, mut x:Option<f32>, mut y:Option<f32>)-> anyhow::Result<()>{
        let node =  match self.sn.nodes.get_mut(id){
            None =>{
                return Err(DagError::NodeError { message: "node doesn't exist".to_string() })?;},
            Some(node) => node
        };

        let use_json_str:String = match json_str{
            None => node.item.get_json_str(),
            Some(ele)=> ele
        };
        if x.is_some() && y.is_some(){
            x = node.x.clone(); 
            y = node.y.clone();
        }
        node.item.modify_fields(use_json_str)?;
        save_node(&self.pool, node, (x,y)).await?;
        Ok(())
    }

    pub async fn delete_node(&mut self,id:&str)->anyhow::Result<()>{
        if !self.sn.nodes.get(id).is_some(){
            Err(DagError::NodeError { message: ("Node doesn't exist".to_string()) })?;
        }

        delete_node(&self.pool, self.sn.nodes.remove(id).unwrap()).await?;
        self.delete_all_incoming_and_outgoing_edges(id).await?;   
        Ok(())
    }

    pub fn get_node(&self, id:&str)->anyhow::Result<&Node>{
        let node= match self.sn.nodes.get(id){
            None => {
                let len = (&self.sn.nodes).len();
                println!("Length of nodes hashmap:{len}");
                for (key,_) in &self.sn.nodes{
                    println!("{key}");
                }
                println!("Failed fetching this node:{id}");
                return Err(DagError::NodeError { message: "node doesn't exist".to_string() })?;},
            Some(node) => node,
        }; 
        Ok(node)
    }

    pub fn get_goal(&self)->&Goal{
        &self.goal
    }
    pub async fn delete_edge(&mut self,predecessor_id:&str, successor_id:&str)-> anyhow::Result<()>{
        _ = self.sn.successors.get_mut(predecessor_id)
                                .unwrap()
                                .extract_if(.., |x|x == successor_id);
        _ = self.sn.predecessors.get_mut(successor_id)
                                .unwrap()
                                .extract_if(.., |x| x == predecessor_id);
        delete_task_dependency(&self.pool, successor_id, predecessor_id).await?;
        Ok(())
    }
    pub async fn delete_all_incoming_and_outgoing_edges(&mut self, node_id:&str)->anyhow::Result<()>{
        let sucessors = self.sn.successors.remove(node_id);
        let predecessors = self.sn.predecessors.remove(node_id);
        for pred in predecessors.unwrap(){
            delete_task_dependency(&self.pool, node_id, &pred).await?;
            _ = self.sn.successors.get_mut(&pred)
                                    .unwrap()
                                    .extract_if(.., |x| x==node_id);
        }
        for succ in sucessors.unwrap(){
            delete_task_dependency(&self.pool, &succ, node_id).await?;
            _ = self.sn.predecessors.get_mut(&succ)
                                    .unwrap()
                                    .extract_if(..,|x| x ==node_id);
        }
        Ok(())
    }
    pub fn topological_sort(&mut self)->Option<Vec<&str>>{
        let mut queue: VecDeque<&str> = VecDeque::new();
        let mut in_degree: HashMap<&str,usize> = HashMap::new();
        let mut topo_sort: Vec<&str> = Vec::new();
        for (uuid,vec) in &self.sn.predecessors{
            let count = vec.len();
            in_degree.insert(uuid,count);
            if count == 0{
                queue.push_back(uuid);
            }
        }
        
        while queue.len() != 0{
            let node_uuid = queue.pop_front().expect("Queue should never be empty");
            topo_sort.push(node_uuid);
            let successor_ids = self.sn.successors.get(node_uuid).unwrap();
            for succ_uuid in successor_ids{
                let count = in_degree.get_mut(succ_uuid.as_str()).unwrap();
                *count-=1; 
                if *count == 0{
                    queue.push_back(succ_uuid);
                }
                
            }
        }

        if self.sn.nodes.len() == topo_sort.len(){
            Some(topo_sort)
        }
        else{
            None   
        }
    }
    pub fn to_json_string(&self)->String{
        let value = json!({
            "nodes": self.sn.nodes, 
            "successors": self.sn.successors,
            "predecessors": self.sn.predecessors
        });
        value.to_string()
    }
    pub fn to_snapshot(&self)->Value{
        serde_json::to_value(&self.sn).unwrap()
    }
    pub fn get_version_num(&self)->&i64{
        self.goal.get_version_num()
    }

}