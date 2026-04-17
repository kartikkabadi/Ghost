import type { CSSProperties, ReactNode } from 'react';

const cardStyle: CSSProperties = {
  border: '1px solid var(--ghost-border)',
  background: 'linear-gradient(180deg, rgba(24,24,27,0.88) 0%, rgba(17,17,19,0.94) 100%)',
  borderRadius: 24,
  boxShadow: '0 28px 80px rgba(0, 0, 0, 0.36), inset 0 1px 0 rgba(255,255,255,0.03)',
  backdropFilter: 'blur(18px)'
};

export function cn(...values: Array<string | undefined | false | null>) {
  return values.filter(Boolean).join(' ');
}

export function Card({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return <section style={{ ...cardStyle, ...style }}>{children}</section>;
}

export function CardHeader({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return <div style={{ display: 'flex', flexDirection: 'column', gap: 8, padding: '24px 24px 0', ...style }}>{children}</div>;
}

export function CardTitle({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return <h2 style={{ margin: 0, fontSize: 18, lineHeight: 1.2, letterSpacing: '-0.02em', ...style }}>{children}</h2>;
}

export function CardDescription({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return <p style={{ margin: 0, color: 'var(--ghost-text-muted)', fontSize: 14, lineHeight: 1.6, ...style }}>{children}</p>;
}

export function CardContent({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return <div style={{ padding: 24, ...style }}>{children}</div>;
}

export function CardFooter({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return <div style={{ padding: '0 24px 24px', ...style }}>{children}</div>;
}

const badgeVariants: Record<string, CSSProperties> = {
  default: {
    background: 'rgba(255,255,255,0.04)',
    color: 'var(--ghost-text-muted)',
    border: '1px solid rgba(255,255,255,0.08)'
  },
  primary: {
    background: 'var(--ghost-accent-soft)',
    color: '#bfdbfe',
    border: '1px solid rgba(59,130,246,0.3)'
  },
  success: {
    background: 'rgba(34,197,94,0.12)',
    color: '#bbf7d0',
    border: '1px solid rgba(34,197,94,0.24)'
  },
  warning: {
    background: 'rgba(245,158,11,0.12)',
    color: '#fde68a',
    border: '1px solid rgba(245,158,11,0.24)'
  },
  danger: {
    background: 'rgba(239,68,68,0.12)',
    color: '#fecaca',
    border: '1px solid rgba(239,68,68,0.24)'
  }
};

export function Badge({ children, tone = 'default' }: { children: ReactNode; tone?: keyof typeof badgeVariants }) {
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 6,
        width: 'fit-content',
        borderRadius: 999,
        padding: '6px 10px',
        fontSize: 12,
        fontWeight: 600,
        letterSpacing: '0.02em',
        lineHeight: 1,
        ...badgeVariants[tone]
      }}
    >
      {children}
    </span>
  );
}

const buttonVariants: Record<string, CSSProperties> = {
  primary: {
    background: 'linear-gradient(180deg, rgba(59,130,246,0.96) 0%, rgba(37,99,235,0.96) 100%)',
    color: '#ffffff',
    border: '1px solid rgba(96,165,250,0.28)'
  },
  secondary: {
    background: 'rgba(255,255,255,0.03)',
    color: 'var(--ghost-text)',
    border: '1px solid var(--ghost-border)'
  },
  ghost: {
    background: 'transparent',
    color: 'var(--ghost-text-muted)',
    border: '1px solid transparent'
  }
};

export function Button({ children, tone = 'secondary' }: { children: ReactNode; tone?: keyof typeof buttonVariants }) {
  return (
    <button
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: 42,
        padding: '0 14px',
        borderRadius: 14,
        fontWeight: 600,
        cursor: 'default',
        boxShadow: tone === 'primary' ? '0 18px 30px rgba(37,99,235,0.18)' : 'none',
        ...buttonVariants[tone]
      }}
    >
      {children}
    </button>
  );
}

export function Progress({ value }: { value: number }) {
  return (
    <div style={{ width: '100%', height: 10, borderRadius: 999, background: 'rgba(255,255,255,0.05)', overflow: 'hidden', border: '1px solid rgba(255,255,255,0.05)' }}>
      <div
        style={{
          width: String(Math.min(100, Math.max(0, value))) + '%',
          height: '100%',
          borderRadius: 999,
          background: 'linear-gradient(90deg, rgba(59,130,246,0.9) 0%, rgba(96,165,250,0.95) 100%)',
          boxShadow: '0 0 18px rgba(59,130,246,0.35)'
        }}
      />
    </div>
  );
}

export function Slider({ value, label }: { value: number; label?: string }) {
  return (
    <div style={{ display: 'grid', gap: 8 }}>
      {label ? (
        <div style={{ display: 'flex', justifyContent: 'space-between', gap: 12, fontSize: 13, color: 'var(--ghost-text-muted)' }}>
          <span>{label}</span>
          <span style={{ color: 'var(--ghost-text)' }}>{value}%</span>
        </div>
      ) : null}
      <div style={{ position: 'relative', height: 10, borderRadius: 999, background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.06)' }}>
        <div style={{ position: 'absolute', inset: '0 auto 0 0', width: String(value) + '%', borderRadius: 999, background: 'linear-gradient(90deg, rgba(59,130,246,0.72) 0%, rgba(147,197,253,0.92) 100%)' }} />
        <div style={{ position: 'absolute', top: '50%', left: 'calc(' + String(value) + '% - 7px)', width: 14, height: 14, borderRadius: 999, transform: 'translateY(-50%)', background: '#dbeafe', border: '1px solid rgba(59,130,246,0.9)', boxShadow: '0 0 0 6px rgba(59,130,246,0.12)' }} />
      </div>
    </div>
  );
}

export function Switch({ checked, label }: { checked: boolean; label: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, padding: '12px 14px', borderRadius: 16, border: '1px solid rgba(255,255,255,0.06)', background: 'rgba(255,255,255,0.02)' }}>
      <span style={{ fontSize: 13, color: 'var(--ghost-text-muted)' }}>{label}</span>
      <span
        aria-checked={checked}
        style={{
          position: 'relative',
          width: 44,
          height: 24,
          borderRadius: 999,
          background: checked ? 'rgba(59,130,246,0.7)' : 'rgba(255,255,255,0.08)',
          border: '1px solid rgba(255,255,255,0.08)'
        }}
      >
        <span
          style={{
            position: 'absolute',
            top: 2,
            left: checked ? 22 : 2,
            width: 18,
            height: 18,
            borderRadius: 999,
            background: '#f8fafc',
            transition: 'left 180ms ease'
          }}
        />
      </span>
    </div>
  );
}
