-- ============================================================
-- DAG seed data — goal as root
-- ============================================================
-- Your `task_dependencies` table's FKs only allow `tasks(id)`
-- on both predecessor_id and successor_id, so a goal can't
-- literally appear as a row in that edge table without a
-- schema change. Instead, this models the goal as root the
-- way your app logic expects it: every task hangs off a goal
-- via `tasks.goal_id`, and the "root task(s)" under that goal
-- are simply the ones with no incoming task_dependencies edge.
-- Your traversal starts at the goal, then walks to whichever
-- tasks have goal_id = <goal> and no predecessor.
--
-- A node's id is the SAME value as the id of the goal/task/habit
-- it wraps (node is a thin wrapper, not a separately-keyed row).
--
-- NOTE: same assumption as last time on the `nodes` table shape:
-- CREATE TABLE nodes (
--     id TEXT PRIMARY KEY,
--     node_type TEXT NOT NULL,  -- "goal", "task", "habit"
--     x REAL,
--     y REAL
-- );
-- ============================================================

-- --------------------------------------------------------------
-- Reset: clear out prior seed data, in FK-safe order
-- (children before parents, so nothing violates a constraint)
-- --------------------------------------------------------------
DELETE FROM taskDependencies;
DELETE FROM habits;
DELETE FROM tasks;
DELETE FROM goals;
DELETE FROM nodes;

-- --------------------------------------------------------------
-- Nodes: one per goal / task / habit below — id matches the
-- wrapped entity's own id exactly.
-- --------------------------------------------------------------
INSERT INTO nodes (id, nodeType, x, y) VALUES
  ('goal-1',  'goal',  NULL, NULL),
  ('task-1',  'task',  NULL, NULL),
  ('task-2',  'task',  NULL, NULL),
  ('task-3',  'task',  NULL, NULL),
  ('task-4',  'task',  NULL, NULL),
  ('habit-1', 'habit', NULL, NULL);

-- --------------------------------------------------------------
-- Goal — this is the root of the DAG
-- --------------------------------------------------------------
INSERT INTO goals (id, nodeId, title, targetDate, status, version) VALUES
  ('goal-1', 'goal-1', 'Pass Calculus with an A', '2026-12-15', 'active', 0);

-- --------------------------------------------------------------
-- Tasks, all scoped to goal-1 via goalId.
--
-- task-1 has NO incoming taskDependencies edge, so it's the
-- single root task directly under the goal:
--
--   goal-1 (ROOT)
--     └── task-1 (Read Chapter 3)
--           ├── task-2 (Problem Set)
--           └── task-3 (Study Session)
--                 └──┬── task-4 (Take Midterm Exam)
--                    (task-2 also feeds task-4)
-- --------------------------------------------------------------
INSERT INTO tasks (
  id, nodeId, goalId, eventContextId, title, taskType,
  baseDuration, scheduledStart, scheduledEnd,
  urgencyScore, importanceScore, priorityWeight, status
) VALUES
  ('task-1', 'task-1', 'goal-1', NULL, 'Read Chapter 3',        'study', 45, NULL, NULL, 0.3, 0.6, 0.45, 'pending'),
  ('task-2', 'task-2', 'goal-1', NULL, 'Complete Problem Set',  'study', 60, NULL, NULL, 0.4, 0.7, 0.55, 'pending'),
  ('task-3', 'task-3', 'goal-1', NULL, 'Group Study Session',   'study', 90, NULL, NULL, 0.4, 0.6, 0.50, 'pending'),
  ('task-4', 'task-4', 'goal-1', NULL, 'Take Midterm Exam',     'exam',  120, NULL, NULL, 0.9, 1.0, 0.95, 'pending');

-- --------------------------------------------------------------
-- Task Dependencies (DAG edges) — task-1 has no predecessor,
-- so it's the root task reachable directly from goal-1.
-- --------------------------------------------------------------
INSERT INTO taskDependencies (predecessorId, successorId, goalId) VALUES
  ('task-1', 'task-2', 'goal-1'),
  ('task-1', 'task-3', 'goal-1'),
  ('task-2', 'task-4', 'goal-1'),
  ('task-3', 'task-4', 'goal-1'),
  ('goal-1', 'task-1', 'goal-1');

-- --------------------------------------------------------------
-- Habit: recurring template feeding into the same goal
-- --------------------------------------------------------------
INSERT INTO habits (id, nodeId, goalId, title, frequency, targetTime, streakCount) VALUES
  ('habit-1', 'habit-1', 'goal-1', 'Daily Practice Problems', 'daily', 20, 0);