"use client";

import { useState, useCallback } from "react";
import Link from "next/link";
import {
  Shield, Play, Zap, CheckCircle2, XCircle, ChevronRight,
  Cpu, FileCode, Binary, Layers, Lock, AlertTriangle,
  Loader2, Copy, Check, GitBranch, Hash, ArrowLeft,
} from "lucide-react";
import {
  executeProgram, generateProof, verifyProof, parseAssembly,
  type ExecutionResult, type ProofResult, type VerifyResult, type JsTraceRow,
} from "@/lib/wasm";

const mono = { fontFamily: "'JetBrains Mono', 'Fira Code', monospace" };

const PROGRAMS: Record<string, { name: string; desc: string; code: string }> = {
  nicomachus: {
    name: "Nicomachus's Theorem (n=4)",
    desc: "Proves 1³ + 2³ + 3³ + 4³ = (1+2+3+4)² — both equal 100",
    code: `// Nicomachus: sum of cubes = square of sum
// r0=k  r1=sum_cubes  r2=sum_k  r3=scratch  r4=(sum_k)²
IMM r0, 0
IMM r1, 0
IMM r2, 0
ADDI r0, r0, 1
MUL r3, r0, r0
MUL r3, r3, r0
ADD r1, r1, r3
ADD r2, r2, r0
ADDI r0, r0, 1
MUL r3, r0, r0
MUL r3, r3, r0
ADD r1, r1, r3
ADD r2, r2, r0
ADDI r0, r0, 1
MUL r3, r0, r0
MUL r3, r3, r0
ADD r1, r1, r3
ADD r2, r2, r0
ADDI r0, r0, 1
MUL r3, r0, r0
MUL r3, r3, r0
ADD r1, r1, r3
ADD r2, r2, r0
MUL r4, r2, r2
ASSERT_EQ r1, r4
HALT`,
  },
  pythagorean: {
    name: "Pythagorean Triple (3, 4, 5)",
    desc: "Proves 3² + 4² = 5² using Goldilocks field arithmetic",
    code: `// 3² + 4² = 5²
IMM r0, 3
IMM r1, 4
ADDI r2, r1, 1
MUL r3, r0, r0
MUL r4, r1, r1
ADD r3, r3, r4
MUL r5, r2, r2
ASSERT_EQ r3, r5
HALT`,
  },
  arithmetic: {
    name: "Basic Arithmetic",
    desc: "Computes (3 + 4)² − 7 = 42",
    code: `// (3 + 4)² - 7 = 42
IMM r0, 3
IMM r1, 4
ADD r2, r0, r1
MUL r3, r2, r2
SUB r0, r3, r2
HALT`,
  },
  false_claim: {
    name: "⚠ False Claim (Will Fail)",
    desc: "1² + 2² + 3² + 4² ≠ (1+2+3+4)² → 30 ≠ 100",
    code: `// FALSE: sum of squares ≠ square of sum
IMM r0, 0
IMM r1, 0
IMM r2, 0
ADDI r0, r0, 1
MUL r3, r0, r0
ADD r1, r1, r3
ADD r2, r2, r0
ADDI r0, r0, 1
MUL r3, r0, r0
ADD r1, r1, r3
ADD r2, r2, r0
ADDI r0, r0, 1
MUL r3, r0, r0
ADD r1, r1, r3
ADD r2, r2, r0
ADDI r0, r0, 1
MUL r3, r0, r0
ADD r1, r1, r3
ADD r2, r2, r0
MUL r4, r2, r2
ASSERT_EQ r1, r4
HALT`,
  },
};

type Stage = "idle" | "executing" | "executed" | "proving" | "proved" | "verifying" | "verified";

