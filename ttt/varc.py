"""VARC-style solver (Experiment 26b): a network pretrained across many generated tasks, then fine-tuned per task.

Model (about 3M parameters, no language model):
  * every grid sits at the top-left of a 30x30 canvas; 11 classes per cell (colours 0-9, 10 = outside the grid),
    so the output size is predicted too;
  * a pair encoder reads (input, output) example pairs; their mean is the task vector z;
  * a decoder (dilated residual convolutions with FiLM conditioning on z) maps the query input to the output canvas.

Pretraining (`pretrain`): episodes of 2-4 context pairs + 1 query pair, all drawn from one RE-ARC generator, with
one random flip/rotation and colour permutation per episode. Data: RE-ARC examples of TRAIN-pool generators only.

Test time (`job` / `solve`): for each task, fine-tune a copy of the model on leave-one-out episodes built from the
task's own train pairs (the last pair held back), then
  1. honesty check: the held-back pair must be predicted exactly from the other pairs;
  2. predict each test output with all train pairs as context, under the 8 flips/rotations, and vote;
     answer only if at least 5 of 8 views agree. Otherwise no answer.

Usage:
  python3 varc.py pretrain --data DIR --out model.pt [--steps N] [--batch B] [--width W]
  python3 varc.py job --model model.pt [--ttt-steps N]        (ARC_JOB_IN / ARC_JOB_OUT contract, like the reasoner)
  python3 varc.py solve task.json --model model.pt
"""
import glob, json, math, os, random, sys, time
import torch, torch.nn as nn, torch.nn.functional as F

def arg(name, default, cast=str):
    a = sys.argv
    return cast(a[a.index(name) + 1]) if name in a else default

DEV = "cuda" if torch.cuda.is_available() else ("mps" if torch.backends.mps.is_available() else "cpu")
S = 30          # canvas size
OUT = 10        # "outside the grid" class


def d8(g, k):
    g = torch.rot90(g, k % 4, dims=(-2, -1))
    return torch.flip(g, dims=(-1,)) if k >= 4 else g


def d8_inv(g, k):
    if k >= 4: g = torch.flip(g, dims=(-1,))
    return torch.rot90(g, -(k % 4), dims=(-2, -1))


def canvas(grid, k=0, perm=None):
    """Grid (list of lists) -> (S, S) long tensor at top-left, transformed by d8 k and colour permutation."""
    g = torch.as_tensor(grid, dtype=torch.long)
    if perm is not None: g = perm[g]
    g = d8(g, k)
    c = torch.full((S, S), OUT, dtype=torch.long)
    c[:g.shape[0], :g.shape[1]] = g
    return c


def onehot(c):                       # (..., S, S) long -> (..., 11, S, S) float
    return F.one_hot(c, 11).movedim(-1, -3).float()


def decode(pred):
    """(S, S) class map -> grid (list of lists) of the top-left non-outside rectangle, or None."""
    h = next((r for r in range(S) if pred[r, 0] == OUT), S)
    w = next((c for c in range(S) if pred[0, c] == OUT), S)
    if h == 0 or w == 0: return None
    g = pred[:h, :w]
    if (g == OUT).any(): return None
    return g.tolist()


class Block(nn.Module):
    def __init__(self, w, d, zdim):
        super().__init__()
        self.conv = nn.Conv2d(w, w, 3, padding=d, dilation=d)
        self.norm = nn.GroupNorm(8, w)
        self.film = nn.Linear(zdim, 2 * w)

    def forward(self, h, z):
        g, b = self.film(z).chunk(2, dim=-1)
        y = self.norm(self.conv(h)) * (1 + g[..., None, None]) + b[..., None, None]
        return h + F.gelu(y)


