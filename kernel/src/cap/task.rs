//! Per-task capability tables (0.3.3).

use super::space::CapSpace;

pub const MAX_TASKS: usize = 8;
pub const NAME_LEN: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId(pub usize);

struct Task {
    alive: bool,
    name: [u8; NAME_LEN],
    name_len: usize,
    cspace: CapSpace,
}

impl Task {
    const fn empty() -> Self {
        Self {
            alive: false,
            name: [0; NAME_LEN],
            name_len: 0,
            cspace: CapSpace::new(),
        }
    }
}

pub struct TaskTable {
    tasks: [Task; MAX_TASKS],
}

impl TaskTable {
    pub const fn new() -> Self {
        Self {
            tasks: [const { Task::empty() }; MAX_TASKS],
        }
    }

    /*
     * spawn - allocate a task slot with an empty CapSpace
     * @name: short debug label (truncated to NAME_LEN)
     */
    pub fn spawn(&mut self, name: &str) -> Option<TaskId> {
        let idx = self.tasks.iter().position(|t| !t.alive)?;
        let bytes = name.as_bytes();
        let n = bytes.len().min(NAME_LEN);
        let mut buf = [0u8; NAME_LEN];
        buf[..n].copy_from_slice(&bytes[..n]);
        self.tasks[idx] = Task {
            alive: true,
            name: buf,
            name_len: n,
            cspace: CapSpace::new(),
        };
        Some(TaskId(idx))
    }

    pub fn name(&self, id: TaskId) -> &str {
        let t = &self.tasks[id.0];
        core::str::from_utf8(&t.name[..t.name_len]).unwrap_or("?")
    }

    pub fn cspace(&self, id: TaskId) -> Option<&CapSpace> {
        self.tasks.get(id.0).filter(|t| t.alive).map(|t| &t.cspace)
    }

    pub fn cspace_mut(&mut self, id: TaskId) -> Option<&mut CapSpace> {
        self.tasks
            .get_mut(id.0)
            .filter(|t| t.alive)
            .map(|t| &mut t.cspace)
    }

    pub fn alive_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.alive).count()
    }
}

impl Default for TaskTable {
    fn default() -> Self {
        Self::new()
    }
}
