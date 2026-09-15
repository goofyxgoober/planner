import type { Node as RFNode } from '@xyflow/react';

export interface Goal {
  id: string;
  nodeId: string;
  title: string;
  targetDate: string;
  status: string;
  version: number;
}

export interface Habit {
  id: string;
  nodeId: string;
  goalId: string | null;
  title: string;
  frequency: string;
  targetTime: number;
  streakCount: number;
}

export interface Task {
  id: string;
  nodeId: string;
  goalId: string | null;
  eventContextId: string | null;
  title: string;
  taskType: string;
  baseDuration: number;
  scheduleStart: string;
  scheduleEnd: string;
  urgencyScore: number;
  importanceScore: number;
  priorityWeight: number;
  status: string;
}

export type NodeType = 'HABIT' | 'GOAL' | 'TASK';

// Discriminated union instead of a single Node with `item: Task | Habit | Goal`.
// This is the actual fix: with the old shape, `nodeType: 'GOAL'` didn't guarantee
// `item: Goal` to the type checker, which is why toRFNode needed `as` casts.
// This shape makes that connection real, so narrowing on nodeType narrows item too.
export type GraphNode =
  | { id: string; nodeType: 'GOAL'; x: number | null; y: number | null; item: Goal }
  | { id: string; nodeType: 'HABIT'; x: number | null; y: number | null; item: Habit }
  | { id: string; nodeType: 'TASK'; x: number | null; y: number | null; item: Task };

export type GoalNodeData = { item: Goal };
export type HabitNodeData = { item: Habit };
export type TaskNodeData = { item: Task };

export type AppNode =
  | RFNode<GoalNodeData, 'GOAL'>
  | RFNode<HabitNodeData, 'HABIT'>
  | RFNode<TaskNodeData, 'TASK'>;

export function toRFNode(n: GraphNode): AppNode {
  const position = { x: n.x ?? 0, y: n.y ?? 0 };
  switch (n.nodeType) {
    case 'GOAL':
      return { id: n.id, type: 'GOAL', position, data: { item: n.item } };
    case 'HABIT':
      return { id: n.id, type: 'HABIT', position, data: { item: n.item } };
    case 'TASK':
      return { id: n.id, type: 'TASK', position, data: { item: n.item } };
  }
}

export interface Snapshot {
  nodes: Record<string, GraphNode>;
  successors: Record<string, string[]>;
  predecessors: Record<string, string[]>;
}

export type SnapshotVersion = {
  snapshot: Snapshot;
  version: number;
};

export type Op =
  | { type: 'add_task'; task: Task; x: number | null; y: number | null }
  | { type: 'add_habit'; habit: Habit; x: number | null; y: number | null }
  | { type: 'add_goal'; goal: Goal; x: number | null; y: number | null }
  | { type: 'remove_node'; id: string }
  | { type: 'add_edge'; predecessorId: string; successorId: string }
  | { type: 'remove_edge'; predecessorId: string; successorId: string }
  | { type: 'move_node'; id: string; x: number; y: number }
  | { type: 'modify_node'; id: string; jsonStr: string }
  | { type: 'batch'; ops: Array<Op> };