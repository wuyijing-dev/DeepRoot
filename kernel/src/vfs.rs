//! Teaching VFS tree — directories + small files (1.9.0).
//!
//! In-RAM hierarchy rooted at `/`. Embed ramfs and DRFS still appear as
//! root-level siblings via [`crate::fs`]; this module owns user-created
//! directories and nested files (including `>` redirects into paths).

use core::cell::UnsafeCell;

use crate::sync::SpinLock;

pub const MAX_NODES: usize = 48;
pub const NAME_MAX: usize = 28;
pub const FILE_MAX: usize = 256;
pub const ROOT: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Dir,
    File,
}

pub struct Node {
    pub used: bool,
    pub kind: Kind,
    pub parent: usize,
    pub name_len: usize,
    pub name: [u8; NAME_MAX],
    pub data_len: usize,
    pub data: [u8; FILE_MAX],
}

impl Node {
    const fn empty() -> Self {
        Self {
            used: false,
            kind: Kind::Dir,
            parent: ROOT,
            name_len: 0,
            name: [0; NAME_MAX],
            data_len: 0,
            data: [0; FILE_MAX],
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

struct Table {
    nodes: [Node; MAX_NODES],
}

struct Cell(UnsafeCell<Table>);
unsafe impl Sync for Cell {}

static LOCK: SpinLock = SpinLock::new();
static TABLE: Cell = Cell(UnsafeCell::new(Table {
    nodes: [const { Node::empty() }; MAX_NODES],
}));

fn table() -> &'static mut Table {
    unsafe { &mut *TABLE.0.get() }
}

/*
 * init - create root directory node 0
 */
pub fn init() {
    let _g = LOCK.lock();
    let t = table();
    for n in t.nodes.iter_mut() {
        *n = Node::empty();
    }
    t.nodes[ROOT].used = true;
    t.nodes[ROOT].kind = Kind::Dir;
    t.nodes[ROOT].parent = ROOT;
    t.nodes[ROOT].name_len = 1;
    t.nodes[ROOT].name[0] = b'/';
}

fn alloc_node(t: &mut Table) -> Option<usize> {
    t.nodes.iter().position(|n| !n.used)
}

fn child_named(t: &Table, parent: usize, name: &str) -> Option<usize> {
    for (i, n) in t.nodes.iter().enumerate() {
        if n.used && n.parent == parent && i != ROOT && n.name_str() == name {
            return Some(i);
        }
    }
    None
}

fn has_children(t: &Table, dir: usize) -> bool {
    t.nodes
        .iter()
        .enumerate()
        .any(|(i, n)| n.used && n.parent == dir && i != ROOT)
}

/*
 * path_components - split absolute or relative path into name components
 *
 * Skips empty / `.` segments. Returns false if a component is too long or `..`
 * appears (caller resolves `..` while walking).
 */
fn walk(
    t: &mut Table,
    start: usize,
    path: &str,
    create_dirs: bool,
) -> Result<usize, ()> {
    let mut cur = start;
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return Ok(start);
    }

    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if cur != ROOT {
                cur = t.nodes[cur].parent;
            }
            continue;
        }
        if part.len() >= NAME_MAX {
            return Err(());
        }
        match child_named(t, cur, part) {
            Some(idx) => {
                cur = idx;
            }
            None => {
                if !create_dirs {
                    return Err(());
                }
                if t.nodes[cur].kind != Kind::Dir {
                    return Err(());
                }
                let idx = alloc_node(t).ok_or(())?;
                let n = &mut t.nodes[idx];
                *n = Node::empty();
                n.used = true;
                n.kind = Kind::Dir;
                n.parent = cur;
                n.name_len = part.len();
                n.name[..part.len()].copy_from_slice(part.as_bytes());
                cur = idx;
            }
        }
    }
    Ok(cur)
}

fn resolve_parent_and_name<'a>(
    t: &mut Table,
    cwd: usize,
    path: &'a str,
) -> Result<(usize, &'a str), ()> {
    let abs = path.starts_with('/');
    let start = if abs { ROOT } else { cwd };
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return Err(());
    }
    let (parent_path, name) = match path.rfind('/') {
        Some(i) => (&path[..i], &path[i + 1..]),
        None => ("", path),
    };
    if name.is_empty() || name == "." || name == ".." || name.len() >= NAME_MAX {
        return Err(());
    }
    let parent = if parent_path.is_empty() {
        start
    } else {
        walk(t, start, parent_path, false)?
    };
    if t.nodes[parent].kind != Kind::Dir {
        return Err(());
    }
    Ok((parent, name))
}

pub fn mkdir(cwd: usize, path: &str) -> bool {
    let _g = LOCK.lock();
    let t = table();
    let Ok((parent, name)) = resolve_parent_and_name(t, cwd, path) else {
        return false;
    };
    if child_named(t, parent, name).is_some() {
        return false;
    }
    let Some(idx) = alloc_node(t) else {
        return false;
    };
    let n = &mut t.nodes[idx];
    *n = Node::empty();
    n.used = true;
    n.kind = Kind::Dir;
    n.parent = parent;
    n.name_len = name.len();
    n.name[..name.len()].copy_from_slice(name.as_bytes());
    true
}

