import React, { useState, useEffect } from "react";
import type { Node, Edge } from "@xyflow/react";
import { fetchSnapshot } from "../api/fetchSnapshot";
import type { Snapshot, SnapshotVersion } from "../types";
import { GraphView } from "../graph/GraphView";
import { graphStore } from "../state/graphStore";
import { useStore } from "zustand";

const initialNodes: Node[] = [
  { id: "n1", position: { x: 0, y: 0 }, data: { label: "Node 1" } },
  { id: "n2", position: { x: 0, y: 100 }, data: { label: "Node 2" } },
];

const initialEdges: Edge[] = [
  { id: "n1-n2", source: "n1", target: "n2" }
];

const GOAL_ID = "goal-1";

function snapshotToFlow({snapshot,}: SnapshotVersion): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = Object.values(snapshot.nodes).map((n) => {
    // 1. Safely check if the API provided actual numbers for x and y
    // We use typeof instead of truthiness so we don't accidentally ignore a valid 0 coordinate
    const hasSavedPosition = typeof n.x === "number" && typeof n.y === "number";

    return {
      id: n.id,
      position: { x: n.x ?? 0, y: n.y ?? 0 },
      data: { 
        label: n.item.title, 
        nodeType: n.nodeType,
        // 2. Pass the flag directly into the node's data payload
        layouted: hasSavedPosition 
      },
    };
  });

  const edges: Edge[] = Object.entries(snapshot.successors).flatMap(
    ([sourceId, targetIds]) =>
      targetIds.map((targetId) => ({
        id: `${sourceId}-${targetId}`,
        source: sourceId,
        target: targetId,
      }))
  );

  return { nodes, edges };
}


interface GraphContainerProps {
  goalId:string,
  //loading?: boolean;
}

export const GraphContainer: React.FC<GraphContainerProps> = ({
  goalId,
  //loading=false,
}) => {
  const [loading, setLoading] = useState<boolean>(true);
  const [loadGraph,getGraph] = useStore(graphStore,(state)=>[state.loadGraph,state.getGraph]);
  const [snapshot,setSnapshot] =useState<SnapshotVersion>();
  const graphErrors = useStore(graphStore, (state)=>state.errors[goalId]);
  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      await loadGraph(goalId); 
      const snapshotVer = getGraph(goalId);
      if(!snapshotVer || graphErrors){
        cancelled=true;
      }
      if (cancelled) return;
      setSnapshot(snapshotVer);
      setLoading(false);
    }

    load();
    return () => {
      cancelled = true;
    };
  }, []);

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


