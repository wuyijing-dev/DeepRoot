//! shell — DeepRoot-native shell 1.10 (modload + 1.9 dirs + 1.8 argv/|/&).

#![no_std]
#![no_main]

use deeproot_user::sys;
use deeproot_user::sys::STDOUT_CONSOLE;

core::arch::global_asm!(
    r#"
    .section .text.entry, "ax"
    .globl _start
_start:
    la t0, __bss_start
    la t1, __bss_end
1:
    bgeu t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:
    call main
    li a0, 0
    li a7, 9
    ecall
3:
    wfi
    j 3b
"#
);

const LINE_MAX: usize = 96;
const HIST_MAX: usize = 16;
const ARG_MAX: usize = 8;
const ARG_LEN: usize = 32;
const ENV_MAX: usize = 8;
const PIPE_STAGES: usize = 3;

struct Env {
    keys: [[u8; 16]; ENV_MAX],
    key_len: [usize; ENV_MAX],
    vals: [[u8; 32]; ENV_MAX],
    val_len: [usize; ENV_MAX],
    used: [bool; ENV_MAX],
}

impl Env {
    const fn new() -> Self {
        Self {
            keys: [[0; 16]; ENV_MAX],
            key_len: [0; ENV_MAX],
            vals: [[0; 32]; ENV_MAX],
            val_len: [0; ENV_MAX],
            used: [false; ENV_MAX],
        }
    }

    fn set(&mut self, k: &[u8], v: &[u8]) -> bool {
        if k.is_empty() || k.len() >= 16 || v.len() >= 32 {
            return false;
        }
        for i in 0..ENV_MAX {
            if self.used[i] && &self.keys[i][..self.key_len[i]] == k {
                self.val_len[i] = v.len();
                self.vals[i][..v.len()].copy_from_slice(v);
                return true;
            }
        }
        for i in 0..ENV_MAX {
            if !self.used[i] {
                self.used[i] = true;
                self.key_len[i] = k.len();
                self.keys[i][..k.len()].copy_from_slice(k);
                self.val_len[i] = v.len();
                self.vals[i][..v.len()].copy_from_slice(v);
                return true;
            }
        }
        false
    }

    fn get<'a>(&'a self, k: &[u8]) -> Option<&'a [u8]> {
        for i in 0..ENV_MAX {
            if self.used[i] && &self.keys[i][..self.key_len[i]] == k {
                return Some(&self.vals[i][..self.val_len[i]]);
            }
        }
        None
    }

    fn list(&self) {
        for i in 0..ENV_MAX {
            if self.used[i] {
                let _ = sys::debug_write_bytes(&self.keys[i][..self.key_len[i]]);
                let _ = sys::debug_write("=");
                let _ = sys::debug_write_bytes(&self.vals[i][..self.val_len[i]]);
                let _ = sys::debug_write("\n");
            }
        }
    }
}

struct History {
    lines: [[u8; LINE_MAX]; HIST_MAX],
    lens: [usize; HIST_MAX],
    count: usize,
    next: usize,
}

impl History {
    const fn new() -> Self {
        Self {
            lines: [[0; LINE_MAX]; HIST_MAX],
            lens: [0; HIST_MAX],
            count: 0,
            next: 0,
        }
    }

    fn push(&mut self, line: &[u8]) {
        if line.is_empty() {
            return;
        }
        let n = line.len().min(LINE_MAX);
        let i = self.next % HIST_MAX;
        self.lines[i][..n].copy_from_slice(&line[..n]);
        self.lens[i] = n;
        self.next = self.next.wrapping_add(1);
        if self.count < HIST_MAX {
            self.count += 1;
        }
    }

    fn get(&self, back: usize) -> Option<&[u8]> {
        if back == 0 || back > self.count {
            return None;
        }
        let idx = self.next.wrapping_sub(back) % HIST_MAX;
        Some(&self.lines[idx][..self.lens[idx]])
    }

    fn dump(&self) {
        for b in (1..=self.count).rev() {
            if let Some(l) = self.get(b) {
                let _ = sys::debug_write("  ");
                let _ = sys::debug_write_bytes(l);
                let _ = sys::debug_write("\n");
            }
        }
    }
}

