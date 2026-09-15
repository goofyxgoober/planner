import { Handle, Position } from '@xyflow/react';
import type {NodeProps} from '@xyflow/react';
import ArrowCircleRightIcon from '@mui/icons-material/ArrowCircleRight';
import type { AppNode, GoalNodeData, HabitNodeData, TaskNodeData } from '../types';

function GoalNode({ data }: NodeProps<Extract<AppNode, { type: 'GOAL' }>>) {
  const { title, targetDate, status } = data.item;
  return (
    <div className="rounded-lg border-2 border-purple-500 bg-purple-50 px-4 py-2 shadow-md">
      <div className="font-bold text-purple-900">{title}</div>
      <div className="text-xs text-purple-700">Due {targetDate}</div>
      <div className="text-xs text-purple-700">{status}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}

function HabitNode({ data }: NodeProps<Extract<AppNode, { type: 'HABIT' }>>) {
  const { title, frequency, streakCount } = data.item;
  return (
    <div className="rounded-lg border-2 border-green-500 bg-green-50 px-4 py-2 shadow-md">
      <Handle type="target" position={Position.Top} />
      <div className="font-bold text-green-900">{title}</div>
      <div className="text-xs text-green-700">{frequency} · streak {streakCount}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}

function TaskNode({ data }: NodeProps<Extract<AppNode, { type: 'TASK' }>>) {
  const { title, status, priorityWeight } = data.item;
  return (
    <div className="rounded-lg border-2 border-blue-500 bg-blue-50 px-4 py-2 shadow-md">
      <Handle type="target" position={Position.Top} />
      <div className="font-bold text-blue-900">{title}</div>
      <div className="text-xs text-blue-700">{status} · P{priorityWeight.toFixed(1)}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}