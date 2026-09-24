import React, { useState, useEffect } from "react";
//import type { Node, Edge } from "@xyflow/react";
//import type { Snapshot, SnapshotVersion } from "../types";
import { GraphView } from "../graph/GraphView";
import { graphStore } from "../state/graphStore";
import { historyStore } from "../state/historyStore";
import { useStore } from "zustand";

interface GraphContainerProps {
  goalId:string,
  //loading?: boolean;
}

export const GraphContainer: React.FC<GraphContainerProps> = ({
  goalId,
  //loading=false,
}) => {
  const [loading, setLoading] = useState<boolean>(true);
  const [loadGraph,proposeOp] = useStore(graphStore,(state)=>[state.loadGraph,state.proposeOp]);
  const [snapshot]= useStore(graphStore,(state)=>[state.graphs[goalId]])
  const [opQueue] = useStore(historyStore,(state)=>[state.pendingOps[goalId] ?? []]);
  const [popFromPendingOp,pushUndoStack] = useStore(historyStore,(state)=>[state.popFromPendingOp,state.pushUndoStack]);
  //const graphErrors = useStore(graphStore, (state)=>state.errors[goalId]);
  
  useEffect(() => {
    console.log(`Started loading graph`)
    async function load() {
      setLoading(true);
      await loadGraph(goalId); 
      setLoading(false);
    }

    load();
    console.log(`Loaded Graph Successfully`)
  }, []);


  useEffect(() => {
    if(opQueue.length===0)
      return; 
    const queueHead = popFromPendingOp(goalId);
    if(queueHead===undefined)
      return;
    console.log("Operation triggered");
    proposeOp(goalId,queueHead)
      .then(inverseOp=>{

        if (inverseOp === undefined) return;
        pushUndoStack(goalId,{op:queueHead,inverseOp:inverseOp});
      })
      .catch(error=>{
        //handle error ig
        console.log("Error proposing op:",error);
      });
  
  }, [opQueue]);

  if(snapshot===undefined){
    return (<div>loading...</div>)
  }
  return (
    <GraphView
      snapshot={snapshot}
      goalId={goalId}
      loading={loading}
    />
  );
};