fn write_u32(mut n: u32) {
    let mut buf = [0u8; 10];
    let mut i = 10;
    if n == 0 {
        let _ = sys::debug_write("0");
        return;
    }
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    let _ = sys::debug_write_bytes(&buf[i..]);
}

fn parse_u64(s: &[u8]) -> Option<u64> {
    let s = trim(s);
    if s.is_empty() {
        return None;
    }
    let (s, base) = if s.len() > 2 && s[0] == b'0' && (s[1] == b'x' || s[1] == b'X') {
        (&s[2..], 16u64)
    } else {
        (s, 10u64)
    };
    let mut v = 0u64;
    for &c in s {
        let d = match c {
            b'0'..=b'9' => (c - b'0') as u64,
            b'a'..=b'f' => (c - b'a' + 10) as u64,
            b'A'..=b'F' => (c - b'A' + 10) as u64,
            _ => return None,
        };
        if d >= base {
            return None;
        }
        v = v.saturating_mul(base).saturating_add(d);
    }
    Some(v)
}


fn path_basename(path: &[u8]) -> &[u8] {
    match path.iter().rposition(|&b| b == b'/') {
        Some(i) => &path[i + 1..],
        None => path,
    }
}

fn default_badge(path: &[u8]) -> u64 {
    let b = path_basename(path);
    if b == b"modnote" || b == b"mynote" {
        0xD002
    } else if b == b"moddemo" {
        0xD001
    } else {
        0xD010
    }
}

fn do_modload(path: &[u8], badge: u64) {
    let slot = sys::spawn_server(path, badge);
    if slot < 0 {
        let _ = sys::debug_write("shell: modload failed (badge in use? try another badge)\n");
        return;
    }
    let _ = sys::debug_write("shell: modload ok slot=");
    write_u32(slot as u32);
    let _ = sys::debug_write("\n");
    let _ = sys::yield_now();
    let _ = sys::yield_now();
    let rc = sys::ipc_call(slot as usize, 0x4D44, 1);
    if rc >= 0 {
        let _ = sys::debug_write("shell: module call ok\n");
    } else {
        let _ = sys::debug_write("shell: module call failed\n");
    }
}

fn trim(mut line: &[u8]) -> &[u8] {
    while matches!(line.first().copied(), Some(b' ' | b'\t')) {
        line = &line[1..];
    }
    while matches!(line.last().copied(), Some(b' ' | b'\t')) {
        line = &line[..line.len() - 1];
    }
    line
}

/// Expand `$NAME` in place into `out` (best-effort, no nested).
fn expand_vars(env: &Env, src: &[u8], out: &mut [u8]) -> usize {
    let mut i = 0usize;
    let mut o = 0usize;
    while i < src.len() && o < out.len() {
        if src[i] == b'$' && i + 1 < src.len() {
            i += 1;
            let start = i;
            while i < src.len()
                && (src[i].is_ascii_alphanumeric() || src[i] == b'_')
            {
                i += 1;
            }
            if let Some(v) = env.get(&src[start..i]) {
                let n = v.len().min(out.len() - o);
                out[o..o + n].copy_from_slice(&v[..n]);
                o += n;
            }
        } else {
            out[o] = src[i];
            o += 1;
            i += 1;
        }
    }
    o
}

struct Argv {
    args: [[u8; ARG_LEN]; ARG_MAX],
    lens: [usize; ARG_MAX],
    n: usize,
}

impl Argv {
    const fn empty() -> Self {
        Self {
            args: [[0; ARG_LEN]; ARG_MAX],
            lens: [0; ARG_MAX],
            n: 0,
        }
    }

    fn push(&mut self, tok: &[u8]) -> bool {
        if self.n >= ARG_MAX || tok.is_empty() {
            return false;
        }
        let n = tok.len().min(ARG_LEN);
        self.args[self.n][..n].copy_from_slice(&tok[..n]);
        self.lens[self.n] = n;
        self.n += 1;
        true
    }

    fn get(&self, i: usize) -> Option<&[u8]> {
        if i < self.n {
            Some(&self.args[i][..self.lens[i]])
        } else {
            None
        }
    }
}

