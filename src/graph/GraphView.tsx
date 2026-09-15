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

import type { Snapshot, SnapshotVersion, AppNode, GraphNode } from "../types";
 

interface GraphViewProps {
  snapshot: SnapshotVersion;
  loading?: boolean;
  layoutOptions?: DagreGraphOptions;
}



const defaultLayoutOptions: DagreGraphOptions = {
  rankdir: "TB",
  nodesep: 50,
  ranksep: 50,
};

function snapshotToFlow({snapshot,}: SnapshotVersion): [ Node[], Edge[] ] {
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
  loading = false,
  layoutOptions = defaultLayoutOptions,
}) => {
  const [, setViewIsFit] = useState<boolean>(false);
  const [initialNodes,initialEdges] = snapshotToFlow(snapshot);
  const [nodes, setNodes] = useState<Node[]>(initialNodes);
  const [edges, setEdges] = useState<Edge[]>(initialEdges);




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
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          fitView
        />
      </ReactFlowProvider>
    </div>
  );
};