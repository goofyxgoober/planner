import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
//import ArrowCircleRightIcon from '@mui/icons-material/ArrowCircleRight';
import { historyStore } from '../state/historyStore';
import { useStore } from "zustand";
import type { AppNode, Op, Goal, Habit, Task } from '../types';




/*

  How do I handle this????
  Need to make a general FormElements that can take an unknown type of 

*/
interface GoalElements extends HTMLFormControlsCollection {
  title: HTMLInputElement,
  due: HTMLInputElement,
  status: HTMLInputElement
}

interface TaskElements extends HTMLFormControlsCollection {
  //todo
  title: HTMLInputElement,
  status: HTMLInputElement,
  priorityWeight: HTMLInputElement
}

interface HabitElements extends HTMLFormControlsCollection {
  //todo
  title: HTMLInputElement,
  frequency: HTMLInputElement,
  streakCount: HTMLInputElement
}

type NodeElements = GoalElements | TaskElements | HabitElements;

interface NodeFormElement<Type extends NodeElements> extends HTMLFormElement {
  readonly elements: Type
}



export function GoalNode({ data }: NodeProps<Extract<AppNode, { type: 'GOAL' }>>) {
  const { id, title, targetDate, status } = data.item;
  const [pushToPendingOp] = useStore(historyStore, (state) => [state.pushToPendingOp]);

  function constructGoalModifyOp(element: GoalElements) {
    let modified_goal: Goal = { ...data.item };
    modified_goal.title = element.title.value;
    modified_goal.targetDate = element.due.value;
    modified_goal.status = element.status.value;
    const op: Op = { type: "modify_node", id: id, jsonStr: JSON.stringify(modified_goal) };
    return op;
  }

  function handleSubmitGoal(event: React.SubmitEvent<NodeFormElement<GoalElements>>) {
    event.preventDefault();
    const op = constructGoalModifyOp(event.currentTarget.elements);
    pushToPendingOp(id, op);
  }

  return (
    <div className="rounded-lg border-2 border-purple-500 bg-purple-50 px-4 py-2 shadow-md">
      <div className="font-bold text-purple-900">{title}</div>
      <form onSubmit={handleSubmitGoal} className='nodrag'>
        <label htmlFor="title-goal">Title:</label>
        <input id="title-goal" className="text-xs text-purple-700"/>{title}
        <label htmlFor="due">Due:</label>
        <input id="due-goal" className="text-xs text-purple-700"/>{targetDate}
        <label htmlFor="status-goal">Status:</label>
        <input id="status-goal" className="text-xs text-purple-700"/>{status}
      </form>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}

export function HabitNode({ data }: NodeProps<Extract<AppNode, { type: 'HABIT' }>>) {
  const { id, goalId, title, frequency, streakCount } = data.item;
  const [pushToPendingOp] = useStore(historyStore, (state) => [state.pushToPendingOp]);

  function constructGoalModifyOp(element: HabitElements) {
    let modified_habit: Habit = { ...data.item };
    modified_habit.title = element.title.value;
    modified_habit.frequency = element.frequency.value;
    modified_habit.streakCount = element.streakCount.valueAsNumber;
    const op: Op = { type: "modify_node", id: id, jsonStr: JSON.stringify(modified_habit) };
    return op;
  }

  function handleSubmitHabit(event: React.SubmitEvent<NodeFormElement<HabitElements>>) {
    event.preventDefault();
    const op = constructGoalModifyOp(event.currentTarget.elements);
    if (goalId === null)
      return;
    pushToPendingOp(goalId, op);
  }


  return (
    <div className="rounded-lg border-2 border-green-500 bg-green-50 px-4 py-2 shadow-md">
      <Handle type="target" position={Position.Top} />
      <form onSubmit={handleSubmitHabit} className='nodrag'>
        <label htmlFor='title-habit'>Title:</label>
        <input id="title-habit" className="text-xs text-green-700"/>{title}
        <label htmlFor='frequency-habit'>Frequency:</label>
        <input id="frequency-habit" className="text-xs text-green-700"/>{frequency}
        <label htmlFor='streakCount-habit'>StreakCount:</label>
        <input id="streakCount-habit" className="text-xs text-green-700"/>{streakCount}
      </form>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}

export function TaskNode({ data }: NodeProps<Extract<AppNode, { type: 'TASK' }>>) {
  const { id, goalId, title, status, priorityWeight } = data.item;
  const [pushToPendingOp] = useStore(historyStore, (state) => [state.pushToPendingOp]);


  function constructGoalModifyOp(element: TaskElements) {
    let modified_task: Task = { ...data.item };
    modified_task.title = element.title.value;
    modified_task.status = element.status.value;
    modified_task.priorityWeight = element.priorityWeight.valueAsNumber;
    const op: Op = { type: "modify_node", id: id, jsonStr: JSON.stringify(modified_task) };
    return op;
  }

  function handleSubmitTask(event: React.SubmitEvent<NodeFormElement<TaskElements>>) {
    event.preventDefault();
    const op = constructGoalModifyOp(event.currentTarget.elements);
    if (goalId === null)
      return;
    pushToPendingOp(goalId, op);
  }


  return (
    <div className="rounded-lg border-2 border-blue-500 bg-blue-50 px-4 py-2 shadow-md">
      <Handle type="target" position={Position.Top} />
      <form onSubmit={handleSubmitTask} className='nodrag'>
        <label htmlFor='title-task'>Title:</label>
        <input id="title-task" className="text-xs text-green-700"/>{title}
        <label htmlFor='status-task'>Status:</label>
        <input id="status-task" className="text-xs text-green-700"/>{status}
        <label htmlFor='streakCount-task'>PriorityWeight:</label>
        <input id="streakCount-task" className="text-xs text-green-700"/>{priorityWeight}
      </form>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}