fn tokenize(line: &[u8], argv: &mut Argv) -> bool {
    *argv = Argv::empty();
    let mut i = 0usize;
    while i < line.len() {
        while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
            i += 1;
        }
        if i >= line.len() {
            break;
        }
        /* Operators are handled by split_pipeline; skip them here so a
         * one-shot tokenize of a full line (e.g. `echo x > f`) cannot spin. */
        if line[i] == b'|' || line[i] == b'>' || line[i] == b'<' || line[i] == b'&' {
            i += 1;
            continue;
        }
        let mut tok = [0u8; ARG_LEN];
        let mut tn = 0usize;
        if line[i] == b'"' || line[i] == b'\'' {
            let q = line[i];
            i += 1;
            while i < line.len() && line[i] != q {
                if tn < ARG_LEN {
                    tok[tn] = line[i];
                    tn += 1;
                }
                i += 1;
            }
            if i < line.len() {
                i += 1;
            }
        } else {
            while i < line.len()
                && line[i] != b' '
                && line[i] != b'\t'
                && line[i] != b'|'
                && line[i] != b'>'
                && line[i] != b'<'
                && line[i] != b'&'
            {
                if tn < ARG_LEN {
                    tok[tn] = line[i];
                    tn += 1;
                }
                i += 1;
            }
        }
        if tn > 0 && !argv.push(&tok[..tn]) {
            return false;
        }
    }
    true
}

struct Stage {
    argv: Argv,
    redir: Option<[u8; ARG_LEN]>,
    redir_len: usize,
    redir_append: bool,
}

impl Stage {
    const fn empty() -> Self {
        Self {
            argv: Argv::empty(),
            redir: None,
            redir_len: 0,
            redir_append: false,
        }
    }
}

fn split_pipeline(line: &[u8], stages: &mut [Stage], nstage: &mut usize, bg: &mut bool) -> bool {
    *nstage = 0;
    *bg = false;
    let mut line = trim(line);
    if line.last() == Some(&b'&') {
        *bg = true;
        line = trim(&line[..line.len() - 1]);
    }
    let mut start = 0usize;
    let mut i = 0usize;
    let mut in_q: Option<u8> = None;
    while i <= line.len() {
        let at_end = i == line.len();
        let c = if at_end { 0 } else { line[i] };
        if !at_end {
            if in_q.is_none() && (c == b'"' || c == b'\'') {
                in_q = Some(c);
            } else if in_q == Some(c) {
                in_q = None;
            }
        }
        if (at_end || (c == b'|' && in_q.is_none())) && *nstage < stages.len() {
            let chunk = trim(&line[start..i]);
            if !chunk.is_empty() {
                let mut st = Stage::empty();
                /* Split trailing `>` / `>>` file from chunk. */
                let mut body = chunk;
                let mut redir_path: Option<&[u8]> = None;
                if let Some((pos, append)) = find_redir(chunk) {
                    body = trim(&chunk[..pos]);
                    let after = if append { pos + 2 } else { pos + 1 };
                    redir_path = Some(trim(&chunk[after.min(chunk.len())..]));
                    st.redir_append = append;
                }
                if !tokenize(body, &mut st.argv) {
                    return false;
                }
                if let Some(rp) = redir_path {
                    let n = rp.len().min(ARG_LEN);
                    let mut buf = [0u8; ARG_LEN];
                    buf[..n].copy_from_slice(&rp[..n]);
                    st.redir = Some(buf);
                    st.redir_len = n;
                }
                stages[*nstage] = st;
                *nstage += 1;
            }
            start = i + 1;
        }
        if !at_end {
            i += 1;
        } else {
            break;
        }
    }
    *nstage > 0
}

fn find_redir(chunk: &[u8]) -> Option<(usize, bool)> {
    let mut in_q: Option<u8> = None;
    for (i, &c) in chunk.iter().enumerate() {
        if in_q.is_none() && (c == b'"' || c == b'\'') {
            in_q = Some(c);
        } else if in_q == Some(c) {
            in_q = None;
        } else if in_q.is_none() && c == b'>' {
            let append = i + 1 < chunk.len() && chunk[i + 1] == b'>';
            return Some((i, append));
        }
    }
    None
}

fn wait_fg(id: isize) {
    if id < 0 {
        return;
    }
    loop {
        let st = sys::wait(id as usize);
        if st != -11 {
            break;
        }
        let _ = sys::yield_now();
    }
}

