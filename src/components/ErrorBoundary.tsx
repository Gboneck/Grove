import { Component, ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("[grove] Error boundary caught:", error, info.componentStack);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div
          className="min-h-screen flex items-center justify-center bg-[#0a0a0a]"
          role="alert"
        >
          <div className="max-w-md text-center p-8">
            <h1
              className="text-2xl font-display text-[#d4a853] mb-4"
              style={{ fontFamily: "Instrument Serif, serif" }}
            >
              Grove encountered an error
            </h1>
            <p
              className="text-gray-400 mb-6 text-sm"
              style={{ fontFamily: "Syne, sans-serif" }}
            >
              {this.state.error?.message || "Something went wrong."}
            </p>
            <button
              onClick={() => {
                this.setState({ hasError: false, error: null });
                window.location.reload();
              }}
              className="px-6 py-2 bg-[#d4a853] text-[#0a0a0a] rounded-lg text-sm font-medium hover:brightness-110 transition-all"
              style={{ fontFamily: "Syne, sans-serif" }}
            >
              Reload Grove
            </button>
            <p
              className="text-gray-600 mt-4 text-xs"
              style={{ fontFamily: "JetBrains Mono, monospace" }}
            >
              {this.state.error?.stack?.split("\n")[0]}
            </p>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
