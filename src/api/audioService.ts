/**
 * Procedural Audio Service for Pulsar Messenger
 * Generates futuristic "blips" and "clicks" using Web Audio API
 */

class AudioService {
  private ctx: AudioContext | null = null;

  private init() {
    if (!this.ctx) {
      this.ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
    }
  }

  public play(type: 'send' | 'receive' | 'click') {
    this.init();
    if (!this.ctx) return;

    // Fast-path for browser audio policies (resume on user interaction)
    if (this.ctx.state === 'suspended') {
      this.ctx.resume();
    }

    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc.connect(gain);
    gain.connect(this.ctx.destination);

    const now = this.ctx.currentTime;

    switch (type) {
      case 'send':
        // A rising chirp
        osc.type = 'sine';
        osc.frequency.setValueAtTime(880, now);
        osc.frequency.exponentialRampToValueAtTime(1760, now + 0.1);
        gain.gain.setValueAtTime(0.1, now);
        gain.gain.exponentialRampToValueAtTime(0.01, now + 0.1);
        osc.start(now);
        osc.stop(now + 0.1);
        break;

      case 'receive':
        // A double blip
        osc.type = 'sine';
        osc.frequency.setValueAtTime(1320, now);
        osc.frequency.exponentialRampToValueAtTime(880, now + 0.05);
        gain.gain.setValueAtTime(0.1, now);
        gain.gain.exponentialRampToValueAtTime(0.01, now + 0.05);
        osc.start(now);
        osc.stop(now + 0.05);

        // Second part
        const osc2 = this.ctx.createOscillator();
        const gain2 = this.ctx.createGain();
        osc2.connect(gain2);
        gain2.connect(this.ctx.destination);
        osc2.type = 'sine';
        osc2.frequency.setValueAtTime(1760, now + 0.07);
        osc2.frequency.exponentialRampToValueAtTime(1320, now + 0.12);
        gain2.gain.setValueAtTime(0.1, now + 0.07);
        gain2.gain.exponentialRampToValueAtTime(0.01, now + 0.12);
        osc2.start(now + 0.07);
        osc2.stop(now + 0.12);
        break;

      case 'click':
        // Short technical tick
        osc.type = 'square';
        osc.frequency.setValueAtTime(440, now);
        gain.gain.setValueAtTime(0.05, now);
        gain.gain.exponentialRampToValueAtTime(0.01, now + 0.02);
        osc.start(now);
        osc.stop(now + 0.02);
        break;
    }
  }
}

export const audioService = new AudioService();
