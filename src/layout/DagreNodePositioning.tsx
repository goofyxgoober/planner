import React, { useEffect } from "react";
import {
  useReactFlow,
  useNodesInitialized,
  Position,
  type Node,
  type Edge,
} from "@xyflow/react";
import dagre from "@dagrejs/dagre";
import { graphStore } from "../state/graphStore";
import { historyStore } from "../state/historyStore";
import { useStore } from "zustand";
export interface DagreGraphOptions {
  rankdir?: "TB" | "BT" | "LR" | "RL";
  nodesep?: number;
  ranksep?: number;
  [key: string]: unknown;
}

import type {Op} from "../types";
interface DagreNodePositioningProps {
  goalId:string;
  Options: DagreGraphOptions;
  SetNodes: React.Dispatch<React.SetStateAction<Node[]>>;
  SetEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
  Edges: Edge[];
  SetViewIsFit: (isFit: boolean) => void;
}

const DagreNodePositioning: React.FC<DagreNodePositioningProps> = ({
  goalId,
  Options,
  SetNodes,
  SetEdges,
  SetViewIsFit,
}) => {
  const { fitView, getNodes, getEdges } = useReactFlow();
  const [pushToPendingOp] = useStore(historyStore, (state) => [state.pushToPendingOp]);

  // This hook turns true ONLY when all nodes have been rendered and measured in the DOM
  const nodesInitialized = useNodesInitialized();

  useEffect(() => {
    if (!nodesInitialized) return;

    const currentNodes = getNodes();
    const currentEdges = getEdges();

    if (currentNodes.length === 0) return;

    // Check if the current nodes have already been laid out to prevent infinite loops
    const isLayouted = currentNodes.every((n) => n.data?.layouted === true);
    if (isLayouted) return;

    const dagreGraph = new dagre.graphlib.Graph();
    dagreGraph.setDefaultEdgeLabel(() => ({}));
    dagreGraph.setGraph(Options);

    // Feed dimensions to Dagre
    currentNodes.forEach((node) => {
      const width = node.measured?.width ?? node.width ?? 150;
      const height = node.measured?.height ?? node.height ?? 50;
      dagreGraph.setNode(node.id, { width, height });
    });

    currentEdges.forEach((edge) => {
      dagreGraph.setEdge(edge.source, edge.target);
    });

    dagre.layout(dagreGraph);

    // Map Dagre's calculated positions back to React Flow nodes
    const layoutedNodes = currentNodes.map((node) => {
      const nodeWithPosition = dagreGraph.node(node.id);
      const width = node.measured?.width ?? node.width ?? 150;
      const height = node.measured?.height ?? node.height ?? 50;

      let targetPosition = Position.Top;
      let sourcePosition = Position.Bottom;

      switch (Options.rankdir) {
        case "BT":
          targetPosition = Position.Bottom;
          sourcePosition = Position.Top;
          break;
        case "LR":
          targetPosition = Position.Left;
          sourcePosition = Position.Right;
          break;
        case "RL":
          targetPosition = Position.Right;
          sourcePosition = Position.Left;
          break;
      }

      return {
        ...node,
        // Mark node as layouted so we don't recalculate it on the next render
        data: { ...node.data, layouted: true },
        position: {
          x: nodeWithPosition.x - width / 2,
          y: nodeWithPosition.y - height / 2,
        },
        targetPosition,
        sourcePosition,
      };
    });

    let opList:Array<Op> = [];
    layoutedNodes.forEach((node)=>{
      const nodeId = node.id;
      const [x,y] = [node.position.x,node.position.y]
      const op:Op = {type:"move_node",id:nodeId,x:x,y:y};
      opList.push(op);
    })
    pushToPendingOp(goalId,{type:"batch",ops:opList});
    SetNodes(layoutedNodes);
    SetEdges(currentEdges);

    window.requestAnimationFrame(() => {
      fitView({ duration: 800, padding: 0.2 });
      SetViewIsFit(true);
    });
    
  }, [
    nodesInitialized,
    Options,
    SetNodes,
    SetEdges,
    SetViewIsFit,
    fitView,
    getNodes,
    getEdges,
  ]);

  return null;
};

export default DagreNodePositioning;