class Model(nn.Module):
    def __init__(self, w=160, zdim=256):
        super().__init__()
        self.enc = nn.Sequential(
            nn.Conv2d(22, w, 3, padding=1), nn.GELU(),
            nn.Conv2d(w, w, 3, padding=2, dilation=2), nn.GELU(),
            nn.Conv2d(w, w, 3, padding=4, dilation=4), nn.GELU(),
            nn.Conv2d(w, w, 3, padding=1), nn.GELU())
        self.zproj = nn.Sequential(nn.Linear(2 * w, zdim), nn.GELU(), nn.Linear(zdim, zdim))
        self.inp = nn.Conv2d(11, w, 3, padding=1)
        self.blocks = nn.ModuleList([Block(w, d, zdim) for d in (1, 2, 4, 8, 1, 2, 4, 8, 1, 1)])
        self.out = nn.Conv2d(w, 11, 1)

    def task_vector(self, ci, co):    # (B, M, S, S) context inputs/outputs -> (B, zdim)
        B, M = ci.shape[:2]
        x = torch.cat([onehot(ci), onehot(co)], dim=2).flatten(0, 1)     # (B*M, 22, S, S)
        h = self.enc(x)
        v = torch.cat([h.mean((-2, -1)), h.amax((-2, -1))], dim=-1)
        return self.zproj(v).view(B, M, -1).mean(1)

    def forward(self, ci, co, qi):    # -> logits (B, 11, S, S)
        z = self.task_vector(ci, co)
        h = F.gelu(self.inp(onehot(qi)))
        for blk in self.blocks: h = blk(h, z)
        return self.out(h)


def episode(examples, rng, m):
    """m context pairs + 1 query from a list of {"input","output"}, one shared random augmentation."""
    k = rng.randrange(8)
    perm = list(range(10)); rest = perm[1:]; rng.shuffle(rest)
    perm = torch.tensor([0] + rest if rng.random() < 0.5 else rng.sample(range(10), 10))
    ex = rng.sample(examples, m + 1)
    ci = torch.stack([canvas(e["input"], k, perm) for e in ex[:m]])
    co = torch.stack([canvas(e["output"], k, perm) for e in ex[:m]])
    return ci, co, canvas(ex[m]["input"], k, perm), canvas(ex[m]["output"], k, perm)


def make_batch(pool, rng, B):
    m = rng.randint(2, 4)
    ok = [ex for ex in pool if len(ex) > m] or pool
    m = min(m, min(len(ex) for ex in ok) - 1)
    eps = [episode(ok[rng.randrange(len(ok))], rng, m) for _ in range(B)]
    return [torch.stack(t).to(DEV) for t in zip(*eps)]


def dev_exact(model, dev_tasks, n=256, seed=7):
    """Diagnostic: share of held-out tasks whose test output is predicted exactly from their train pairs (one view,
    no fine-tuning). Dev tasks come from generators that are not in the pretraining data."""
    rng = random.Random(seed); hit = 0; model.eval()
    for _ in range(n):
        t = dev_tasks[rng.randrange(len(dev_tasks))]
        ctx = [(p["input"], p["output"]) for p in t["train"]]
        hit += predict(model, ctx, t["test"][0]["input"], 0) == t["test"][0]["output"]
    model.train(); return hit / n


def pretrain():
    data, out = arg("--data", ""), arg("--out", "model.pt")
    dev_dir = arg("--dev", "")
    dev_tasks = [json.load(open(f)) for f in sorted(glob.glob(f"{dev_dir}/*.json"))] if dev_dir else []
    steps, B, width = arg("--steps", 20000, int), arg("--batch", 64, int), arg("--width", 160, int)
    pool = []
    for f in sorted(glob.glob(f"{data}/*.json")):
        ex = [e for e in json.load(open(f)) if len(e["input"]) <= S and len(e["input"][0]) <= S and len(e["output"]) <= S and len(e["output"][0]) <= S]
        if len(ex) >= arg("--min-ex", 6, int): pool.append(ex)
    print(f"{len(pool)} generators, {sum(map(len, pool))} examples; device {DEV}", flush=True)
    torch.manual_seed(26); rng = random.Random(26)
    model = Model(width).to(DEV)
    print(f"{sum(p.numel() for p in model.parameters())/1e6:.2f}M parameters", flush=True)
    opt = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=0.01)
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, 1e-3, total_steps=steps, pct_start=0.05)
    amp = torch.autocast("cuda", dtype=torch.bfloat16) if DEV == "cuda" else torch.autocast("cpu", enabled=False)
    t0 = time.time(); run = 0.0
    for step in range(1, steps + 1):
        ci, co, qi, qo = make_batch(pool, rng, B)
        with amp: loss = F.cross_entropy(model(ci, co, qi).float(), qo)
        opt.zero_grad(); loss.backward(); torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0); opt.step(); sched.step()
        run = 0.98 * run + 0.02 * loss.item() if step > 1 else loss.item()
        if step % 500 == 0 or step == steps:
            with torch.no_grad():
                pred = model(ci, co, qi).argmax(1)
                exact = (pred == qo).flatten(1).all(1).float().mean().item()
            dv = f"  dev exact {dev_exact(model, dev_tasks):.3f}" if dev_tasks and step % 2000 == 0 else ""
            print(f"step {step}: loss {run:.4f}  batch exact {exact:.3f}{dv}  ({time.time()-t0:.0f} s)", flush=True)
            torch.save({"width": width, "state": model.state_dict()}, out)
    print("saved", out, flush=True)


