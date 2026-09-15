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
  const [loadGraph,getGraph,proposeOp] = useStore(graphStore,(state)=>[state.loadGraph,state.getGraph,state.proposeOp]);
  const [snapshot]= useStore(graphStore,(state)=>[state.graphs[goalId]])
  const [queueHead] = useStore(historyStore,(state)=>[(state.pendingOps[goalId] ?? [])[0] ]);
  const [popFromPendingOp,pushUndoStack] = useStore(historyStore,(state)=>[state.popFromPendingOp,state.pushUndoStack]);
  const graphErrors = useStore(graphStore, (state)=>state.errors[goalId]);
  
  useEffect(() => {
    async function load() {
      setLoading(true);
      await loadGraph(goalId); 
      setLoading(false);
    }

    load();
  }, []);


  useEffect(() => {
    
    if(queueHead===undefined)
      return; 

    proposeOp(goalId,queueHead)
      .then(inverseOp=>{
        if (inverseOp === undefined) return;
        const poppedOp = popFromPendingOp(goalId);
        if (poppedOp === undefined ) return;
        pushUndoStack(goalId,{op:queueHead,inverseOp:inverseOp});
      })
      .catch(error=>{
        //handle error ig
      });
  
  }, [queueHead]);

  if(snapshot===undefined){
    return (<div>loading...</div>)
  }
  return (
    <GraphView
      snapshot={snapshot}
      loading={loading}
    />
  );
};