export default function ExplorerPage() {
  const [code, setCode] = useState(PROGRAMS.nicomachus.code);
  const [selProg, setSelProg] = useState("nicomachus");
  const [stage, setStage] = useState<Stage>("idle");
  const [exec, setExec] = useState<ExecutionResult | null>(null);
  const [proof, setProof] = useState<ProofResult | null>(null);
  const [vResult, setVResult] = useState<VerifyResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [showPad, setShowPad] = useState(false);
  const [copied, setCopied] = useState(false);
  const [times, setTimes] = useState<{ exec?: number; prove?: number; verify?: number }>({});

  const reset = () => { setExec(null); setProof(null); setVResult(null); setError(null); setTimes({}); };
  const selectProg = (k: string) => { setSelProg(k); setCode(PROGRAMS[k].code); setStage("idle"); reset(); };

  const runAll = useCallback(async () => {
    setLoading(true); reset();
    try {
      const prog = parseAssembly(code);
      setStage("executing");
      let t = performance.now();
      const er = await executeProgram(prog);
      setTimes(p => ({ ...p, exec: performance.now() - t }));
      setExec(er);
      if (!er.success) { setError(er.error || "Execution failed"); setStage("idle"); return; }
      setStage("proving");
      t = performance.now();
      const pr = await generateProof(prog);
      setTimes(p => ({ ...p, prove: performance.now() - t }));
      setProof(pr);
      if (!pr.success) { setError(pr.error || "Proof failed"); setStage("executed"); return; }
      setStage("verifying");
      t = performance.now();
      const vr = await verifyProof(pr.proof_data!);
      setTimes(p => ({ ...p, verify: performance.now() - t }));
      setVResult(vr); setStage("verified");
    } catch (e: any) { setError(e.message); setStage("idle"); }
    finally { setLoading(false); }
  }, [code]);

  const runExec = useCallback(async () => {
    setLoading(true); setError(null); setProof(null); setVResult(null); setStage("executing");
    try {
      const t = performance.now();
      const r = await executeProgram(parseAssembly(code));
      setTimes(p => ({ ...p, exec: performance.now() - t }));
      setExec(r);
      r.success ? setStage("executed") : (setError(r.error || "Failed"), setStage("idle"));
    } catch (e: any) { setError(e.message); setStage("idle"); }
    finally { setLoading(false); }
  }, [code]);

  const runProve = useCallback(async () => {
    setLoading(true); setError(null); setVResult(null); setStage("proving");
    try {
      const t = performance.now();
      const r = await generateProof(parseAssembly(code));
      setTimes(p => ({ ...p, prove: performance.now() - t }));
      setProof(r);
      r.success ? setStage("proved") : (setError(r.error || "Failed"), setStage("executed"));
    } catch (e: any) { setError(e.message); setStage("executed"); }
    finally { setLoading(false); }
  }, [code]);

  const runVerify = useCallback(async () => {
    if (!proof?.proof_data) return;
    setLoading(true); setError(null); setStage("verifying");
    try {
      const t = performance.now();
      const r = await verifyProof(proof.proof_data);
      setTimes(p => ({ ...p, verify: performance.now() - t }));
      setVResult(r); setStage("verified");
    } catch (e: any) { setError(e.message); setStage("proved"); }
    finally { setLoading(false); }
  }, [proof]);

  const rows = exec?.rows.filter(r => showPad || !r.is_padding) ?? [];
  const regs = exec?.success ? exec.rows[exec.real_len - 1]?.regs : null;

  const card = { background: "#241a11", border: "1px solid #3d2e1f", borderRadius: 16 };
  const emptyCard = { ...card, padding: "60px 28px", textAlign: "center" as const };

  return (
    <div style={{ background: "#1a120b", color: "#f0e6d8", minHeight: "100vh" }}>

      {/* Header */}
      <header style={{ position: "sticky", top: 0, zIndex: 50, borderBottom: "1px solid #3d2e1f", background: "rgba(33,23,16,0.92)", backdropFilter: "blur(12px)", padding: "16px 32px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <Link href="/" style={{ display: "flex", alignItems: "center", gap: 8, color: "#7a6a58", textDecoration: "none", fontSize: 14 }}>
            <ArrowLeft size={18} /> Home
          </Link>
          <div style={{ width: 1, height: 24, background: "#3d2e1f" }} />
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <div style={{ width: 36, height: 36, borderRadius: 10, background: "linear-gradient(135deg, #00ff88, #4488ff)", display: "flex", alignItems: "center", justifyContent: "center" }}>
              <Shield size={18} color="#000" />
            </div>
            <span style={{ fontSize: 20, fontWeight: 700 }}>MM2 Explorer</span>
          </div>
        </div>
        {/* Pipeline badges */}
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          {([
            { id: "executed" as const, l: "Executed", I: Play, t: times.exec, act: stage === "executing" },
            { id: "proved" as const, l: "Proved", I: Lock, t: times.prove, act: stage === "proving" },
            { id: "verified" as const, l: "Verified", I: Shield, t: times.verify, act: stage === "verifying" },
          ]).map((s, i) => {
            const ord = ["idle","executing","executed","proving","proved","verifying","verified"];
            const done = ord.indexOf(stage) >= ord.indexOf(s.id);
            return (
              <div key={s.id} style={{ display: "flex", alignItems: "center", gap: 8 }}>
                {i > 0 && <ChevronRight size={16} color={done ? "#00ff88" : "#554332"} />}
                <div style={{
                  display: "flex", alignItems: "center", gap: 8, padding: "8px 14px", borderRadius: 10, fontSize: 14, fontWeight: 500,
                  background: s.act ? "rgba(68,136,255,0.12)" : done ? "rgba(0,255,136,0.08)" : "#2c1f14",
                  color: s.act ? "#4488ff" : done ? "#00ff88" : "#7a6a58",
                }}>
                  {s.act ? <Loader2 size={16} className="animate-spin" /> : <s.I size={16} />}
                  {s.l}
                  {done && s.t != null && <span style={{ fontSize: 12, opacity: 0.6, ...mono }}>{s.t.toFixed(0)}ms</span>}
                </div>
              </div>
            );
          })}
        </div>
      </header>

      <main style={{ padding: "40px 32px", maxWidth: 1600, margin: "0 auto" }}>

        {/* Error */}
        {error && (
          <div style={{ display: "flex", alignItems: "flex-start", gap: 16, padding: "20px 24px", borderRadius: 14, border: "1px solid rgba(255,68,102,0.25)", background: "rgba(255,68,102,0.04)", marginBottom: 32 }}>
            <AlertTriangle size={22} color="#ff4466" style={{ flexShrink: 0, marginTop: 2 }} />
            <div>
              <div style={{ fontSize: 17, fontWeight: 600, color: "#ff4466", marginBottom: 6 }}>Error</div>
              <div style={{ fontSize: 15, color: "#b8a898", ...mono }}>{error}</div>
            </div>
          </div>
        )}

        {/* Two-column */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 32, marginBottom: 40 }}>

          {/* LEFT */}
          <div>
            <div style={{ ...card, padding: 28, marginBottom: 28 }}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 20 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  <FileCode size={22} color="#4488ff" />
                  <h2 style={{ fontSize: 22, fontWeight: 700 }}>Program Editor</h2>
                </div>
                <select value={selProg} onChange={e => selectProg(e.target.value)}
                  style={{ fontSize: 15, padding: "10px 16px", borderRadius: 10, background: "#2c1f14", border: "1px solid #3d2e1f", color: "#b8a898", outline: "none", cursor: "pointer" }}>
                  {Object.entries(PROGRAMS).map(([k, p]) => <option key={k} value={k}>{p.name}</option>)}
                </select>
              </div>

              <p style={{ fontSize: 16, color: "#7a6a58", marginBottom: 20, lineHeight: 1.7 }}>
                {PROGRAMS[selProg]?.desc}
              </p>

              <div style={{ position: "relative", borderRadius: 14, border: "1px solid #3d2e1f", background: "#1a120b", overflow: "hidden" }}>
                <textarea value={code} onChange={e => { setCode(e.target.value); setStage("idle"); }} spellCheck={false}
                  style={{ width: "100%", height: 420, background: "transparent", fontSize: 15, padding: 24, border: "none", outline: "none", resize: "none", color: "#f0e6d8", lineHeight: 1.9, ...mono }} />
                <div style={{ position: "absolute", bottom: 12, right: 16, fontSize: 13, color: "#7a6a58", ...mono }}>
                  {code.split("\n").filter(l => l.trim() && !l.trim().startsWith("//")).length} instructions
                </div>
              </div>

              <div style={{ display: "flex", gap: 12, marginTop: 24 }}>
                <button onClick={runAll} disabled={loading}
                  style={{ flex: 1, padding: "16px 24px", borderRadius: 14, fontSize: 16, fontWeight: 700, display: "flex", alignItems: "center", justifyContent: "center", gap: 10, background: "linear-gradient(135deg, #00ff88, #4488ff)", color: "#000", border: "none", cursor: "pointer", opacity: loading ? 0.4 : 1 }}>
                  {loading ? <Loader2 size={20} className="animate-spin" /> : <Zap size={20} />}
                  Run Full Pipeline
                </button>
                {([
                  { fn: runExec, l: "Execute", I: Play, c: "#4488ff", d: loading },
                  { fn: runProve, l: "Prove", I: Lock, c: "#8855ff", d: loading || !exec?.success },
                  { fn: runVerify, l: "Verify", I: Shield, c: "#00ff88", d: loading || !proof?.success },
                ]).map(b => (
                  <button key={b.l} onClick={b.fn} disabled={b.d}
                    style={{ padding: "16px 20px", borderRadius: 14, fontSize: 15, fontWeight: 600, display: "flex", alignItems: "center", gap: 8, background: "#2c1f14", border: "1px solid #3d2e1f", color: "#f0e6d8", cursor: "pointer", opacity: b.d ? 0.35 : 1 }}>
                    <b.I size={18} color={b.c} />{b.l}
                  </button>
                ))}
              </div>
            </div>

            {/* ISA */}
            <div style={{ ...card, padding: 28 }}>
              <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
                <Layers size={22} color="#7a6a58" />
                <h2 style={{ fontSize: 22, fontWeight: 700 }}>ISA Reference</h2>
                <span style={{ fontSize: 15, color: "#7a6a58" }}>· Goldilocks Field · 7 Registers</span>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
                {[
                  { op: "IMM rd, imm", d: "rd ← immediate", c: "#4488ff" },
                  { op: "ADD rd, rs1, rs2", d: "rd ← rs1 + rs2", c: "#4488ff" },
                  { op: "MUL rd, rs1, rs2", d: "rd ← rs1 × rs2", c: "#4488ff" },
                  { op: "SUB rd, rs1, rs2", d: "rd ← rs1 − rs2", c: "#4488ff" },
                  { op: "ADDI rd, rs1, imm", d: "rd ← rs1 + imm", c: "#4488ff" },
                  { op: "ASSERT_EQ rs1, rs2", d: "require rs1 = rs2", c: "#ffbb00" },
                  { op: "HALT", d: "stop execution", c: "#ff4466" },
                  { op: "NOP", d: "padding only", c: "#7a6a58" },
                ].map(({ op, d, c }) => (
                  <div key={op} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 18px", borderRadius: 12, background: "#2c1f14", border: "1px solid rgba(61,46,31,0.5)" }}>
                    <code style={{ fontSize: 14, fontWeight: 600, color: c, ...mono }}>{op}</code>
                    <span style={{ fontSize: 14, color: "#7a6a58", marginLeft: 16 }}>{d}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* RIGHT */}
          <div>
            {regs ? (
              <div style={{ ...card, padding: 28, marginBottom: 28 }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 20 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                    <Cpu size={22} color="#00ddff" />
                    <h2 style={{ fontSize: 22, fontWeight: 700 }}>Final Register State</h2>
                  </div>
                  {times.exec != null && <span style={{ fontSize: 15, color: "#7a6a58", ...mono }}>{times.exec.toFixed(1)} ms</span>}
                </div>
                <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 14 }}>
                  {regs.map((v, i) => (
                    <div key={i} style={{ background: "#2c1f14", borderRadius: 14, padding: "18px 12px", textAlign: "center", border: "1px solid rgba(61,46,31,0.5)" }}>
                      <div style={{ fontSize: 15, color: "#7a6a58", marginBottom: 8 }}>r{i}</div>
                      <div style={{ fontSize: 28, fontWeight: 700, color: "#00ddff", ...mono }}>{v}</div>
                    </div>
                  ))}
                </div>
                <div style={{ fontSize: 15, color: "#7a6a58", marginTop: 18 }}>
                  {exec!.real_len} instructions executed · Padded to {exec!.padded_len} rows
                </div>
              </div>
            ) : (
              <div style={emptyCard}>
                <Cpu size={48} color="#554332" style={{ margin: "0 auto 20px" }} />
                <h3 style={{ fontSize: 22, fontWeight: 600, color: "#7a6a58", marginBottom: 12 }}>No Execution Yet</h3>
                <p style={{ fontSize: 16, color: "#7a6a58", maxWidth: 380, margin: "0 auto", lineHeight: 1.7 }}>
                  Write a program and click <strong style={{ color: "#b8a898" }}>Run Full Pipeline</strong> to see results.
                </p>
              </div>
            )}

            {proof?.success ? (
              <div style={{ ...card, padding: 28, marginBottom: 28, borderColor: "rgba(136,85,255,0.2)", boxShadow: "0 0 40px rgba(136,85,255,0.06)" }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 20 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                    <Lock size={22} color="#8855ff" />
                    <h2 style={{ fontSize: 22, fontWeight: 700 }}>STARK Proof Generated</h2>
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                    {times.prove != null && <span style={{ fontSize: 15, color: "#7a6a58", ...mono }}>{times.prove.toFixed(0)} ms</span>}
                    <button onClick={() => { navigator.clipboard.writeText(proof.proof_data!); setCopied(true); setTimeout(() => setCopied(false), 2000); }}
                      style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 14px", borderRadius: 10, border: "1px solid #3d2e1f", background: "transparent", color: "#b8a898", cursor: "pointer", fontSize: 14 }}>
                      {copied ? <Check size={16} color="#00ff88" /> : <Copy size={16} />}
                      {copied ? "Copied" : "Copy"}
                    </button>
                  </div>
                </div>
                {([
                  ["Trace Length", `${proof.trace_length} rows`],
                  ["FRI Layers", `${proof.fri_layers}`],
                  ["Queries", `${proof.num_queries}`],
                  ["Trace Root", proof.trace_root?.slice(0, 24) + "…"],
                  ["Quotient Root", proof.quotient_root?.slice(0, 24) + "…"],
                  ["Outputs", `[${proof.outputs?.join(", ")}]`],
                  ["Proof Size", `${((proof.proof_data?.length || 0) / 1024).toFixed(1)} KB`],
                ] as [string, string][]).map(([k, v]) => (
                  <div key={k} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "14px 0", borderBottom: "1px solid rgba(61,46,31,0.4)" }}>
                    <span style={{ fontSize: 16, color: "#7a6a58" }}>{k}</span>
                    <span style={{ fontSize: 15, color: k.includes("Root") ? "#4488ff" : "#f0e6d8", ...mono }}>{v}</span>
                  </div>
                ))}
              </div>
            ) : exec?.success && !proof ? (
              <div style={{ ...emptyCard, marginBottom: 28 }}>
                <Lock size={48} color="#554332" style={{ margin: "0 auto 20px" }} />
                <h3 style={{ fontSize: 22, fontWeight: 600, color: "#7a6a58", marginBottom: 12 }}>No Proof Yet</h3>
                <p style={{ fontSize: 16, color: "#7a6a58", lineHeight: 1.7 }}>Click <strong style={{ color: "#b8a898" }}>Prove</strong> to generate a STARK proof.</p>
              </div>
            ) : null}

            {vResult && (
              <div style={{
                ...card, padding: 28, marginBottom: 28,
                borderColor: vResult.valid ? "rgba(0,255,136,0.2)" : "rgba(255,68,102,0.2)",
                boxShadow: vResult.valid ? "0 0 40px rgba(0,255,136,0.06)" : "0 0 40px rgba(255,68,102,0.06)",
              }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 20 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                    {vResult.valid ? <CheckCircle2 size={28} color="#00ff88" /> : <XCircle size={28} color="#ff4466" />}
                    <h2 style={{ fontSize: 24, fontWeight: 800 }}>{vResult.valid ? "PROOF VERIFIED ✓" : "PROOF REJECTED ✗"}</h2>
                  </div>
                  {times.verify != null && <span style={{ fontSize: 15, color: "#7a6a58", ...mono }}>{times.verify.toFixed(0)} ms</span>}
                </div>
                {vResult.steps.map((s, i) => (
                  <div key={i} style={{ display: "flex", alignItems: "center", gap: 14, padding: "14px 18px", borderRadius: 12, background: "#2c1f14", marginBottom: 8 }}>
                    {s.passed ? <CheckCircle2 size={20} color="#00ff88" /> : <XCircle size={20} color="#ff4466" />}
                    <span style={{ fontSize: 16, fontWeight: 600, minWidth: 200 }}>{s.name}</span>
                    <span style={{ fontSize: 15, color: "#7a6a58", marginLeft: "auto" }}>{s.detail}</span>
                  </div>
                ))}
                {vResult.valid && (
                  <p style={{ fontSize: 15, color: "#b8a898", marginTop: 20, lineHeight: 1.8 }}>
                    The verifier confirmed all 49 AIR constraints are satisfied and the quotient polynomial is low-degree — without ever seeing the execution trace.
                  </p>
                )}
              </div>
            )}

            {!regs && !proof && !vResult && (
              <div style={{ ...emptyCard, marginTop: 28 }}>
                <Shield size={48} color="#554332" style={{ margin: "0 auto 20px" }} />
                <h3 style={{ fontSize: 22, fontWeight: 600, color: "#7a6a58", marginBottom: 12 }}>Proof Panel</h3>
                <p style={{ fontSize: 16, color: "#7a6a58", maxWidth: 400, margin: "0 auto", lineHeight: 1.7 }}>
                  STARK proof details and verification results appear here after running the pipeline.
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Trace (full width) */}
        {exec?.success && (
          <div style={{ ...card, overflow: "hidden" }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "20px 28px", borderBottom: "1px solid #3d2e1f" }}>
              <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                <Binary size={22} color="#ffbb00" />
                <h2 style={{ fontSize: 22, fontWeight: 700 }}>Execution Trace</h2>
                <span style={{ fontSize: 15, color: "#7a6a58" }}>{exec.real_len} real · {exec.padded_len} total</span>
              </div>
              <label style={{ display: "flex", alignItems: "center", gap: 10, fontSize: 15, color: "#7a6a58", cursor: "pointer" }}>
                <input type="checkbox" checked={showPad} onChange={e => setShowPad(e.target.checked)} style={{ accentColor: "#4488ff", width: 16, height: 16 }} />
                Show padding
              </label>
            </div>
            <div style={{ overflowX: "auto" }}>
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr style={{ background: "#2c1f14", borderBottom: "1px solid #3d2e1f" }}>
                    {["CLK", "PC", "Opcode", "Operation", "r0", "r1", "r2", "r3", "r4", "r5", "r6", "Write"].map((h, i) => (
                      <th key={h} style={{ padding: "14px 18px", fontSize: 15, fontWeight: 600, color: "#7a6a58", textAlign: i >= 4 ? "right" : "left", whiteSpace: "nowrap" }}>{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {rows.map((r, i) => (
                    <tr key={i} style={{ borderBottom: "1px solid rgba(61,46,31,0.3)", opacity: r.is_padding ? 0.35 : 1 }}>
                      <td style={{ padding: "12px 18px", fontSize: 15, color: "#7a6a58", ...mono }}>{r.clk}</td>
                      <td style={{ padding: "12px 18px", fontSize: 15, ...mono }}>{r.pc}</td>
                      <td style={{ padding: "12px 18px" }}>
                        <span style={{
                          display: "inline-block", padding: "5px 12px", borderRadius: 8, fontSize: 13, fontWeight: 700, ...mono,
                          background: r.opcode_name === "HALT" ? "rgba(255,68,102,0.12)" : r.opcode_name === "ASSERT_EQ" ? "rgba(255,187,0,0.12)" : r.opcode_name === "NOP" ? "rgba(122,106,88,0.1)" : "rgba(68,136,255,0.12)",
                          color: r.opcode_name === "HALT" ? "#ff4466" : r.opcode_name === "ASSERT_EQ" ? "#ffbb00" : r.opcode_name === "NOP" ? "#7a6a58" : "#4488ff",
                        }}>{r.opcode_name}</span>
                      </td>
                      <td style={{ padding: "12px 18px", fontSize: 15, color: "#b8a898" }}>{fmtOp(r)}</td>
                      {r.regs.map((v, j) => (
                        <td key={j} style={{ padding: "12px 18px", fontSize: 15, textAlign: "right", color: v ? "#00ddff" : "rgba(122,106,88,0.4)", ...mono }}>{v}</td>
                      ))}
                      <td style={{ padding: "12px 18px", fontSize: 15, textAlign: "right", fontWeight: r.write_val ? 600 : 400, color: r.write_val ? "#00ff88" : "rgba(122,106,88,0.4)", ...mono }}>{r.write_val}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </main>

      <footer style={{ borderTop: "1px solid #3d2e1f", padding: "32px 0", textAlign: "center", fontSize: 15, color: "#7a6a58" }}>
        MM2 STARK Proof System · Goldilocks Field · 49 AIR Constraints · FRI · SHA-256 Merkle · WebAssembly
      </footer>
    </div>
  );
}

function fmtOp(r: JsTraceRow): string {
  switch (r.opcode_name) {
    case "IMM": return `r${r.rd} ← ${r.imm}`;
    case "ADD": return `r${r.rd} ← r${r.rs1} + r${r.rs2}`;
    case "MUL": return `r${r.rd} ← r${r.rs1} × r${r.rs2}`;
    case "SUB": return `r${r.rd} ← r${r.rs1} − r${r.rs2}`;
    case "ADDI": return `r${r.rd} ← r${r.rs1} + ${r.imm}`;
    case "ASSERT_EQ": return `r${r.rs1} = r${r.rs2} ?`;
    default: return "";
  }
}
