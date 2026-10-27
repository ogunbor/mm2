"use client";

import Link from "next/link";
import {
  Shield, Cpu, Layers, Lock, GitBranch, ArrowRight,
  ChevronRight, Hash, Binary, Zap, Eye, CheckCircle2,
  Box, FileCode, Database,
} from "lucide-react";

const mono = { fontFamily: "'JetBrains Mono', 'Fira Code', monospace" };
const card = { background: "#241a11", border: "1px solid #3d2e1f", borderRadius: 16 };
const section = { padding: "100px 32px", maxWidth: 1200, margin: "0 auto" };

export default function LandingPage() {
  return (
    <div style={{ background: "#1a120b", color: "#f0e6d8", minHeight: "100vh" }}>

      {/* ═══════════ NAV ═══════════ */}
      <nav style={{
        position: "fixed", top: 0, left: 0, right: 0, zIndex: 50,
        padding: "16px 40px", display: "flex", alignItems: "center", justifyContent: "space-between",
        background: "rgba(33,23,16,0.85)", backdropFilter: "blur(12px)", borderBottom: "1px solid #3d2e1f",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <div style={{ width: 36, height: 36, borderRadius: 10, background: "linear-gradient(135deg, #00ff88, #4488ff)", display: "flex", alignItems: "center", justifyContent: "center" }}>
            <Shield size={20} color="#000" />
          </div>
          <span style={{ fontSize: 20, fontWeight: 800 }}>MM2</span>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 32 }}>
          <a href="#how-it-works" style={{ fontSize: 15, color: "#b8a898", textDecoration: "none" }}>How It Works</a>
          <a href="#architecture" style={{ fontSize: 15, color: "#b8a898", textDecoration: "none" }}>Architecture</a>
          <a href="#isa" style={{ fontSize: 15, color: "#b8a898", textDecoration: "none" }}>ISA</a>
          <Link href="/explorer" style={{
            padding: "10px 24px", borderRadius: 10, fontSize: 15, fontWeight: 600,
            background: "linear-gradient(135deg, #00ff88, #4488ff)", color: "#000", textDecoration: "none",
          }}>
            Launch Explorer →
          </Link>
        </div>
      </nav>

      {/* ═══════════ HERO ═══════════ */}
      <section style={{
        minHeight: "100vh", display: "flex", flexDirection: "column",
        alignItems: "center", justifyContent: "center", padding: "120px 32px 80px",
        backgroundImage: "linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px)",
        backgroundSize: "60px 60px", position: "relative",
      }}>
        <div style={{ position: "absolute", top: "30%", left: "50%", transform: "translate(-50%,-50%)", width: 600, height: 600, borderRadius: "50%", background: "#c8956a", opacity: 0.06, filter: "blur(140px)", pointerEvents: "none" }} />
        <div style={{ position: "absolute", bottom: "20%", right: "20%", width: 400, height: 400, borderRadius: "50%", background: "#8855ff", opacity: 0.04, filter: "blur(100px)", pointerEvents: "none" }} />

        <div style={{ textAlign: "center", maxWidth: 900, position: "relative", zIndex: 1 }}>
          <div style={{ display: "inline-flex", alignItems: "center", gap: 10, padding: "10px 22px", borderRadius: 999, border: "1px solid #554332", background: "#2c1f14", marginBottom: 40 }}>
            <Shield size={18} color="#00ff88" />
            <span style={{ fontSize: 15, color: "#b8a898" }}>Zero-Knowledge Proof System</span>
          </div>

          <h1 style={{
            fontSize: "clamp(72px, 12vw, 140px)", fontWeight: 900, letterSpacing: "-0.04em",
            lineHeight: 1, marginBottom: 24,
            background: "linear-gradient(135deg, #00ff88, #4488ff, #8855ff)",
            WebkitBackgroundClip: "text", WebkitTextFillColor: "transparent",
          }}>
            MM2
          </h1>

          <p style={{ fontSize: 30, fontWeight: 700, color: "#f0e6d8", marginBottom: 20, lineHeight: 1.3 }}>
            Interactive STARK Proof Explorer
          </p>

          <p style={{ fontSize: 19, color: "#b8a898", maxWidth: 700, margin: "0 auto 50px", lineHeight: 1.8 }}>
            A complete, from-scratch STARK proof system. Write assembly programs, execute them
            on a custom virtual machine, generate cryptographic proofs, and verify them — all
            running in your browser via WebAssembly. No servers, no trust required.
          </p>

          <div style={{ display: "flex", justifyContent: "center", gap: 16, marginBottom: 60 }}>
            <Link href="/explorer" style={{
              display: "inline-flex", alignItems: "center", gap: 12,
              padding: "20px 44px", borderRadius: 14, fontSize: 19, fontWeight: 700,
              background: "linear-gradient(135deg, #00ff88, #4488ff)", color: "#000", textDecoration: "none",
            }}>
              Launch Explorer <ArrowRight size={22} />
            </Link>
            <a href="#how-it-works" style={{
              display: "inline-flex", alignItems: "center", gap: 10,
              padding: "20px 36px", borderRadius: 14, fontSize: 19, fontWeight: 600,
              background: "#2c1f14", border: "1px solid #3d2e1f", color: "#f0e6d8", textDecoration: "none",
            }}>
              Learn More
            </a>
          </div>

          {/* Stats row */}
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 20, maxWidth: 800, margin: "0 auto" }}>
            {[
              { v: "8", l: "Instructions", I: Cpu },
              { v: "49", l: "AIR Constraints", I: Layers },
              { v: "30", l: "FRI Queries", I: GitBranch },
              { v: "256-bit", l: "SHA-256 Merkle", I: Lock },
            ].map(({ v, l, I }) => (
              <div key={l} style={{ ...card, padding: "28px 16px", textAlign: "center" }}>
                <I size={24} color="#4488ff" style={{ margin: "0 auto 12px" }} />
                <div style={{ fontSize: 30, fontWeight: 800, ...mono }}>{v}</div>
                <div style={{ fontSize: 14, color: "#7a6a58", marginTop: 8 }}>{l}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ═══════════ WHAT IS A STARK ═══════════ */}
      <section style={{ ...section, borderTop: "1px solid #3d2e1f" }} id="how-it-works">
        <div style={{ textAlign: "center", marginBottom: 64 }}>
          <span style={{ fontSize: 14, fontWeight: 600, color: "#00ff88", letterSpacing: 2, textTransform: "uppercase" }}>Understanding</span>
          <h2 style={{ fontSize: 44, fontWeight: 800, marginTop: 12, lineHeight: 1.2 }}>What Is a STARK?</h2>
          <p style={{ fontSize: 18, color: "#b8a898", maxWidth: 700, margin: "20px auto 0", lineHeight: 1.8 }}>
            A STARK (Scalable Transparent ARgument of Knowledge) is a cryptographic proof system
            that lets you prove a computation was done correctly — without revealing the computation itself.
          </p>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 24 }}>
          {[
            {
              I: Eye, c: "#00ff88", title: "Zero Knowledge",
              text: "The verifier learns nothing about the execution trace — only that it satisfies the algebraic constraints. Your computation remains private.",
            },
            {
              I: Zap, c: "#4488ff", title: "Succinctness",
              text: "The proof is exponentially smaller than the computation. A trace with thousands of rows produces a proof that can be checked in milliseconds.",
            },
            {
              I: Shield, c: "#8855ff", title: "Transparency",
              text: "No trusted setup required. All randomness is derived via Fiat-Shamir from public data. Anyone can verify without trusting a third party.",
            },
          ].map(({ I, c, title, text }) => (
            <div key={title} style={{ ...card, padding: 32 }}>
              <div style={{ width: 52, height: 52, borderRadius: 14, background: `${c}15`, display: "flex", alignItems: "center", justifyContent: "center", marginBottom: 20 }}>
                <I size={26} color={c} />
              </div>
              <h3 style={{ fontSize: 22, fontWeight: 700, marginBottom: 12 }}>{title}</h3>
              <p style={{ fontSize: 16, color: "#b8a898", lineHeight: 1.7 }}>{text}</p>
            </div>
          ))}
        </div>
      </section>

      {/* ═══════════ THE PIPELINE ═══════════ */}
      <section style={{ ...section, borderTop: "1px solid #3d2e1f" }} id="architecture">
        <div style={{ textAlign: "center", marginBottom: 64 }}>
          <span style={{ fontSize: 14, fontWeight: 600, color: "#4488ff", letterSpacing: 2, textTransform: "uppercase" }}>Architecture</span>
          <h2 style={{ fontSize: 44, fontWeight: 800, marginTop: 12, lineHeight: 1.2 }}>The STARK Pipeline</h2>
          <p style={{ fontSize: 18, color: "#b8a898", maxWidth: 700, margin: "20px auto 0", lineHeight: 1.8 }}>
            From writing a program to verifying a proof — every step runs in your browser.
          </p>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr", gap: 0 }}>
          {[
            {
              step: "01", title: "Write a Program", icon: FileCode, color: "#4488ff",
              desc: "Write assembly using our 8-instruction ISA. The VM has 7 registers and operates over the Goldilocks prime field (p = 2⁶⁴ − 2³² + 1). Supported operations include arithmetic (ADD, MUL, SUB), immediates (IMM, ADDI), assertions (ASSERT_EQ), and control flow (HALT).",
            },
            {
              step: "02", title: "Execute on the VM", icon: Cpu, color: "#00ddff",
              desc: "The virtual machine executes your program instruction by instruction, recording every state change into an execution trace. Each row captures the clock cycle, program counter, opcode, register values, and the computed write value. The trace is then padded to the next power of 2.",
            },
            {
              step: "03", title: "Check AIR Constraints", icon: Layers, color: "#ffbb00",
              desc: "The Algebraic Intermediate Representation defines 49 polynomial constraints (degree ≤ 3) that any valid trace must satisfy. These include flag booleanity, register read/write consistency, arithmetic correctness, clock advancement, and assertion checking. If any constraint fails, the trace is invalid.",
            },
            {
              step: "04", title: "Interpolate & Commit", icon: Database, color: "#8855ff",
              desc: "Each of the 46 trace columns is interpolated into a polynomial over the trace domain. These polynomials are then evaluated on a larger LDE (Low Degree Extension) domain with an 8× blowup factor. The evaluations are committed via a SHA-256 Merkle tree, producing the trace root.",
            },
            {
              step: "05", title: "Compute Quotient", icon: Hash, color: "#ff4466",
              desc: "All 49 constraints are combined into a single composition polynomial using random challenges from the Fiat-Shamir transcript. This is divided by the vanishing polynomial Z_H(x) = x^n − 1 to produce the quotient Q(x). If the constraints hold, Q(x) is a low-degree polynomial.",
            },
            {
              step: "06", title: "FRI Low-Degree Test", icon: GitBranch, color: "#00ff88",
              desc: "The FRI (Fast Reed-Solomon Interactive Oracle Proof) protocol verifies that Q(x) is indeed low-degree without revealing it. The polynomial is repeatedly folded using random challenges, halving the degree each time, until a constant remains. 30 queries provide ~90 bits of security.",
            },
            {
              step: "07", title: "Verify the Proof", icon: Shield, color: "#00ff88",
              desc: "The verifier re-derives all Fiat-Shamir challenges, checks the FRI proof, verifies Merkle openings for both trace and quotient, and confirms C(x) = Q(x) · Z_H(x) at every query point. All of this happens without ever seeing the execution trace — only the commitments and openings.",
            },
          ].map(({ step, title, icon: Icon, color, desc }, i) => (
            <div key={step} style={{ display: "flex", gap: 28, padding: "36px 0", borderBottom: i < 6 ? "1px solid #3d2e1f" : "none" }}>
              <div style={{ flexShrink: 0, width: 64, textAlign: "center" }}>
                <div style={{ fontSize: 14, fontWeight: 700, color, ...mono, marginBottom: 10 }}>{step}</div>
                <div style={{ width: 52, height: 52, borderRadius: 14, background: `${color}12`, display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto" }}>
                  <Icon size={24} color={color} />
                </div>
              </div>
              <div style={{ flex: 1 }}>
                <h3 style={{ fontSize: 22, fontWeight: 700, marginBottom: 10 }}>{title}</h3>
                <p style={{ fontSize: 16, color: "#b8a898", lineHeight: 1.8 }}>{desc}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ═══════════ ISA REFERENCE ═══════════ */}
      <section style={{ ...section, borderTop: "1px solid #3d2e1f" }} id="isa">
        <div style={{ textAlign: "center", marginBottom: 64 }}>
          <span style={{ fontSize: 14, fontWeight: 600, color: "#8855ff", letterSpacing: 2, textTransform: "uppercase" }}>Instruction Set</span>
          <h2 style={{ fontSize: 44, fontWeight: 800, marginTop: 12, lineHeight: 1.2 }}>The MM2 ISA</h2>
          <p style={{ fontSize: 18, color: "#b8a898", maxWidth: 700, margin: "20px auto 0", lineHeight: 1.8 }}>
            A minimal but complete instruction set for provable computation. 7 registers (r0–r6),
            all arithmetic in the Goldilocks field.
          </p>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, maxWidth: 900, margin: "0 auto" }}>
          {[
            { op: "IMM", fmt: "IMM rd, imm", d: "Load an immediate value into register rd", c: "#4488ff" },
            { op: "ADD", fmt: "ADD rd, rs1, rs2", d: "Add rs1 and rs2, store result in rd", c: "#4488ff" },
            { op: "MUL", fmt: "MUL rd, rs1, rs2", d: "Multiply rs1 by rs2, store in rd", c: "#4488ff" },
            { op: "SUB", fmt: "SUB rd, rs1, rs2", d: "Subtract rs2 from rs1, store in rd", c: "#4488ff" },
            { op: "ADDI", fmt: "ADDI rd, rs1, imm", d: "Add immediate to rs1, store in rd", c: "#4488ff" },
            { op: "ASSERT_EQ", fmt: "ASSERT_EQ rs1, rs2", d: "Require rs1 equals rs2 or proof fails", c: "#ffbb00" },
            { op: "HALT", fmt: "HALT", d: "Terminate program execution", c: "#ff4466" },
            { op: "NOP", fmt: "NOP", d: "No operation — used for trace padding only", c: "#7a6a58" },
          ].map(({ op, fmt, d, c }) => (
            <div key={op} style={{ ...card, padding: "24px 28px", display: "flex", flexDirection: "column", gap: 8 }}>
              <code style={{ fontSize: 17, fontWeight: 700, color: c, ...mono }}>{fmt}</code>
              <p style={{ fontSize: 15, color: "#b8a898", lineHeight: 1.6 }}>{d}</p>
            </div>
          ))}
        </div>
      </section>

      {/* ═══════════ TECHNICAL DETAILS ═══════════ */}
      <section style={{ ...section, borderTop: "1px solid #3d2e1f" }}>
        <div style={{ textAlign: "center", marginBottom: 64 }}>
          <span style={{ fontSize: 14, fontWeight: 600, color: "#ffbb00", letterSpacing: 2, textTransform: "uppercase" }}>Under the Hood</span>
          <h2 style={{ fontSize: 44, fontWeight: 800, marginTop: 12, lineHeight: 1.2 }}>Technical Specifications</h2>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 }}>
          {[
            { title: "Goldilocks Field", items: ["Prime: p = 2⁶⁴ − 2³² + 1", "64-bit native arithmetic", "Efficient roots of unity (up to 2³²)", "Primitive root: 7"] },
            { title: "Execution Trace", items: ["46 columns per row", "7 register columns + 3 derived values", "One-hot encoded opcode and register flags", "Padded to next power of 2"] },
            { title: "AIR Constraints", items: ["40 local constraints (single row)", "9 transition constraints (row pairs)", "Maximum degree: 3", "Random linear combination via α"] },
            { title: "STARK Proof", items: ["8× blowup factor for LDE", "SHA-256 Merkle tree commitments", "Fiat-Shamir non-interactive transcript", "30 FRI queries (~90 bits security)"] },
          ].map(({ title, items }) => (
            <div key={title} style={{ ...card, padding: 32 }}>
              <h3 style={{ fontSize: 20, fontWeight: 700, marginBottom: 18 }}>{title}</h3>
              <ul style={{ listStyle: "none", padding: 0, display: "flex", flexDirection: "column", gap: 12 }}>
                {items.map(item => (
                  <li key={item} style={{ display: "flex", alignItems: "center", gap: 12, fontSize: 16, color: "#b8a898", lineHeight: 1.6 }}>
                    <CheckCircle2 size={18} color="#00ff88" style={{ flexShrink: 0 }} />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </section>

      {/* ═══════════ EXAMPLE ═══════════ */}
      <section style={{ ...section, borderTop: "1px solid #3d2e1f" }}>
        <div style={{ textAlign: "center", marginBottom: 64 }}>
          <span style={{ fontSize: 14, fontWeight: 600, color: "#00ddff", letterSpacing: 2, textTransform: "uppercase" }}>Example</span>
          <h2 style={{ fontSize: 44, fontWeight: 800, marginTop: 12, lineHeight: 1.2 }}>Nicomachus's Theorem</h2>
          <p style={{ fontSize: 18, color: "#b8a898", maxWidth: 700, margin: "20px auto 0", lineHeight: 1.8 }}>
            The default demo proves that 1³ + 2³ + 3³ + 4³ = (1 + 2 + 3 + 4)² — both sides equal 100.
            This is an instance of Nicomachus's theorem, which states that the sum of the first n cubes
            equals the square of the sum of the first n integers.
          </p>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 32 }}>
          <div style={{ ...card, padding: 32 }}>
            <h3 style={{ fontSize: 20, fontWeight: 700, marginBottom: 16, display: "flex", alignItems: "center", gap: 10 }}>
              <FileCode size={22} color="#4488ff" /> Program (26 instructions)
            </h3>
            <pre style={{ fontSize: 14, color: "#b8a898", lineHeight: 1.9, ...mono, overflow: "auto", background: "#1a120b", borderRadius: 12, padding: 20, border: "1px solid #3d2e1f" }}>{`IMM r0, 0    // k = 0
IMM r1, 0    // sum_cubes = 0
IMM r2, 0    // sum_k = 0

// For k = 1, 2, 3, 4:
ADDI r0, r0, 1    // k++
MUL  r3, r0, r0   // r3 = k²
MUL  r3, r3, r0   // r3 = k³
ADD  r1, r1, r3   // sum_cubes += k³
ADD  r2, r2, r0   // sum_k += k
// ... repeated 4 times ...

MUL  r4, r2, r2       // r4 = sum_k²
ASSERT_EQ r1, r4      // 100 == 100 ✓
HALT`}</pre>
          </div>

          <div style={{ ...card, padding: 32 }}>
            <h3 style={{ fontSize: 20, fontWeight: 700, marginBottom: 16, display: "flex", alignItems: "center", gap: 10 }}>
              <CheckCircle2 size={22} color="#00ff88" /> What Gets Proved
            </h3>
            <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
              <div style={{ background: "#1a120b", borderRadius: 12, padding: 24, border: "1px solid #3d2e1f", textAlign: "center" }}>
                <div style={{ fontSize: 28, fontWeight: 800, ...mono, color: "#00ff88", marginBottom: 8 }}>
                  1³ + 2³ + 3³ + 4³ = (1+2+3+4)²
                </div>
                <div style={{ fontSize: 18, color: "#b8a898", ...mono }}>
                  1 + 8 + 27 + 64 = 10² = <span style={{ color: "#00ddff", fontWeight: 700 }}>100</span>
                </div>
              </div>
              <p style={{ fontSize: 16, color: "#b8a898", lineHeight: 1.8 }}>
                The prover generates a STARK proof that a committed execution trace satisfies
                all 49 algebraic constraints. The verifier checks this proof in milliseconds
                without ever seeing the trace — only Merkle roots and FRI query responses.
              </p>
              <p style={{ fontSize: 16, color: "#b8a898", lineHeight: 1.8 }}>
                Try the <strong style={{ color: "#ff4466" }}>False Claim</strong> demo in the explorer to see
                what happens when ASSERT_EQ fails — the verifier correctly rejects the proof.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* ═══════════ CTA ═══════════ */}
      <section style={{ padding: "120px 32px", textAlign: "center", borderTop: "1px solid #3d2e1f", position: "relative" }}>
        <div style={{ position: "absolute", top: "50%", left: "50%", transform: "translate(-50%,-50%)", width: 500, height: 500, borderRadius: "50%", background: "#00ff88", opacity: 0.03, filter: "blur(120px)", pointerEvents: "none" }} />
        <div style={{ position: "relative", zIndex: 1 }}>
          <h2 style={{ fontSize: 48, fontWeight: 900, marginBottom: 20, lineHeight: 1.2 }}>Try It Yourself</h2>
          <p style={{ fontSize: 19, color: "#b8a898", maxWidth: 600, margin: "0 auto 40px", lineHeight: 1.8 }}>
            Write programs, watch the VM execute, generate real STARK proofs, and verify them.
            Everything runs locally in your browser — no servers, no trust assumptions.
          </p>
          <Link href="/explorer" style={{
            display: "inline-flex", alignItems: "center", gap: 14,
            padding: "22px 52px", borderRadius: 16, fontSize: 20, fontWeight: 800,
            background: "linear-gradient(135deg, #00ff88, #4488ff)", color: "#000", textDecoration: "none",
          }}>
            Launch the Explorer <ArrowRight size={24} />
          </Link>
        </div>
      </section>

      {/* ═══════════ FOOTER ═══════════ */}
      <footer style={{ borderTop: "1px solid #3d2e1f", padding: "40px 32px", textAlign: "center" }}>
        <div style={{ fontSize: 15, color: "#7a6a58", lineHeight: 1.8 }}>
          MM2 STARK Proof System — Built from scratch in Rust, compiled to WebAssembly
        </div>
        <div style={{ fontSize: 14, color: "#554332", marginTop: 8 }}>
          Goldilocks Field (p = 2⁶⁴ − 2³² + 1) · 49 AIR Constraints · FRI Protocol · SHA-256 Merkle Trees
        </div>
      </footer>
    </div>
  );
}
