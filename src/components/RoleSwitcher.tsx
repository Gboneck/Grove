import { useState, useEffect, useCallback } from "react";
import { listRoles, getActiveRole, setActiveRole } from "../lib/tauri";
import type { Role } from "../lib/tauri";

interface RoleSwitcherProps {
  onRoleChange?: (roleName: string | null) => void;
}

export function RoleSwitcher({ onRoleChange }: RoleSwitcherProps) {
  const [roles, setRoles] = useState<Role[]>([]);
  const [activeRole, setActiveRoleState] = useState<string | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [loadedRoles, currentRole] = await Promise.all([
          listRoles(),
          getActiveRole(),
        ]);
        setRoles(loadedRoles);
        setActiveRoleState(currentRole);
      } catch (e) {
        console.error("[RoleSwitcher] Failed to load roles:", e);
      } finally {
        setIsLoading(false);
      }
    }
    load();
  }, []);

  const handleSelect = useCallback(
    async (roleName: string | null) => {
      try {
        await setActiveRole(roleName);
        setActiveRoleState(roleName);
        setIsOpen(false);
        onRoleChange?.(roleName);
      } catch (e) {
        console.error("[RoleSwitcher] Failed to set role:", e);
      }
    },
    [onRoleChange]
  );

  if (isLoading || roles.length === 0) {
    return null;
  }

  const current = roles.find((r) => r.name === activeRole);

  return (
    <div className="role-switcher" style={{ position: "relative" }}>
      <button
        className="role-switcher-trigger"
        onClick={() => setIsOpen(!isOpen)}
        style={{
          background: "transparent",
          border: "1px solid #333",
          borderRadius: "6px",
          padding: "4px 10px",
          color: activeRole ? "#d4a853" : "#888",
          fontFamily: "Syne, sans-serif",
          fontSize: "12px",
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          gap: "6px",
        }}
      >
        <span style={{ fontSize: "14px" }}>
          {current ? current.display : "No Role"}
        </span>
        <span style={{ fontSize: "10px", opacity: 0.6 }}>
          {isOpen ? "\u25B2" : "\u25BC"}
        </span>
      </button>

      {isOpen && (
        <div
          className="role-switcher-dropdown"
          style={{
            position: "absolute",
            top: "calc(100% + 4px)",
            right: 0,
            background: "#1a1a1a",
            border: "1px solid #333",
            borderRadius: "8px",
            padding: "4px",
            minWidth: "200px",
            zIndex: 100,
            boxShadow: "0 4px 20px rgba(0,0,0,0.5)",
          }}
        >
          {/* No role option */}
          <button
            onClick={() => handleSelect(null)}
            style={{
              display: "block",
              width: "100%",
              textAlign: "left",
              background: activeRole === null ? "#2a2a2a" : "transparent",
              border: "none",
              borderRadius: "6px",
              padding: "8px 10px",
              color: "#888",
              fontFamily: "Syne, sans-serif",
              fontSize: "12px",
              cursor: "pointer",
            }}
          >
            <div style={{ fontWeight: 500 }}>No Role</div>
            <div style={{ fontSize: "11px", opacity: 0.6, marginTop: "2px" }}>
              Default reasoning mode
            </div>
          </button>

          {roles.map((role) => (
            <button
              key={role.name}
              onClick={() => handleSelect(role.name)}
              style={{
                display: "block",
                width: "100%",
                textAlign: "left",
                background:
                  activeRole === role.name ? "#2a2a2a" : "transparent",
                border: "none",
                borderRadius: "6px",
                padding: "8px 10px",
                color: activeRole === role.name ? "#d4a853" : "#ccc",
                fontFamily: "Syne, sans-serif",
                fontSize: "12px",
                cursor: "pointer",
              }}
            >
              <div style={{ fontWeight: 500 }}>{role.display}</div>
              <div
                style={{ fontSize: "11px", opacity: 0.6, marginTop: "2px" }}
              >
                {role.description}
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
