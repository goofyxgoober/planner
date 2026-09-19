import React, { useCallback, useState } from "react";
import {
  ReactFlow,
  ReactFlowProvider,
  applyNodeChanges,
  applyEdgeChanges,
  addEdge,
} from "@xyflow/react";
import type {
  Node,
  Edge,
  NodeChange,
  EdgeChange,
  Connection,

} from "@xyflow/react";
import DagreNodePositioning, {
  type DagreGraphOptions,
} from "../layout/DagreNodePositioning";
import "@xyflow/react/dist/style.css";

import {GoalNode,HabitNode,TaskNode} from "./Nodes"
import type {SnapshotVersion, AppNode } from "../types";
import { historyStore } from "../state/historyStore";

import {useStore} from "zustand";

import type {Op} from "../types";
interface GraphViewProps {
  snapshot: SnapshotVersion;
  goalId: string,
  loading?: boolean;
  layoutOptions?: DagreGraphOptions;
}

const nodeTypes = {
  goalNode: GoalNode, 
  taskNode: TaskNode, 
  habitNode: HabitNode 
}

const defaultLayoutOptions: DagreGraphOptions = {
  rankdir: "TB",
  nodesep: 50,
  ranksep: 50,
};

function snapshotToFlow({snapshot,}: SnapshotVersion): [ Node[], Edge[] ] {
  const nodes: Node[] = Object.values(snapshot.nodes).map((n) => {
    const hasSavedPosition = typeof n.x === "number" && typeof n.y === "number";
    const renderNodeType = ({'GOAL':'goalType','TASK':'taskType','HABIT':'habitType'})[n.nodeType];
    return {
      id: n.id,
      position: { x: n.x ?? 0, y: n.y ?? 0 },
      type: renderNodeType,
      data: { 
        label: n.item.title,
        item: n.item,
        layouted: hasSavedPosition 
      } as AppNode['data'],
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

  return [ nodes, edges ];
}



export const GraphView: React.FC<GraphViewProps> = ({
  snapshot,
  goalId,
  loading = false,
  layoutOptions = defaultLayoutOptions,
}) => {
  const [, setViewIsFit] = useState<boolean>(false);
  const [initialNodes,initialEdges] = snapshotToFlow(snapshot);
  const [nodes, setNodes] = useState<Node[]>(initialNodes);
  const [edges, setEdges] = useState<Edge[]>(initialEdges);
  const [pushToPendingOp] = useStore(historyStore, (state) => [state.pushToPendingOp]);


  const onNodesDragStop = useCallback(
    (_: MouseEvent | TouchEvent, node: Node)=>{
      const nodeId:string = (typeof node.data.id === "string") ? node.data.id : "";
      if(nodeId==="")
        return;
      const [x,y] = [node.position.x,node.position.y]
      const op:Op = {type:"move_node",id:nodeId,x:x,y:y};
      pushToPendingOp(goalId,op);
    },
    [pushToPendingOp]
  )
  const onNodesChange = useCallback(
    (changes: NodeChange[]) =>{
      setNodes((nodesSnapshot) => applyNodeChanges(changes, nodesSnapshot));
      
    },  
    [setNodes]
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) =>
      setEdges((edgesSnapshot) => applyEdgeChanges(changes, edgesSnapshot)),
    [setEdges]
  );

  const onConnect = useCallback(
    (params: Connection) =>
      setEdges((edgesSnapshot) => addEdge(params, edgesSnapshot)),
    [setEdges]
  );

  return (
    <div style={{ width: "100vw", height: "100vh", position: "relative" }}>
      {loading && (
        <div style={{ position: "absolute", zIndex: 1, padding: 8 }}>
          Loading…
        </div>
      )}
      <ReactFlowProvider>
        <DagreNodePositioning
          Options={layoutOptions}
          SetNodes={setNodes}
          SetEdges={setEdges}
          Edges={edges}
          SetViewIsFit={setViewIsFit}
        />
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeDragStop={onNodesDragStop}
          fitView
        />
      </ReactFlowProvider>
    </div>
  );
};