def load(path):
    ck = torch.load(path, map_location=DEV, weights_only=True)
    m = Model(ck["width"]).to(DEV); m.load_state_dict(ck["state"]); return m


def predict(model, ctx, grid, k):
    """Prediction for one query grid under view k (context transformed the same way)."""
    ci = torch.stack([canvas(i, k) for i, _ in ctx])[None].to(DEV)
    co = torch.stack([canvas(o, k) for _, o in ctx])[None].to(DEV)
    qi = canvas(grid, k)[None].to(DEV)
    with torch.no_grad(): pred = model(ci, co, qi).argmax(1)[0].cpu()
    t = decode(pred)
    if t is None: return None
    return d8_inv(torch.as_tensor(t), k).tolist()


def vote(model, ctx, grid):
    v = {}
    for k in range(8):
        p = predict(model, ctx, grid, k)
        if p is not None: key = json.dumps(p); v[key] = v.get(key, 0) + 1
    order = sorted(v.items(), key=lambda kv: -kv[1])
    return [json.loads(k) for k, _ in order], (order[0][1] if order else 0)


def ttt(base, pairs, steps, seed):
    """Fine-tune a copy on leave-one-out episodes from `pairs` (with augmentation)."""
    model = Model(base.inp.out_channels).to(DEV); model.load_state_dict(base.state_dict())
    if len(pairs) < 2 or steps == 0: return model
    opt = torch.optim.AdamW(model.parameters(), lr=3e-4, weight_decay=0.0)
    rng = random.Random(seed); ex = [{"input": i, "output": o} for i, o in pairs]
    for _ in range(steps):
        eps = [episode(ex, rng, len(ex) - 1) for _ in range(8)]
        ci, co, qi, qo = [torch.stack(t).to(DEV) for t in zip(*eps)]
        loss = F.cross_entropy(model(ci, co, qi), qo)
        opt.zero_grad(); loss.backward(); opt.step()
    return model


def solve(base, task, steps, seed=0):
    t0 = time.time()
    pairs = [(p["input"], p["output"]) for p in task["train"]]
    if len(pairs) < 2: return None, {"status": "skip-one-pair"}
    model = ttt(base, pairs[:-1], steps, seed)
    held, _ = vote(model, pairs[:-1], pairs[-1][0])
    info = {"seconds": round(time.time() - t0, 1)}
    if not held or held[0] != pairs[-1][1]: return None, {"status": "held-out-wrong", **info}
    attempts = []
    for p in task["test"]:
        cands, top = vote(model, pairs, p["input"])
        if top < 5: return None, {"status": "views-disagree", **info}
        attempts.append({f"attempt_{j + 1}": c for j, c in enumerate(cands[:2])})
    return attempts, {"status": "answered", **info}


def job():
    base = load(arg("--model", "model.pt")); steps = arg("--ttt-steps", 100, int)
    inp, home = os.environ["ARC_JOB_IN"], os.environ.get("ARC_JOB_OUT", ".")
    os.makedirs(f"{home}/out", exist_ok=True)
    ids = [l.strip() for l in open(f"{inp}/tasks.txt") if l.strip()]
    lines = []
    for tid in ids:
        if not all(c.isalnum() or c in "_-" for c in tid) or len(tid) > 64: continue
        task = json.load(open(f"{inp}/{tid}.json"))
        att, info = solve(base, task, steps)
        with open(f"{home}/out/{tid}.json", "w") as f: json.dump(att if att else [{} for _ in task["test"]], f)
        lines.append(json.dumps({"task": tid, "solved": att is not None, **info}))
        with open(f"{home}/results.jsonl", "w") as f: f.write("\n".join(lines) + "\n")
    print(json.dumps({"tasks": len(ids), "answered": sum('"solved": true' in l for l in lines)}), flush=True)


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode == "pretrain": pretrain()
    elif mode == "job": job()
    elif mode == "solve":
        att, info = solve(load(arg("--model", "model.pt")), json.load(open(sys.argv[2])), arg("--ttt-steps", 100, int))
        print(json.dumps(info)); print(json.dumps(att))
    else: print(__doc__); sys.exit(64)
