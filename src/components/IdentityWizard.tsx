import { useState } from "react";
import { generateSoul } from "../lib/tauri";

interface IdentityWizardProps {
  onComplete: () => void;
}

type Step = "name" | "role" | "projects" | "priorities" | "style" | "generating";

export default function IdentityWizard({ onComplete }: IdentityWizardProps) {
  const [step, setStep] = useState<Step>("name");
  const [name, setName] = useState("");
  const [location, setLocation] = useState("");
  const [role, setRole] = useState("");
  const [projects, setProjects] = useState<string[]>([""]);
  const [priorities, setPriorities] = useState<string[]>([""]);
  const [workStyle, setWorkStyle] = useState("");
  const [error, setError] = useState("");

  const handleGenerate = async () => {
    setStep("generating");
    setError("");
    try {
      await generateSoul(
        name,
        location || null,
        role || null,
        projects.filter((p) => p.trim()),
        priorities.filter((p) => p.trim()),
        workStyle || null
      );
      onComplete();
    } catch (e) {
      setError(String(e));
      setStep("style");
    }
  };

  const addProject = () => setProjects([...projects, ""]);
  const updateProject = (i: number, val: string) => {
    const next = [...projects];
    next[i] = val;
    setProjects(next);
  };

  const addPriority = () => setPriorities([...priorities, ""]);
  const updatePriority = (i: number, val: string) => {
    const next = [...priorities];
    next[i] = val;
    setPriorities(next);
  };

  const canAdvance = () => {
    if (step === "name") return name.trim().length > 0;
    return true;
  };

  const next = () => {
    const order: Step[] = ["name", "role", "projects", "priorities", "style"];
    const idx = order.indexOf(step);
    if (idx < order.length - 1) {
      setStep(order[idx + 1]);
    } else {
      handleGenerate();
    }
  };

  const back = () => {
    const order: Step[] = ["name", "role", "projects", "priorities", "style"];
    const idx = order.indexOf(step);
    if (idx > 0) setStep(order[idx - 1]);
  };

  const stepIndex = ["name", "role", "projects", "priorities", "style"].indexOf(step);
  const totalSteps = 5;

  if (step === "generating") {
    return (
      <div className="min-h-screen flex items-center justify-center p-8">
        <div className="text-center space-y-4">
          <div className="w-8 h-8 rounded-full bg-grove-accent/30 mx-auto animate-pulse" />
          <p className="text-grove-text-secondary text-sm">
            Building your identity...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-8">
      <div className="max-w-md w-full space-y-8">
        {/* Header */}
        <div className="text-center space-y-2">
          <h1 className="text-3xl font-display text-grove-accent">Grove</h1>
          <p className="text-grove-text-secondary text-sm">
            Let's set up your identity
          </p>
          {/* Progress */}
          <div className="flex gap-1.5 justify-center pt-2">
            {Array.from({ length: totalSteps }).map((_, i) => (
              <div
                key={i}
                className={`h-1 w-8 rounded-full transition-colors ${
                  i <= stepIndex ? "bg-grove-accent" : "bg-grove-border"
                }`}
              />
            ))}
          </div>
        </div>

        {/* Step content */}
        <div className="space-y-4">
          {step === "name" && (
            <>
              <label className="block text-sm text-grove-text-primary font-medium">
                What's your name?
              </label>
              <input
                autoFocus
                value={name}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && canAdvance() && next()}
                placeholder="Your name"
                className="w-full bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors text-sm"
              />
              <label className="block text-sm text-grove-text-secondary mt-3">
                Where are you based? <span className="opacity-50">(optional)</span>
              </label>
              <input
                value={location}
                onChange={(e) => setLocation(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && canAdvance() && next()}
                placeholder="City, country"
                className="w-full bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors text-sm"
              />
            </>
          )}

          {step === "role" && (
            <>
              <label className="block text-sm text-grove-text-primary font-medium">
                What do you do?
              </label>
              <p className="text-xs text-grove-text-secondary">
                Your role, profession, or how you'd describe yourself.
              </p>
              <input
                autoFocus
                value={role}
                onChange={(e) => setRole(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && next()}
                placeholder="e.g. Software engineer, Designer, Student..."
                className="w-full bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors text-sm"
              />
            </>
          )}

          {step === "projects" && (
            <>
              <label className="block text-sm text-grove-text-primary font-medium">
                What are you working on?
              </label>
              <p className="text-xs text-grove-text-secondary">
                List your current projects, ventures, or areas of focus.
              </p>
              {projects.map((p, i) => (
                <input
                  key={i}
                  autoFocus={i === 0}
                  value={p}
                  onChange={(e) => updateProject(i, e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && p.trim()) addProject();
                  }}
                  placeholder={`Project ${i + 1}`}
                  className="w-full bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors text-sm"
                />
              ))}
              <button
                onClick={addProject}
                className="text-xs text-grove-accent hover:text-grove-accent/80 transition-colors"
              >
                + Add another
              </button>
            </>
          )}

          {step === "priorities" && (
            <>
              <label className="block text-sm text-grove-text-primary font-medium">
                What matters most right now?
              </label>
              <p className="text-xs text-grove-text-secondary">
                Your top priorities — what should Grove help you focus on?
              </p>
              {priorities.map((p, i) => (
                <input
                  key={i}
                  autoFocus={i === 0}
                  value={p}
                  onChange={(e) => updatePriority(i, e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && p.trim()) addPriority();
                  }}
                  placeholder={`Priority ${i + 1}`}
                  className="w-full bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors text-sm"
                />
              ))}
              <button
                onClick={addPriority}
                className="text-xs text-grove-accent hover:text-grove-accent/80 transition-colors"
              >
                + Add another
              </button>
            </>
          )}

          {step === "style" && (
            <>
              <label className="block text-sm text-grove-text-primary font-medium">
                How do you like to work?
              </label>
              <p className="text-xs text-grove-text-secondary">
                Describe your work style, preferences, or anything Grove should
                know about how you operate. <span className="opacity-50">(optional)</span>
              </p>
              <textarea
                autoFocus
                value={workStyle}
                onChange={(e) => setWorkStyle(e.target.value)}
                rows={4}
                placeholder="e.g. I prefer deep focus blocks in the morning, keep things concise, I like structured plans..."
                className="w-full bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors text-sm resize-none"
              />
            </>
          )}
        </div>

        {error && (
          <p className="text-xs text-grove-status-red">{error}</p>
        )}

        {/* Navigation */}
        <div className="flex gap-3">
          {stepIndex > 0 && (
            <button
              onClick={back}
              className="px-6 py-3 rounded-lg text-sm text-grove-text-secondary hover:text-grove-text-primary border border-grove-border hover:border-grove-accent/40 transition-all"
            >
              Back
            </button>
          )}
          <button
            onClick={next}
            disabled={!canAdvance()}
            className="flex-1 bg-grove-accent text-grove-bg py-3 rounded-lg font-medium hover:brightness-110 transition-all disabled:opacity-50"
          >
            {step === "style" ? "Create My Identity" : "Continue"}
          </button>
        </div>

        {/* Skip */}
        {step === "name" && (
          <button
            onClick={onComplete}
            className="w-full text-xs text-grove-text-secondary hover:text-grove-text-primary transition-colors py-1"
          >
            Skip — I'll edit soul.md manually
          </button>
        )}
      </div>
    </div>
  );
}