fn drain_pipe(pid: usize, out: &mut [u8]) -> usize {
    let mut n = 0usize;
    let mut spins = 0usize;
    while n < out.len() && spins < 100_000 {
        let mut tmp = [0u8; 64];
        let r = sys::pipe_read(pid, &mut tmp);
        if r > 0 {
            let take = (r as usize).min(out.len() - n).min(tmp.len());
            out[n..n + take].copy_from_slice(&tmp[..take]);
            n += take;
            spins = 0;
        } else {
            spins += 1;
            let _ = sys::yield_now();
        }
    }
    n
}

fn cmd_echo(argv: &Argv, env: &Env, out: &mut [u8]) -> usize {
    let mut o = 0usize;
    for i in 1..argv.n {
        if i > 1 && o < out.len() {
            out[o] = b' ';
            o += 1;
        }
        if let Some(a) = argv.get(i) {
            let mut exp = [0u8; ARG_LEN];
            let en = expand_vars(env, a, &mut exp);
            let take = en.min(out.len().saturating_sub(o));
            out[o..o + take].copy_from_slice(&exp[..take]);
            o += take;
        }
    }
    if o < out.len() {
        out[o] = b'\n';
        o += 1;
    }
    o
}

fn run_exec(
    path: &[u8],
    bg: bool,
    capture_pipe: Option<usize>,
) -> isize {
    let id = sys::exec(path);
    if id < 0 {
        let _ = sys::debug_write("shell: exec failed\n");
        return id;
    }
    if let Some(p) = capture_pipe {
        let _ = sys::task_stdout(id as usize, p);
    }
    if bg {
        let _ = sys::debug_write("shell: [bg] id=");
        write_u32(id as u32);
        let _ = sys::debug_write("\n");
    } else {
        wait_fg(id);
        if capture_pipe.is_some() {
            let _ = sys::task_stdout(id as usize, STDOUT_CONSOLE);
        }
    }
    id
}

fn help() {
    let _ = sys::debug_write("DeepRoot shell 1.11 — builtins:\n");
    let _ = sys::debug_write("  help / ls [DIR] / cat FILE / cp SRC DST / run ELF / exit\n");
    let _ = sys::debug_write("  mkdir DIR / rmdir DIR / rm FILE\n");
    let _ = sys::debug_write("  modload PATH [badge] / modules / moddemo / modnote\n");
    let _ = sys::debug_write("  echo ARGS   pwd   cd DIR   env   export KEY=VAL   history\n");
    let _ = sys::debug_write("Operators:  |  pipe   > file   >> append   &  background\n");
    let _ = sys::debug_write("Root `>` / `>>` write durable DRFS (survives QEMU restart).\n");
    let _ = sys::debug_write("Examples:\n");
    let _ = sys::debug_write("  echo hello > note.txt; cat note.txt\n");
    let _ = sys::debug_write("  echo more >> note.txt\n");
}

fn read_line(buf: &mut [u8], hist: &History) -> usize {
    let mut n = 0usize;
    let mut hist_back = 0usize;
    let mut esc = 0u8; /* 0 none, 1 saw ESC, 2 saw [ */
    while n < buf.len() {
        let c = sys::debug_read_byte();
        if c < 0 {
            let _ = sys::yield_now();
            continue;
        }
        let b = c as u8;
        if esc == 1 {
            esc = if b == b'[' { 2 } else { 0 };
            continue;
        }
        if esc == 2 {
            esc = 0;
            if b == b'A' {
                /* Up: recall history */
                hist_back = hist_back.saturating_add(1).min(hist.count);
                if let Some(h) = hist.get(hist_back) {
                    while n > 0 {
                        n -= 1;
                        let _ = sys::debug_write("\x08 \x08");
                    }
                    let take = h.len().min(buf.len());
                    buf[..take].copy_from_slice(&h[..take]);
                    n = take;
                    let _ = sys::debug_write_bytes(&buf[..n]);
                }
            } else if b == b'B' {
                if hist_back > 0 {
                    hist_back -= 1;
                }
                while n > 0 {
                    n -= 1;
                    let _ = sys::debug_write("\x08 \x08");
                }
                if hist_back > 0 {
                    if let Some(h) = hist.get(hist_back) {
                        let take = h.len().min(buf.len());
                        buf[..take].copy_from_slice(&h[..take]);
                        n = take;
                        let _ = sys::debug_write_bytes(&buf[..n]);
                    }
                }
            }
            continue;
        }
        if b == 0x1b {
            esc = 1;
            continue;
        }
        if b == b'\r' || b == b'\n' {
            let _ = sys::debug_write("\n");
            break;
        }
        if b == 0x7f || b == 8 {
            if n > 0 {
                n -= 1;
                let _ = sys::debug_write("\x08 \x08");
            }
            hist_back = 0;
            continue;
        }
        if b < 0x20 {
            continue;
        }
        buf[n] = b;
        n += 1;
        hist_back = 0;
        let _ = sys::debug_write_bytes(&buf[n - 1..n]);
    }
    n
}

