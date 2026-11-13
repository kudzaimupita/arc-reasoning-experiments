"""Test-time-training (TTT) solver, Experiment 26.

For each task, train a small fully convolutional network from scratch on that task's own train pairs (augmented with
the 8 flips/rotations; random recolouring only with --recolour), then predict each test output; the 8
flip/rotation views of the test input vote. No pretraining and no data beyond the task itself.

Honesty rules: train on all train pairs but the last; answer only if the held-out pair is predicted exactly and at
least 5 of 8 views agree on each test answer. Otherwise write no attempt ("don't know"), like the reasoner.

Phase 1 covers tasks whose outputs have the same size as their inputs in every train pair; other tasks get no answer.

Job mode (same contract as `reasoner job`): reads $ARC_JOB_IN/tasks.txt and $ARC_JOB_IN/<id>.json (test outputs may
be stripped), writes $ARC_JOB_OUT/out/<id>.json (predictions) and $ARC_JOB_OUT/results.jsonl. ARC_JOB_OUT defaults to
the current folder. Usage:
    python3 ttt_solver.py job [--steps N] [--width W] [--threads T] [--seed S]
    python3 ttt_solver.py solve <task.json> [...]           (one task, prints a summary)
"""
import json, os, sys, time, random
import torch, torch.nn as nn, torch.nn.functional as F


def arg(name, default, cast=str):
    a = sys.argv
    return cast(a[a.index(name) + 1]) if name in a else default


STEPS, WIDTH, SEED = arg("--steps", 1500, int), arg("--width", 64, int), arg("--seed", 0, int)
RECOLOUR = "--recolour" in sys.argv          # random recolouring breaks colour-specific rules; off by default
torch.set_num_threads(arg("--threads", 1, int))
DEV = "cpu"


def d8(g, k):
    """The k-th of the 8 flips/rotations of a 2-D tensor (k in 0..7)."""
    g = torch.rot90(g, k % 4, dims=(-2, -1))
    return torch.flip(g, dims=(-1,)) if k >= 4 else g


def d8_inv(g, k):
    if k >= 4: g = torch.flip(g, dims=(-1,))
    return torch.rot90(g, -(k % 4), dims=(-2, -1))


class Net(nn.Module):
    """Residual fully convolutional net; dilations give a receptive field of about 30 cells."""
    def __init__(self, w):
        super().__init__()
        self.inp = nn.Conv2d(11, w, 3, padding=1)
        self.blocks = nn.ModuleList([nn.Conv2d(w, w, 3, padding=d, dilation=d) for d in (1, 2, 4, 8, 1, 2, 4, 1)])
        self.norms = nn.ModuleList([nn.GroupNorm(8, w) for _ in self.blocks])
        self.out = nn.Conv2d(w, 10, 1)

    def forward(self, x):                      # x: (B, 11, H, W) = 10 one-hot colours + validity mask
        h = F.gelu(self.inp(x))
        for c, n in zip(self.blocks, self.norms): h = h + F.gelu(n(c(h)))
        return self.out(h)


