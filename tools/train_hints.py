#!/usr/bin/env python3
"""Train the hint network (Experiment 19): an MLP from task features to the operation kinds a task needs.

Input: dream files written by `reasoner dream` (random programs on train-pool inputs only) and their kinds.json.
Output: reasoner/src/hints.json (weights compiled into the reasoner). No real task's solution is used for training.
Usage: python3 tools/train_hints.py <dream dir> [epochs] [out file] [real.jsonl] [real repeats]
E22: real.jsonl (tools/real_hint_examples.py) adds train-pool solutions, repeated so they carry weight.
"""
import glob, json, sys, time
import numpy as np, torch, torch.nn as nn
d = sys.argv[1]; epochs = int(sys.argv[2]) if len(sys.argv) > 2 else 6
out_path = sys.argv[3] if len(sys.argv) > 3 else "reasoner/src/hints.json"
real_path = sys.argv[4] if len(sys.argv) > 4 else None; real_rep = int(sys.argv[5]) if len(sys.argv) > 5 else 0
kinds = json.load(open(f"{d}/kinds.json")); K = len(kinds)
F, Y = [], []
for f in sorted(glob.glob(f"{d}/dreams-*.jsonl")):
    for line in open(f):
        r = json.loads(line); F.append(r["f"]); y = np.zeros(K, np.float32); y[r["k"]] = 1; Y.append(y)
nreal = 0
if real_path:
    for line in open(real_path):
        r = json.loads(line); y = np.zeros(K, np.float32); y[r["k"]] = 1
        for _ in range(real_rep): F.append(r["f"]); Y.append(y)
        nreal += 1
    print(f"added {nreal} real examples x {real_rep}", flush=True)
X = torch.tensor(np.array(F, np.float32)); Y = torch.tensor(np.array(Y))
n = len(X); perm = torch.randperm(n, generator=torch.Generator().manual_seed(19)); X, Y = X[perm], Y[perm]
nv = min(50_000, n // 20); Xv, Yv, Xt, Yt = X[:nv], Y[:nv], X[nv:], Y[nv:]
print(f"{n} dreams, {X.shape[1]} features, {K} kinds; base rate of a kind {Y.mean():.4f}", flush=True)
dev = "mps" if torch.backends.mps.is_available() else "cpu"
m = nn.Sequential(nn.Linear(X.shape[1], 512), nn.ReLU(), nn.Linear(512, 512), nn.ReLU(), nn.Linear(512, K)).to(dev)
print(f"{sum(p.numel() for p in m.parameters())/1e6:.2f}M parameters on {dev}", flush=True)
opt = torch.optim.AdamW(m.parameters(), 2e-3, weight_decay=1e-4); lossf = nn.BCEWithLogitsLoss()
bs = 4096; steps = epochs * (len(Xt) // bs); sched = torch.optim.lr_scheduler.OneCycleLR(opt, 2e-3, total_steps=steps)
Xv, Yv = Xv.to(dev), Yv.to(dev); t0 = time.time()
for ep in range(epochs):
    m.train(); idx = torch.randperm(len(Xt))
    for i in range(len(Xt) // bs):
        b = idx[i * bs:(i + 1) * bs]; x, y = Xt[b].to(dev), Yt[b].to(dev)
        opt.zero_grad(); loss = lossf(m(x), y); loss.backward(); opt.step(); sched.step()
    m.eval()
    with torch.no_grad():
        lv = m(Xv); vl = lossf(lv, Yv).item()
        top = lv.topk(10, dim=1).indices; hit = torch.gather(Yv, 1, top).sum(1); rec = (hit / Yv.sum(1).clamp(min=1)).mean().item()
    print(f"epoch {ep+1}: train loss {loss.item():.4f}  val loss {vl:.4f}  val recall@10 {rec:.3f}  ({time.time()-t0:.0f} s)", flush=True)
layers = []
for mod in m:
    if isinstance(mod, nn.Linear):
        w = mod.weight.detach().cpu().numpy(); b = mod.bias.detach().cpu().numpy()
        layers.append({"rows": w.shape[0], "cols": w.shape[1], "w": [round(float(v), 5) for v in w.ravel()], "b": [round(float(v), 5) for v in b]})
json.dump({"kinds": kinds, "layers": layers, "trained_on": f"{n} examples: dreams from train-pool inputs plus {nreal} real train-pool solutions x {real_rep}"}, open(out_path, "w"))
print("wrote", out_path, flush=True)