fn run_stage(
    st: &Stage,
    env: &Env,
    bg: bool,
    infile: Option<&[u8]>,
    capture: bool,
    outbuf: &mut [u8],
) -> usize {
    let Some(cmd) = st.argv.get(0) else {
        return 0;
    };

    /* Builtins that can produce text into outbuf when capture/redir. */
    let mut produced = [0u8; 256];
    let mut plen = 0usize;

    if cmd == b"echo" {
        plen = cmd_echo(&st.argv, env, &mut produced);
    } else if cmd == b"help" {
        help();
        return 0;
    } else if cmd == b"ls" {
        if let Some(p) = st.argv.get(1) {
            let _ = sys::fs_list_path(p);
        } else {
            let _ = sys::fs_list();
        }
        return 0;
    } else if cmd == b"pwd" {
        let mut buf = [0u8; 96];
        let n = sys::getcwd(&mut buf);
        if n > 0 {
            let _ = sys::debug_write_bytes(&buf[..n as usize]);
            let _ = sys::debug_write("\n");
        }
        return 0;
    } else if cmd == b"cd" {
        return 0; /* handled by caller */
    } else if cmd == b"mkdir" {
        if let Some(p) = st.argv.get(1) {
            if sys::fs_mkdir(p) < 0 {
                let _ = sys::debug_write("shell: mkdir failed\n");
            }
        } else {
            let _ = sys::debug_write("shell: mkdir <dir>\n");
        }
        return 0;
    } else if cmd == b"rmdir" {
        if let Some(p) = st.argv.get(1) {
            if sys::fs_rmdir(p) < 0 {
                let _ = sys::debug_write("shell: rmdir failed\n");
            }
        } else {
            let _ = sys::debug_write("shell: rmdir <dir>\n");
        }
        return 0;
    } else if cmd == b"rm" {
        if let Some(p) = st.argv.get(1) {
            if sys::fs_unlink(p) < 0 {
                let _ = sys::debug_write("shell: rm failed\n");
            }
        } else {
            let _ = sys::debug_write("shell: rm <file>\n");
        }
        return 0;
    } else if cmd == b"modules" {
        let _ = sys::module_list();
        return 0;
    } else if cmd == b"cp" {
        let (Some(src), Some(dst)) = (st.argv.get(1), st.argv.get(2)) else {
            let _ = sys::debug_write("shell: cp <src> <dst>  (dst is a VFS path)\n");
            return 0;
        };
        if sys::fs_cp(src, dst) < 0 {
            let _ = sys::debug_write("shell: cp failed\n");
        }
        return 0;
    } else if cmd == b"moddemo" {
        do_modload(b"moddemo", 0xD001);
        return 0;
    } else if cmd == b"modnote" {
        do_modload(b"modnote", 0xD002);
        return 0;
    } else if cmd == b"modload" {
        let Some(p) = st.argv.get(1) else {
            let _ = sys::debug_write("shell: modload <elf|vfs-path> [badge_hex]\n");
            return 0;
        };
        let mut badge = default_badge(p);
        if let Some(b) = st.argv.get(2) {
            badge = parse_u64(b).unwrap_or(badge);
        }
        do_modload(p, badge);
        return 0;
    } else if cmd == b"env" {
        env.list();
        return 0;
    } else if cmd == b"export" {
        return 0;
    } else if cmd == b"history" {
        return 0;
    } else if cmd == b"exit" {
        let _ = sys::debug_write("shell: bye\n");
        sys::exit(0);
    } else if cmd == b"cat" {
        if let Some(pipe_in) = infile {
            let _ = sys::debug_write_bytes(pipe_in);
            return 0;
        }
        if let Some(p) = st.argv.get(1) {
            let _ = sys::fs_cat(p);
        } else {
            let _ = sys::debug_write("shell: cat <file>\n");
        }
        return 0;
    } else if cmd == b"run" || cmd == b"hello" || cmd == b"badapple" {
        let path: &[u8] = if cmd == b"run" {
            match st.argv.get(1) {
                Some(p) => p,
                None => {
                    let _ = sys::debug_write("shell: run <elf>\n");
                    return 0;
                }
            }
        } else {
            cmd
        };
        /* Embed ELFs live at root; pass basename / absolute as typed. */
        if capture || st.redir.is_some() {
            let pid = sys::pipe();
            if pid < 0 {
                let _ = sys::debug_write("shell: pipe failed\n");
                return 0;
            }
            let _ = run_exec(path, bg, Some(pid as usize));
            if !bg {
                plen = drain_pipe(pid as usize, &mut produced);
                let _ = sys::pipe_close(pid as usize);
            } else {
                let _ = sys::pipe_close(pid as usize);
            }
        } else {
            let _ = run_exec(path, bg, None);
            return 0;
        }
    } else {
        let _ = sys::debug_write("shell: unknown - type: help\n");
        return 0;
    }

    /* Redir `>` / `>>` — root basenames land on durable DRFS (1.11). */
    if let Some(rb) = st.redir {
        let path = &rb[..st.redir_len];
        let rc = if st.redir_append {
            sys::fs_append(path, &produced[..plen])
        } else {
            sys::fs_write(path, &produced[..plen])
        };
        if rc < 0 {
            let _ = sys::debug_write("shell: write failed\n");
        }
        return 0;
    }

    if capture {
        let take = plen.min(outbuf.len());
        outbuf[..take].copy_from_slice(&produced[..take]);
        return take;
    }

    if plen > 0 {
        let _ = sys::debug_write_bytes(&produced[..plen]);
    }
    0
}

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write(
        "shell: DeepRoot shell 1.11 ready (help, >, >> durable at /)\n",
    );
    let mut buf = [0u8; LINE_MAX];
    let mut hist = History::new();
    let mut env = Env::new();
    let _ = env.set(b"SHELL", b"deeproot");
    let _ = env.set(b"VERSION", b"1.11.0");

    loop {
        let _ = sys::debug_write("deeproot> ");
        let n = read_line(&mut buf, &hist);
        let line = trim(&buf[..n]);
        if line.is_empty() {
            continue;
        }
        hist.push(line);

        /* One-shot builtins that need mut env before pipeline. */
        let mut argv0 = Argv::empty();
        if tokenize(line, &mut argv0) {
            if let Some(c) = argv0.get(0) {
                if c == b"export" {
                    if let Some(spec) = argv0.get(1) {
                        if let Some(eq) = spec.iter().position(|&b| b == b'=') {
                            let _ = env.set(&spec[..eq], &spec[eq + 1..]);
                        } else {
                            let _ = sys::debug_write("shell: export KEY=VAL\n");
                        }
                    }
                    continue;
                }
                if c == b"cd" {
                    let path = argv0.get(1).unwrap_or(b"/");
                    if sys::chdir(path) < 0 {
                        let _ = sys::debug_write("shell: cd failed\n");
                    }
                    continue;
                }
                if c == b"history" {
                    hist.dump();
                    continue;
                }
            }
        }

        let mut stages = [const { Stage::empty() }; PIPE_STAGES];
        let mut nstage = 0usize;
        let mut bg = false;
        if !split_pipeline(line, &mut stages, &mut nstage, &mut bg) {
            let _ = sys::debug_write("shell: parse error\n");
            continue;
        }

        let mut pipe_buf = [0u8; 256];
        let mut pipe_len = 0usize;
        for s in 0..nstage {
            let capture = s + 1 < nstage;
            let infile = if s > 0 {
                Some(&pipe_buf[..pipe_len] as &[u8])
            } else {
                None
            };
            /* Only last stage may use & for ELF; builtins ignore. */
            let stage_bg = bg && s + 1 == nstage;
            let mut next = [0u8; 256];
            let n = run_stage(
                &stages[s],
                &env,
                stage_bg,
                infile,
                capture,
                &mut next,
            );
            if capture {
                pipe_buf = next;
                pipe_len = n;
            }
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("shell: PANIC\n");
    sys::exit(1);
}
