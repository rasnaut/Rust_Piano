import { useEffect, useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type PianoKey = {
  key: string;
  isBlack: boolean;
};

const WAVEFORM_UPDATE_INTERVAL = 1000 / 60; // ~60 FPS

function WaveformVisualizer() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animationId: number;
    let lastUpdate = 0;

    const draw = async (timestamp: number) => {
      // Throttle updates to target FPS
      if (timestamp - lastUpdate < WAVEFORM_UPDATE_INTERVAL) {
        animationId = requestAnimationFrame(draw);
        return;
      }
      lastUpdate = timestamp;

      try {
        // Fetch waveform data from Rust backend
        const samples = await invoke<number[]>("get_waveform_data");

        // Clear canvas
        ctx.fillStyle = "#0a0a15";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        // Draw grid lines
        ctx.strokeStyle = "#1a1a2e";
        ctx.lineWidth = 1;
        ctx.beginPath();
        // Horizontal center line
        ctx.moveTo(0, canvas.height / 2);
        ctx.lineTo(canvas.width, canvas.height / 2);
        // Vertical lines
        for (let i = 0; i < 8; i++) {
          const x = (canvas.width / 8) * i;
          ctx.moveTo(x, 0);
          ctx.lineTo(x, canvas.height);
        }
        ctx.stroke();

        // Draw waveform
        if (samples && samples.length > 0) {
          ctx.strokeStyle = "#4a9eff";
          ctx.lineWidth = 2;
          ctx.beginPath();

          const sliceWidth = canvas.width / samples.length;
          let x = 0;

          for (let i = 0; i < samples.length; i++) {
            const sample = samples[i];
            // Map sample (-1 to 1) to canvas height
            const y = ((1 - sample) / 2) * canvas.height;

            if (i === 0) {
              ctx.moveTo(x, y);
            } else {
              ctx.lineTo(x, y);
            }
            x += sliceWidth;
          }
          ctx.stroke();

          // Add glow effect
          ctx.strokeStyle = "rgba(74, 158, 255, 0.3)";
          ctx.lineWidth = 6;
          ctx.stroke();
        }
      } catch (error) {
        // Command not implemented yet - draw placeholder
        ctx.fillStyle = "#0a0a15";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = "#333";
        ctx.font = "14px monospace";
        ctx.textAlign = "center";
        ctx.fillText(
          "Waveform visualization not implemented",
          canvas.width / 2,
          canvas.height / 2,
        );
      }

      animationId = requestAnimationFrame(draw);
    };

    animationId = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, []);

  return (
    <div className="waveform-container">
      <canvas
        ref={canvasRef}
        width={600}
        height={150}
        className="waveform-canvas"
      />
    </div>
  );
}

function App() {
  const [pianoKeys, setPianoKeys] = useState<PianoKey[]>([]);
  const [pressedKeys, setPressedKeys] = useState<Set<string>>(new Set());

  useEffect(() => {
    invoke<[string, boolean][]>("get_piano_keys").then((keys) => {
      setPianoKeys(keys.map(([key, isBlack]) => ({ key, isBlack })));
    });
  }, []);

  const handleNoteOn = useCallback(
    async (key: string) => {
      if (pressedKeys.has(key)) return;
      setPressedKeys((prev) => new Set(prev).add(key));
      await invoke("note_on", { key });
    },
    [pressedKeys],
  );

  const handleNoteOff = useCallback(async (key: string) => {
    setPressedKeys((prev) => {
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
    await invoke("note_off", { key });
  }, []);

  useEffect(() => {
    const validKeys = new Set(pianoKeys.map((k) => k.key));

    const handleKeyDown = (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      if (validKeys.has(key) && !e.repeat) {
        handleNoteOn(key);
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      if (validKeys.has(key)) {
        handleNoteOff(key);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, [pianoKeys, handleNoteOn, handleNoteOff]);

  const whiteKeys = pianoKeys.filter((k) => !k.isBlack);
  const blackKeys = pianoKeys.filter((k) => k.isBlack);

  // Map black keys to their positions relative to white keys
  const blackKeyPositions: { [key: string]: number } = {};
  let whiteIndex = 0;
  for (const pk of pianoKeys) {
    if (!pk.isBlack) {
      whiteIndex++;
    } else {
      // Black key goes after the previous white key
      blackKeyPositions[pk.key] = whiteIndex;
    }
  }

  return (
    <main className="piano-container">
      <h1>Piano</h1>
      <WaveformVisualizer />
      <div className="piano">
        <div className="white-keys">
          {whiteKeys.map((pk) => (
            <button
              key={pk.key}
              className={`white-key ${pressedKeys.has(pk.key) ? "pressed" : ""}`}
              onMouseDown={() => handleNoteOn(pk.key)}
              onMouseUp={() => handleNoteOff(pk.key)}
              onMouseLeave={() => {
                if (pressedKeys.has(pk.key)) handleNoteOff(pk.key);
              }}
            >
              <span className="key-label">{pk.key.toUpperCase()}</span>
            </button>
          ))}
        </div>
        <div className="black-keys">
          {blackKeys.map((pk) => {
            const pos = blackKeyPositions[pk.key];
            const whiteKeyWidth = 60;
            const left = pos * whiteKeyWidth - 18;
            return (
              <button
                key={pk.key}
                className={`black-key ${pressedKeys.has(pk.key) ? "pressed" : ""}`}
                style={{ left: `${left}px` }}
                onMouseDown={() => handleNoteOn(pk.key)}
                onMouseUp={() => handleNoteOff(pk.key)}
                onMouseLeave={() => {
                  if (pressedKeys.has(pk.key)) handleNoteOff(pk.key);
                }}
              >
                <span className="key-label">{pk.key.toUpperCase()}</span>
              </button>
            );
          })}
        </div>
      </div>
      <p className="hint">Click keys or use keyboard</p>
    </main>
  );
}

export default App;
