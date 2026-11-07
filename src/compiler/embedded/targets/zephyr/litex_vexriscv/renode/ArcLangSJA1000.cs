//
// An SJA1000-compatible CAN controller (PeliCAN mode) on the SoC bus, one
// register per 32-bit word, as an SJA1000-compatible open core (OpenCores
// "can") sits on a LiteX Wishbone bus. Written for ArcLang's processor in
// the loop: it is what Zephyr's SJA1000 driver (drivers/can/can_sja1000.c)
// needs, the bus itself is the bench: Receive() puts a frame in the receive
// FIFO, every frame the CPU transmits is kept (LastSent, SentCount).
// No bit timing, no arbitration, no error confinement: a transmission
// completes at once and is acknowledged.
//
using System;
using System.Collections.Generic;
using Antmicro.Renode.Core;
using Antmicro.Renode.Logging;
using Antmicro.Renode.Peripherals.Bus;

namespace Antmicro.Renode.Peripherals.ArcLang
{
    public class SJA1000 : IDoubleWordPeripheral, IKnownSize
    {
        public SJA1000(IMachine machine)
        {
            IRQ = new GPIO();
            Reset();
        }

        public long Size => 0x80;

        public GPIO IRQ { get; }

        public void Reset()
        {
            mode = ModeReset;
            interrupts = 0;
            interruptEnable = 0;
            transmitStatus = true;
            regs = new byte[32];
            transmitBuffer = new byte[13];
            fifo = new Queue<byte[]>();
            sent = new Dictionary<uint, byte[]>();
            sentCount = new Dictionary<uint, int>();
            Update();
        }

        // The bench: a frame on the bus (standard when id <= 0x7FF).
        public void Receive(uint id, byte byte0, int length)
        {
            var frame = new byte[13];
            var extended = id > 0x7FF;
            var dlc = Math.Min(length, 8);
            frame[0] = (byte)(dlc | (extended ? 0x80 : 0));
            int data;
            if(extended)
            {
                frame[1] = (byte)(id >> 21);
                frame[2] = (byte)(id >> 13);
                frame[3] = (byte)(id >> 5);
                frame[4] = (byte)((id & 0x1F) << 3);
                data = 5;
            }
            else
            {
                frame[1] = (byte)(id >> 3);
                frame[2] = (byte)((id & 0x7) << 5);
                data = 3;
            }
            if(dlc > 0)
            {
                frame[data] = byte0;
            }
            if((mode & ModeReset) != 0)
            {
                this.Log(LogLevel.Warning, "frame 0x{0:X} dropped: the controller is in reset mode", id);
                return;
            }
            fifo.Enqueue(frame);
            interrupts |= InterruptReceive;
            Update();
        }

        // The first data byte of the last frame transmitted with this id, -1 when none was.
        public int LastSent(uint id)
        {
            byte[] frame;
            if(!sent.TryGetValue(id, out frame))
            {
                return -1;
            }
            return frame.Length > 0 ? frame[0] : 0;
        }

        public int SentCount(uint id)
        {
            int n;
            return sentCount.TryGetValue(id, out n) ? n : 0;
        }

        public uint ReadDoubleWord(long offset)
        {
            var reg = (int)(offset / 4);
            switch(reg)
            {
            case Mod:
                return mode;
            case Sr:
                return Status();
            case Ir:
            {
                // Reading clears every flag but the receive one, held while the FIFO is not empty.
                var value = interrupts;
                interrupts &= InterruptReceive;
                Update();
                return value;
            }
            case Ier:
                return interruptEnable;
            case Rmc:
                return (uint)fifo.Count;
            }
            if(reg >= 16 && reg <= 28 && (mode & ModeReset) == 0)
            {
                // Operating mode: the receive buffer.
                return fifo.Count > 0 ? fifo.Peek()[reg - 16] : 0u;
            }
            return reg < regs.Length ? regs[reg] : 0u;
        }

        public void WriteDoubleWord(long offset, uint value)
        {
            var reg = (int)(offset / 4);
            var b = (byte)value;
            switch(reg)
            {
            case Mod:
                mode = b;
                return;
            case Cmr:
                Command(b);
                return;
            case Ier:
                interruptEnable = b;
                Update();
                return;
            case Sr:
            case Ir:
                return;
            }
            if(reg >= 16 && reg <= 28 && (mode & ModeReset) == 0)
            {
                transmitBuffer[reg - 16] = b;
                return;
            }
            if(reg < regs.Length)
            {
                regs[reg] = b;
            }
        }

        private void Command(byte command)
        {
            if((command & CommandReleaseReceiveBuffer) != 0 && fifo.Count > 0)
            {
                fifo.Dequeue();
                if(fifo.Count == 0)
                {
                    interrupts &= ~InterruptReceive;
                }
            }
            if((command & (CommandTransmit | CommandSelfReception)) != 0 && (mode & ModeReset) == 0)
            {
                var frame = (byte[])transmitBuffer.Clone();
                var extended = (frame[0] & 0x80) != 0;
                uint id;
                int data;
                if(extended)
                {
                    id = ((uint)frame[1] << 21) | ((uint)frame[2] << 13) | ((uint)frame[3] << 5) | ((uint)frame[4] >> 3);
                    data = 5;
                }
                else
                {
                    id = ((uint)frame[1] << 3) | ((uint)frame[2] >> 5);
                    data = 3;
                }
                var dlc = Math.Min(frame[0] & 0xF, 8);
                var payload = new byte[dlc];
                Array.Copy(frame, data, payload, 0, dlc);
                sent[id] = payload;
                int n;
                sentCount[id] = sentCount.TryGetValue(id, out n) ? n + 1 : 1;
                if((command & CommandSelfReception) != 0)
                {
                    fifo.Enqueue(frame);
                    interrupts |= InterruptReceive;
                }
                transmitStatus = true;
                interrupts |= InterruptTransmit;
            }
            Update();
        }

        private uint Status()
        {
            uint status = StatusTransmitBuffer;
            if(fifo.Count > 0)
            {
                status |= StatusReceiveBuffer;
            }
            if(transmitStatus)
            {
                status |= StatusTransmissionComplete;
            }
            return status;
        }

        private void Update()
        {
            IRQ.Set((interrupts & interruptEnable) != 0);
        }

        private byte mode;
        private uint interrupts;
        private uint interruptEnable;
        private bool transmitStatus;
        private byte[] regs;
        private byte[] transmitBuffer;
        private Queue<byte[]> fifo;
        private Dictionary<uint, byte[]> sent;
        private Dictionary<uint, int> sentCount;

        private const int Mod = 0;
        private const int Cmr = 1;
        private const int Sr = 2;
        private const int Ir = 3;
        private const int Ier = 4;
        private const int Rmc = 29;
        private const byte ModeReset = 0x1;
        private const byte CommandTransmit = 0x1;
        private const byte CommandReleaseReceiveBuffer = 0x4;
        private const byte CommandSelfReception = 0x10;
        private const uint StatusReceiveBuffer = 0x1;
        private const uint StatusTransmitBuffer = 0x4;
        private const uint StatusTransmissionComplete = 0x8;
        private const uint InterruptReceive = 0x1;
        private const uint InterruptTransmit = 0x2;
    }
}
