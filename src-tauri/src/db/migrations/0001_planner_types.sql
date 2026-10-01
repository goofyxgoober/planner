-- 1. Locations (GPS & Addresses)
CREATE TABLE locations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,                -- e.g., "Home", "Campus Library"
    address TEXT NOT NULL,             -- Formatted street address
    latitude REAL NOT NULL,
    longitude REAL NOT NULL
);

-- 2. Event Contexts (Classes, Work, or Fixed Commitments)
CREATE TABLE eventContexts (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,               -- e.g., "Calculus I Lecture"
    category TEXT NOT NULL,            -- e.g., "class", "work", "appointment"
    startLocationId TEXT,
    endLocationId TEXT,
    recurringRule TEXT,                -- e.g., "FREQ=WEEKLY;BYDAY=MO,WE,FR"
    FOREIGN KEY (startLocationId) REFERENCES locations(id),
    FOREIGN KEY (endLocationId) REFERENCES locations(id)
);


CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    nodeType TEXT NOT NULL,  -- "goal", "task", "habit"
    x REAL,
    y REAL
);


-- 3. Goals (Top-level aspirations)
CREATE TABLE goals (
    id TEXT PRIMARY KEY,
    nodeId TEXT,
    title TEXT NOT NULL,               -- e.g., "Pass Calculus with an A"
    targetDate TEXT NOT NULL,          -- ISO8601 date string
    status TEXT NOT NULL,               -- "active", "completed", "archived"
    version INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (nodeId) REFERENCES nodes(id) ON DELETE CASCADE
);

-- 4. Tasks (Actionable items, commutes, and checkpoints)
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    nodeId TEXT,
    goalId TEXT,
    eventContextId TEXT,
    title TEXT NOT NULL,
    taskType TEXT NOT NULL,            -- "standard", "commute", "study", "exam"
    baseDuration INTEGER NOT NULL,     -- Estimated minutes
    scheduledStart TEXT,               -- ISO8601 timestamp
    scheduledEnd TEXT,                 -- ISO8601 timestamp
    urgencyScore REAL DEFAULT 0.0,
    importanceScore REAL DEFAULT 0.0,
    priorityWeight REAL DEFAULT 0.0,
    status TEXT NOT NULL,              -- "pending", "completed", "postponed"
    FOREIGN KEY (goalId) REFERENCES goals(id),
    FOREIGN KEY (eventContextId) REFERENCES eventContexts(id) ON DELETE SET NULL
    FOREIGN KEY (nodeId) REFERENCES nodes(id) ON DELETE CASCADE
);

-- 5. Task Dependencies (DAG Edges)
CREATE TABLE taskDependencies (
    predecessorId TEXT NOT NULL,       -- Must finish first
    successorId TEXT NOT NULL,         -- Depends on predecessor
    goalId TEXT NOT NULL,
    PRIMARY KEY (predecessorId, successorId),
    FOREIGN KEY (predecessorId) REFERENCES nodes(id),
    FOREIGN KEY (successorId) REFERENCES nodes(id),
    FOREIGN KEY (goalId) REFERENCES goals(id)
);

-- 6. Habits (Templates that generate recurring tasks)
CREATE TABLE habits (
    id TEXT PRIMARY KEY,
    nodeId TEXT,
    goalId TEXT,
    title TEXT NOT NULL,
    frequency TEXT NOT NULL,           -- e.g., "daily", "weekly"
    targetTime INTEGER NOT NULL,       -- Duration in minutes
    streakCount INTEGER DEFAULT 0,
    FOREIGN KEY (goalId) REFERENCES goals(id) ON DELETE SET NULL
    FOREIGN KEY (nodeId) REFERENCES nodes(id) ON DELETE CASCADE

);

-- 7. Task Feedback (Adaptive learning loop data)
CREATE TABLE taskFeedback (
    id TEXT PRIMARY KEY,
    taskId TEXT NOT NULL,
    estimatedDuration INTEGER NOT NULL,
    actualDuration INTEGER NOT NULL,
    userSentiment TEXT,                -- "easy", "just_right", "struggled", "overdue"
    completionQuality INTEGER,         -- Scale 1-5 evaluated via SLM
    createdAt TEXT NOT NULL,
    FOREIGN KEY (taskId) REFERENCES tasks(id)
);

-- 8. User Ability Profile (Bayesian multipliers for time estimation)
CREATE TABLE userAbilityProfiles (
    category TEXT PRIMARY KEY,         -- e.g., "coding", "writing", "math"
    velocityMultiplier REAL NOT NULL DEFAULT 1.0, -- Actual / Estimated ratio
    notes TEXT                         -- JSON or text summary updated by SLM coach
);

-- 9. Gradebook (Academic performance tracking)
CREATE TABLE gradeItems (
    id TEXT PRIMARY KEY,
    eventContextId TEXT NOT NULL,      -- Links to the specific class
    title TEXT NOT NULL,               -- e.g., "Midterm Exam"
    weight REAL NOT NULL,              -- Percentage weight (e.g., 0.25 for 25%)
    earnedScore REAL,                  -- Points earned
    maxScore REAL NOT NULL,            -- Total possible points
    targetGrade REAL,                  -- Desired score
    FOREIGN KEY (eventContextId) REFERENCES eventContexts(id)
);