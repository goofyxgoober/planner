// App.tsx
import { useEffect, useState } from 'react';
import {GraphContainer} from './components/GraphContainer';
import {invoke} from '@tauri-apps/api/core';

export default function App() {
  const [goalIds,setGoalIds] = useState([""]);
  

  useEffect(()=>{
    async function loadGoalIds(){
      const result = await invoke<string[]>('fetch_goal_ids');  
      setGoalIds(result);
    }
    loadGoalIds();
  },[])

  return (
    <div style={{ width: '100%', height: '100vh' }}>
      {goalIds.map(goalId=>(
        <GraphContainer key={goalId} goalId={goalId}/>
      ))}
    </div>
  );
}