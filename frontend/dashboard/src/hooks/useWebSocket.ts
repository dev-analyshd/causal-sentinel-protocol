import { useEffect, useState, useRef } from 'react';

interface WebSocketData {
  type: string;
  total_events?: number;
  last_block?: number;
  perceptual_entropy?: number;
}

export function useWebSocket(url: string) {
  const [connected, setConnected] = useState(false);
  const [data, setData] = useState<WebSocketData | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
    };

    ws.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data);
        setData(parsed);
      } catch {
        // Ignore non-JSON messages
      }
    };

    ws.onclose = () => {
      setConnected(false);
    };

    ws.onerror = () => {
      setConnected(false);
    };

    return () => {
      ws.close();
    };
  }, [url]);

  return { connected, data };
}