def one_hot(grid, H, W):
    """(11, H, W): one-hot colours padded to H x W, plus a mask channel marking real cells."""
    g = torch.as_tensor(grid)
    x = torch.zeros(11, H, W)
    x[g.flatten().long(), torch.arange(g.numel()) // g.shape[1], torch.arange(g.numel()) % g.shape[1]] = 1
    x[10, :g.shape[0], :g.shape[1]] = 1
    return x


def batch(pairs, rng):
    """One augmented copy of all train pairs: same random flip/rotation and recolouring for the whole batch."""
    k = rng.randrange(8)
    perm = list(range(10))
    if RECOLOUR: rest = perm[1:]; rng.shuffle(rest); perm = [0] + rest                # keep colour 0 (background)
    P = torch.tensor(perm)
    ins = [d8(P[torch.as_tensor(i)], k) for i, _ in pairs]
    outs = [d8(P[torch.as_tensor(o)], k) for _, o in pairs]
    H = max(t.shape[0] for t in ins); W = max(t.shape[1] for t in ins)
    X = torch.stack([one_hot(t.tolist(), H, W) for t in ins])
    Y = torch.full((len(outs), H, W), -100, dtype=torch.long)
    for j, t in enumerate(outs): Y[j, :t.shape[0], :t.shape[1]] = t
    return X, Y


def _unused_predict(model, grid):
    """Votes over the 8 views; returns grids ordered by votes (most first)."""
    votes = {}
    with torch.no_grad():
        for k in range(8):
            t = d8(torch.as_tensor(grid), k)
            logits = model(one_hot(t.tolist(), t.shape[0], t.shape[1])[None])[0]
            pred = d8_inv(logits.argmax(0), k)
            key = json.dumps(pred.tolist())
            votes[key] = votes.get(key, 0) + 1
    return [json.loads(k) for k, _ in sorted(votes.items(), key=lambda kv: -kv[1])]


def train(pairs, seed):
    torch.manual_seed(seed); rng = random.Random(seed)
    model = Net(WIDTH).to(DEV)
    opt = torch.optim.AdamW(model.parameters(), lr=3e-3, weight_decay=1e-4)
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, 3e-3, total_steps=STEPS)
    for step in range(STEPS):
        X, Y = batch(pairs, rng)
        loss = F.cross_entropy(model(X), Y, ignore_index=-100)
        opt.zero_grad(); loss.backward(); opt.step(); sched.step()
    return model, loss.item()


def votes(model, grid):
    """(grids by votes, top vote count) over the 8 views."""
    v = {}
    with torch.no_grad():
        for k in range(8):
            t = d8(torch.as_tensor(grid), k)
            logits = model(one_hot(t.tolist(), t.shape[0], t.shape[1])[None])[0]
            key = json.dumps(d8_inv(logits.argmax(0), k).tolist())
            v[key] = v.get(key, 0) + 1
    order = sorted(v.items(), key=lambda kv: -kv[1])
    return [json.loads(k) for k, _ in order], order[0][1]


def solve(task, seed):
    """Honesty checks (E26): (1) hold out the last train pair, train on the rest, and require the held-out pair to
    be predicted exactly; (2) require at least 5 of 8 views to agree on every test answer. Otherwise no answer."""
    t0 = time.process_time()
    pairs = [(p["input"], p["output"]) for p in task["train"]]
    if not all(len(i) == len(o) and len(i[0]) == len(o[0]) for i, o in pairs):
        return None, {"status": "skip-size", "cpu_s": 0.0}
    if len(pairs) < 2:
        return None, {"status": "skip-one-pair", "cpu_s": 0.0}
    model, loss = train(pairs[:-1], seed)
    held, agree = votes(model, pairs[-1][0])
    info = {"cpu_s": round(time.process_time() - t0, 1), "loss": round(loss, 4)}
    if held[0] != pairs[-1][1]:
        return None, {"status": "held-out-wrong", **info}
    attempts = []
    for p in task["test"]:
        cands, top = votes(model, p["input"])
        if top < 5: return None, {"status": "views-disagree", **info}
        attempts.append({f"attempt_{j + 1}": c for j, c in enumerate(cands[:2])})
    return attempts, {"status": "answered", **info}


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode == "solve":
        task = json.load(open(sys.argv[2]))
        att, info = solve(task, SEED)
        print(json.dumps(info)); print(json.dumps(att))
    elif mode == "job":
        inp, home = os.environ["ARC_JOB_IN"], os.environ.get("ARC_JOB_OUT", ".")
        os.makedirs(f"{home}/out", exist_ok=True)
        ids = [l.strip() for l in open(f"{inp}/tasks.txt") if l.strip()]
        lines = []
        for tid in ids:
            if not all(c.isalnum() or c in "_-" for c in tid) or len(tid) > 64: continue
            task = json.load(open(f"{inp}/{tid}.json"))
            att, info = solve(task, SEED)
            n_test = len(task["test"])
            with open(f"{home}/out/{tid}.json", "w") as f: json.dump(att if att else [{} for _ in range(n_test)], f)
            lines.append(json.dumps({"task": tid, "solved": att is not None, **info}))
            with open(f"{home}/results.jsonl", "w") as f: f.write("\n".join(lines) + "\n")
        print(json.dumps({"tasks": len(ids), "answered": sum('"solved": true' in l for l in lines)}))
    else:
        print(__doc__); sys.exit(64)


if __name__ == "__main__":
    main()