pub fn rmdir(cwd: usize, path: &str) -> bool {
    let _g = LOCK.lock();
    let t = table();
    let abs = path.starts_with('/');
    let start = if abs { ROOT } else { cwd };
    let Ok(idx) = walk(t, start, path.trim_start_matches('/'), false) else {
        return false;
    };
    if idx == ROOT || t.nodes[idx].kind != Kind::Dir || has_children(t, idx) {
        return false;
    }
    t.nodes[idx] = Node::empty();
    true
}

pub fn unlink(cwd: usize, path: &str) -> bool {
    let _g = LOCK.lock();
    let t = table();
    let abs = path.starts_with('/');
    let start = if abs { ROOT } else { cwd };
    let Ok(idx) = walk(t, start, path.trim_start_matches('/'), false) else {
        return false;
    };
    if idx == ROOT || t.nodes[idx].kind != Kind::File {
        return false;
    }
    t.nodes[idx] = Node::empty();
    true
}

pub fn chdir(cwd: usize, path: &str) -> Option<usize> {
    let _g = LOCK.lock();
    let t = table();
    let abs = path.starts_with('/');
    let start = if abs { ROOT } else { cwd };
    let trimmed = path.trim_start_matches('/');
    let idx = if trimmed.is_empty() && abs {
        ROOT
    } else {
        walk(t, start, trimmed, false).ok()?
    };
    if t.nodes[idx].kind != Kind::Dir {
        return None;
    }
    Some(idx)
}

pub fn getcwd(cwd: usize, out: &mut [u8]) -> usize {
    let _g = LOCK.lock();
    let t = table();
    if cwd >= MAX_NODES || !t.nodes[cwd].used {
        if !out.is_empty() {
            out[0] = b'/';
            return 1;
        }
        return 0;
    }
    /* Build path by walking to root. */
    let mut stack: [&str; MAX_NODES] = [""; MAX_NODES];
    let mut depth = 0usize;
    let mut cur = cwd;
    while cur != ROOT && depth < MAX_NODES {
        stack[depth] = t.nodes[cur].name_str();
        depth += 1;
        cur = t.nodes[cur].parent;
    }
    let mut n = 0usize;
    if depth == 0 {
        if !out.is_empty() {
            out[0] = b'/';
            return 1;
        }
        return 0;
    }
    for i in (0..depth).rev() {
        if n >= out.len() {
            break;
        }
        out[n] = b'/';
        n += 1;
        let s = stack[i].as_bytes();
        let take = s.len().min(out.len().saturating_sub(n));
        out[n..n + take].copy_from_slice(&s[..take]);
        n += take;
    }
    n
}

pub fn resolve(cwd: usize, path: &str) -> Option<(usize, Kind)> {
    let _g = LOCK.lock();
    let t = table();
    let abs = path.starts_with('/');
    let start = if abs { ROOT } else { cwd };
    let trimmed = path.trim_start_matches('/');
    let idx = if trimmed.is_empty() {
        if abs {
            ROOT
        } else {
            cwd
        }
    } else {
        walk(t, start, trimmed, false).ok()?
    };
    if !t.nodes[idx].used {
        return None;
    }
    Some((idx, t.nodes[idx].kind))
}

pub fn write_file(cwd: usize, path: &str, data: &[u8]) -> bool {
    let _g = LOCK.lock();
    let t = table();
    let Ok((parent, name)) = resolve_parent_and_name(t, cwd, path) else {
        return false;
    };
    let ncopy = data.len().min(FILE_MAX);
    if let Some(idx) = child_named(t, parent, name) {
        if t.nodes[idx].kind != Kind::File {
            return false;
        }
        t.nodes[idx].data_len = ncopy;
        t.nodes[idx].data[..ncopy].copy_from_slice(&data[..ncopy]);
        return true;
    }
    let Some(idx) = alloc_node(t) else {
        return false;
    };
    let n = &mut t.nodes[idx];
    *n = Node::empty();
    n.used = true;
    n.kind = Kind::File;
    n.parent = parent;
    n.name_len = name.len();
    n.name[..name.len()].copy_from_slice(name.as_bytes());
    n.data_len = ncopy;
    n.data[..ncopy].copy_from_slice(&data[..ncopy]);
    true
}

pub fn read_file(cwd: usize, path: &str, out: &mut [u8]) -> Option<usize> {
    let _g = LOCK.lock();
    let t = table();
    let abs = path.starts_with('/');
    let start = if abs { ROOT } else { cwd };
    let trimmed = path.trim_start_matches('/');
    let idx = walk(t, start, trimmed, false).ok()?;
    if t.nodes[idx].kind != Kind::File {
        return None;
    }
    let n = t.nodes[idx].data_len.min(out.len());
    out[..n].copy_from_slice(&t.nodes[idx].data[..n]);
    Some(n)
}

/*
 * list_dir - invoke callback for each child name under @dir
 */
pub fn list_dir(dir: usize, mut f: impl FnMut(&str, Kind, usize)) {
    let _g = LOCK.lock();
    let t = table();
    if dir >= MAX_NODES || !t.nodes[dir].used || t.nodes[dir].kind != Kind::Dir {
        return;
    }
    for (i, n) in t.nodes.iter().enumerate() {
        if n.used && n.parent == dir && i != ROOT {
            f(n.name_str(), n.kind, if n.kind == Kind::File { n.data_len } else { 0 });
        }
    }
}
